use std::error::Error;
use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};

use crate::rendezvous::Peer;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

const CLEANUP_INTERVAL_SECS: u64 = 5;
const STALE_PEER_TIMEOUT_SECS: u64 = 20;
const SCAN_RESPONSE_TIMEOUT_SECS: u64 = 10;
const USERNAME_POLL_INTERVAL_MILLIS: u64 = 300;

type ActivePeers = Arc<Mutex<HashMap<PeripheralId, (Peer, tokio::time::Instant)>>>;

/// Sets up the Bluetooth manager and returns the first available hardware adapter.
async fn setup_bluetooth_adapter() -> Result<Adapter, Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        return Err("No Bluetooth adapters found on this system".into());
    }

    // Pick the first available Bluetooth adapter natively detected by the OS
    let adapter = adapters.into_iter().nth(0).unwrap();
    Ok(adapter)
}

/// Safely attempts to decode the URL-safe Base64 string that was broadcasted
/// into the IPv4 address `[u8; 4]` and the HTTP port number `u16`.
fn decode_network_info_from_name(name: &str) -> Option<([u8; 4], u16)> {
    // We expect exactly 6 bytes of encoded data (8 characters in base64 without padding)
    let decoded = URL_SAFE_NO_PAD.decode(name).ok()?;

    if decoded.len() != 6 {
        return None;
    }

    let mut ip = [0u8; 4];
    ip.copy_from_slice(&decoded[0..4]);

    let mut port_bytes = [0u8; 2];
    port_bytes.copy_from_slice(&decoded[4..6]);
    let port = u16::from_be_bytes(port_bytes);

    Some((ip, port))
}

fn extract_network_payload(local_name: &str) -> &str {
    local_name.split('|').next().unwrap_or(local_name)
}

