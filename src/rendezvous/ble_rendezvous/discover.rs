use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};

use crate::rendezvous::Peer;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use futures::stream::StreamExt;
use uuid::Uuid;

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
    
    // 3. Decode the base64 network info safely
    let (ip, port) = decode_network_info_from_name(&local_name)?;
    
    Some(Peer {
        ip,
        port,
        name: local_name,
    })
}

/// Main entrypoint loop to continuously stream and discover peers instantaneously.
pub async fn discover_rendezvous() -> Result<(), Box<dyn Error>> {
    let target_uuid = Uuid::parse_str(crate::rendezvous::RendezvousManager::APP_SERVICE_UUID)?;
    
    println!("Initializing Bluetooth Discovery Adapter...");
    let adapter = setup_bluetooth_adapter().await?;
    
    // Subscribe to the unbuffered native hardware event stream
    let mut events = adapter.events().await?;
    
    // Start continuous hardware scanning
    let scan_filter = ScanFilter { services: vec![target_uuid] };
    adapter.start_scan(scan_filter).await?;
    
    println!("Entering active CargoDrop BLE streaming loop...");

    // Store active peers (Key: Base64 payload Name, Value: (Peer, Last Seen Timestamp))
    let mut active_peers: std::collections::HashMap<String, (Peer, tokio::time::Instant)> = std::collections::HashMap::new();
    
    // Create an interval timer that ticks every 5 seconds to run our "Device Lost" disconnect logic
    let mut cleanup_interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            // Branch 1: A hardware event fires the instant the radio receives a packet
            Some(event) = events.next() => {
                match event {
                    // Triggers anytime a packet is heard
                    CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                        if let Ok(peripheral) = adapter.peripheral(&id).await {
                            if let Some(peer) = filter_and_parse_peripheral(&peripheral, target_uuid).await {
                                let now = tokio::time::Instant::now();
                                let peer_name = peer.name.clone();
                                
                                // Only print if this is a brand new peer we haven't seen yet
                                // if !active_peers.contains_key(&peer_name) {
                                    println!("--- [INSTANT] PEER DETECTED ---");
                                    println!("  + IP: {:?}, Port: {}, Base64: '{}'", peer.ip, peer.port, peer.name);
                                // }
                                
                                // Insert or update the heartbeat timer
                                active_peers.insert(peer_name, (peer, now));
                            }
                        }
                    }
                    _ => {} // Ignore connection/disconnection standard events since we're connectionless
                }
            }
            
            // Branch 2: Our 5-second ticker fires to check for timeout "disconnects"
            _ = cleanup_interval.tick() => {
                let now = tokio::time::Instant::now();
                active_peers.retain(|name, (peer, last_seen)| {
                    // If we haven't heard an advertising packet from them for 15 seconds, they're gone
                    if now.duration_since(*last_seen).as_secs() > 15 {
                        println!("--- [TIMEOUT] PEER LOST ---");
                        println!("  - IP: {:?}, Port: {}, Base64: '{}' went offline.", peer.ip, peer.port, name);
                        false // Drop from HashMap
                    } else {
                        true // Keep in HashMap
                    }
                });
            }
        }
    }
}
