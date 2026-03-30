use std::error::Error;
use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};

use crate::rendezvous::Peer;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures::stream::StreamExt;
use uuid::Uuid;

use super::{APP_SERVICE_UUID, USERNAME_LEN_OFFSET, USERNAME_OFFSET};

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

fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

/// Decodes the URL-safe Base64 payload into (IPv4, port, username).
/// Layout: [4 bytes IPv4][2 bytes port][1 byte username_len][N bytes username].
fn decode_network_info_from_name(name: &str) -> Option<([u8; 4], u16, String)> {
    let decoded = URL_SAFE_NO_PAD.decode(name).ok()?;

    if decoded.len() < USERNAME_OFFSET {
        return None;
    }

    let username_len = decoded[USERNAME_LEN_OFFSET] as usize;
    if decoded.len() != USERNAME_OFFSET + username_len {
        return None;
    }

    let mut ip = [0u8; 4];
    ip.copy_from_slice(&decoded[0..4]);

    let mut port_bytes = [0u8; 2];
    port_bytes.copy_from_slice(&decoded[4..6]);
    let port = u16::from_be_bytes(port_bytes);

    let username_bytes = &decoded[USERNAME_OFFSET..USERNAME_OFFSET + username_len];
    let username = String::from_utf8(username_bytes.to_vec()).ok()?;

    Some((ip, port, username))
}

/// Asynchronously fetches properties of a peripheral.
/// Filters based on the App UUID, and then safely decodes its local name into a `Peer`.
async fn filter_and_parse_peripheral(
    peripheral: &Peripheral,
    target_uuid: Uuid,
) -> Option<(String, Peer)> {
    let properties = peripheral.properties().await.ok()??;

    // 1. Verify that this device is in our CargoDrop ecosystem
    //    (by checking the primary advertised service UUID)
    if !properties.services.contains(&target_uuid) {
        return None;
    }

    // 2. Extract the device's encoded local name chunk
    let local_name = properties.local_name?;

    // 3. Decode the base64 network info safely
    let (ip, port, username) = decode_network_info_from_name(&local_name)?;

    Some((
        local_name,
        Peer {
            ip,
            port,
            username,
        },
    ))
}

/// Main entrypoint loop to continuously stream and discover peers instantaneously.
pub async fn discover_rendezvous() -> Result<(), Box<dyn Error>> {
    let target_uuid = Uuid::parse_str(APP_SERVICE_UUID)?;

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

    // Store active peers (Key: encoded payload, Value: (Peer, Last Seen Timestamp))
    let mut active_peers: std::collections::HashMap<String, (Peer, tokio::time::Instant)> =
        std::collections::HashMap::new();

    // Create an interval timer that ticks every 5 seconds to run our "Device Lost" disconnect logic
    let mut cleanup_interval = tokio::time::interval(Duration::from_secs(5));

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
                            if let Some((payload_key, peer)) = filter_and_parse_peripheral(&peripheral, target_uuid).await {
                                let now = tokio::time::Instant::now();
                                let peer_ip = format_ip(peer.ip);
                                
                                // Only print if this is a brand new peer we haven't seen yet
                                // if !active_peers.contains_key(&payload_key) {
                                    let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                                    let display_name = format!("{}_????", peer.username);
                                    println!("[{}] --- PEER DETECTED ---", time_str);
                                    println!(
                                        "  + Username: '{}', IP: {}, Port: {}",
                                        display_name, peer_ip, peer.port
                                    );
                                // }
                                
                                // Insert or update the heartbeat timer
                                active_peers.insert(payload_key, (peer, now));
                            }
                        }
                    }
                    _ => {} // Ignore connection/disconnection events since we're connectionless
                }
            }

            _ = cleanup_interval.tick() => {
                let now = tokio::time::Instant::now();
                active_peers.retain(|name, (peer, last_seen)| {
                    if now.duration_since(*last_seen).as_secs() > 20 {
                        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                        let peer_ip = format_ip(peer.ip);
                        println!("[{}] --- PEER LOST ---", time_str);
                        println!(
                            "  - Username: '{}', IP: {}, Port: {} went offline (payload='{}').",
                            peer.username, peer_ip, peer.port, name
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