fn extract_username_from_scan_response(local_name: &str) -> Option<String> {
    if let Some((_, username)) = local_name.split_once('|') {
        let cleaned = username.trim();
        if !cleaned.is_empty() {
            return Some(cleaned.to_string());
        }
    }

    if decode_network_info_from_name(local_name).is_some() {
        return None;
    }

    let cleaned = local_name.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

/// Asynchronously fetches properties of a peripheral.
/// Filters based on the App UUID, and then safely decodes its local name into a `Peer`.
async fn filter_and_parse_peripheral(peripheral: &Peripheral, target_uuid: Uuid) -> Option<Peer> {
    let properties = peripheral.properties().await.ok()??;

    // 1. Verify that this device is in our CargoDrop ecosystem
    //    (by checking the primary advertised service UUID)
    if !properties.services.contains(&target_uuid) {
        return None;
    }

    // 2. Extract the device's encoded local name chunk
    let local_name = properties.local_name?;
    let network_payload = extract_network_payload(&local_name);

    // 3. Decode the base64 network info safely
    let (ip, port) = decode_network_info_from_name(network_payload)?;

    Some(Peer {
        ip,
        port,
        username: network_payload.to_string(),
    })
}

async fn wait_for_username(
    adapter: Adapter,
    peer_id: PeripheralId,
    peers: ActivePeers,
    timeout: Duration,
) {
    let start = tokio::time::Instant::now();

    loop {
        if start.elapsed() >= timeout {
            let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
            println!(
                "[{}] --- USERNAME UNRESOLVED ---\n  - peer {} did not publish a username within {}s.",
                time_str,
                peer_id,
                timeout.as_secs()
            );
            return;
        }

        if let Ok(peripheral) = adapter.peripheral(&peer_id).await {
            if let Ok(Some(properties)) = peripheral.properties().await {
                if let Some(local_name) = properties.local_name {
                    if let Some(resolved_username) = extract_username_from_scan_response(&local_name)
                    {
                        let mut guard = peers.lock().await;
                        if let Some((peer, _)) = guard.get_mut(&peer_id) {
                            if peer.username != resolved_username {
                                peer.username = resolved_username.clone();
                                let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                                println!("[{}] --- USERNAME RESOLVED ---", time_str);
                                println!(
                                    "  = IP: {:?}, Port: {}, Username: '{}'",
                                    peer.ip, peer.port, peer.username
                                );
                            }
                        }
                        return;
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(USERNAME_POLL_INTERVAL_MILLIS)).await;
    }
}

/// Main entrypoint loop to continuously stream and discover peers instantaneously.
pub async fn discover_rendezvous() -> Result<(), Box<dyn Error>> {
    let target_uuid = Uuid::parse_str(crate::rendezvous::RendezvousManager::APP_SERVICE_UUID)?;

    println!("Initializing Bluetooth Discovery Adapter...");
    let adapter = setup_bluetooth_adapter().await?;

    // Subscribe to the unbuffered native hardware event stream
    let mut events = adapter.events().await?;

    // Start continuous hardware scanning
    let scan_filter = ScanFilter {
        services: vec![target_uuid],
    };
    adapter.start_scan(scan_filter).await?;

    println!("Entering active CargoDrop BLE streaming loop...");

    // Track when the app started to filter out initial "ghost" OS cache dumps
    let app_start_time = tokio::time::Instant::now();

    // Store active peers (Key: Base64 payload Name, Value: (Peer, Last Seen Timestamp))
    let active_peers: ActivePeers = Arc::new(Mutex::new(HashMap::new()));

    // Create an interval timer that ticks every 5 seconds to run our "Device Lost" disconnect logic
    let mut cleanup_interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));

    loop {
        tokio::select! {
            Some(event) = events.next() => {
                match &event {
                    // We react to events that indicate LIVE packets are arriving right now.
                    CentralEvent::DeviceDiscovered(id) |
                    CentralEvent::DeviceUpdated(id) |
                    CentralEvent::ManufacturerDataAdvertisement { id, .. } |
                    CentralEvent::ServiceDataAdvertisement { id, .. } |
                    CentralEvent::RssiUpdate { id, .. } => {
                        // We simulate "clearing the peers cache" from previous discoveries by ignoring packets in the 1st second
                        if let CentralEvent::DeviceDiscovered(_) = event {
                            if app_start_time.elapsed().as_millis() < 1000 {
                                continue;
                            }
                        }

                        if let Ok(peripheral) = adapter.peripheral(id).await {
                            if let Some(peer) = filter_and_parse_peripheral(&peripheral, target_uuid).await {
                                let now = tokio::time::Instant::now();
                                let peer_id = id.clone();
                                let mut guard = active_peers.lock().await;
                                let is_new_peer = !guard.contains_key(&peer_id);

                                if is_new_peer {
                                    let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                                    println!("[{}] --- PEER DETECTED ---", time_str);
                                    println!(
                                        "  + IP: {:?}, Port: {}, Username (placeholder): '{}'",
                                        peer.ip, peer.port, peer.username
                                    );
                                }

                                guard.insert(peer_id.clone(), (peer, now));
                                drop(guard);

                                if is_new_peer {
                                    let peers = Arc::clone(&active_peers);
                                    let adapter_clone = adapter.clone();
                                    tokio::spawn(async move {
                                        wait_for_username(
                                            adapter_clone,
                                            peer_id,
                                            peers,
                                            Duration::from_secs(SCAN_RESPONSE_TIMEOUT_SECS),
                                        )
                                        .await;
                                    });
                                }
                            }
                        }
                    }
                    _ => {} // Ignore connection/disconnection events since we're connectionless
                }
            }

            _ = cleanup_interval.tick() => {
                let now = tokio::time::Instant::now();
                let mut guard = active_peers.lock().await;
                guard.retain(|_id, (peer, last_seen)| {
                    if now.duration_since(*last_seen).as_secs() > STALE_PEER_TIMEOUT_SECS {
                        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                        println!("[{}] --- PEER LOST ---", time_str);
                        println!(
                            "  - IP: {:?}, Port: {}, Username: '{}' went offline.",
                            peer.ip, peer.port, peer.username
                        );
                        false // Drop from HashMap
                    } else {
                        true // Keep in HashMap
                    }
                });
            }
        }
    }
}
