use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};

use crate::rendezvous::Peer;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
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

/// Main entrypoint loop to continuously discover peers every 10 seconds.
pub async fn discover_rendezvous() -> Result<(), Box<dyn Error>> {
    let target_uuid = Uuid::parse_str(crate::rendezvous::RendezvousManager::APP_SERVICE_UUID)?;
    
    println!("Initializing Bluetooth Discovery Adapter...");
    let adapter = setup_bluetooth_adapter().await?;
    
    // Filter applied at the hardware hardware level for efficiency
    let scan_filter = ScanFilter {
        services: vec![target_uuid],
    };
    
    println!("Entering active CargoDrop BLE scanning loop...");
    
    loop {
        // Start hardware scanning
        adapter.start_scan(scan_filter.clone()).await?;
        
        // Accumulate detections for 10 seconds
        time::sleep(Duration::from_secs(10)).await;
        
        // Gather and process all strictly cached BLE peripherals 
        let peripherals = adapter.peripherals().await?;
        let mut detected_peers = Vec::new();
        
        for p in peripherals {
            if let Some(peer) = filter_and_parse_peripheral(&p, target_uuid).await {
                // We successfully vetted the peripheral and decoded its payload
                detected_peers.push(peer);
            }
        }
        
        // Output detection results
        println!("--- Scan Cycle Complete ---");
        if detected_peers.is_empty() {
             println!("No peers detected nearby.");
        } else {
             println!("Detected {} peer(s):", detected_peers.len());
             for peer in detected_peers {
                  println!("  - IP: {:?}, Port: {}, Base64: '{}'", peer.ip, peer.port, peer.name);
             }
        }
        
        // Stop scanning to clear cached peers
        adapter.stop_scan().await?;
    }
}
