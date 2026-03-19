use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use tokio::time;
use uuid::Uuid;

use super::APP_SERVICE_UUID;

/// Starts a Bluetooth scan to discover nearby devices that are advertising
/// our specific application service UUID (`APP_SERVICE_UUID`).
pub async fn discover_app_peripherals() -> Result<(), Box<dyn Error>> {
    // Initialize the Bluetooth manager to talk to the OS
    let manager = Manager::new().await?;

    // Get available bluetooth adapters and pick the first one
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .next()
        .expect("No Bluetooth adapters found. Please make sure Bluetooth is enabled and authorized.");

    // We must securely register for events BEFORE we ask the adapter to start scanning.
    // This allows us to catch the first advertisement packets straight away.
    let mut events = central.events().await?;

    println!("Starting Bluetooth scan specifically for our CargoDrop Service UUID...");
    
    // We only want to discover devices that actively advertise our APP_SERVICE_UUID.
    // Instead of filtering manually after the fact, we can also pass a filter to `start_scan`
    // but doing it manually allows more robust inspection if needed.
    // For now, we use a basic default scan, then filter manually to demonstrate the logic.
    central.start_scan(ScanFilter::default()).await?;

    // Sleep timer for 10 seconds of scanning
    let timeout = time::sleep(Duration::from_secs(10));
    tokio::pin!(timeout); // pins the timeout variable to the stack for safe async polling

    // We will collect active unique IDs over the 10 seconds
    let mut discovered_ids = HashSet::new();

    loop {
        // tokio::select! awaits on multiple async operations simultaneously
        // It returns as soon as the first branch completes.
        tokio::select! {
            _ = &mut timeout => {
                break; // Timer finished
            }
            Some(event) = events.next() => {
                match event {
                    CentralEvent::DeviceDiscovered(id)
                    | CentralEvent::DeviceUpdated(id)
                    | CentralEvent::DeviceConnected(id)
                    | CentralEvent::DeviceDisconnected(id)
                    | CentralEvent::ManufacturerDataAdvertisement { id, .. }
                    | CentralEvent::ServiceDataAdvertisement { id, .. }
                    | CentralEvent::ServicesAdvertisement { id, .. } => {
                        discovered_ids.insert(id);
                    }
                    _ => {}
                }
            }
        }
    }

    println!("Scan finished. Checking {} devices for our CargoDrop service...", discovered_ids.len());

    let mut cargodrop_devices = 0;

    for id in discovered_ids {
        if let Ok(peripheral) = central.peripheral(&id).await {
            // Retrieve properties of this specific peripheral
            let properties = peripheral.properties().await?;
            if let Some(props) = properties {
                // Determine if this peripheral is advertising our sought UUID
                if filter_by_uuid(&props) {
                    cargodrop_devices += 1;
                    
                    let name = props.local_name.unwrap_or_else(|| String::from("Unnamed-CargoDrop"));
                    let addr = props.address.to_string();
                    let rssi = props.rssi.unwrap_or(0); // If RSSI exists, else 0 (might be cached)
                    
                    println!("  🚀 FOUND CargoDrop Device!");
                    println!("     - Name: {:<20} | Address: {} | RSSI: {} dBm", name, addr, rssi);
                }
            }
        }
    }

    if cargodrop_devices == 0 {
        println!("No nearby CargoDrop compatible devices found.");
    } else {
        println!("Total nearby CargoDrop devices: {}", cargodrop_devices);
    }

    // Always ensure the adapter stops scanning to save power and free resources
    central.stop_scan().await?;

    Ok(())
}

/// A filter function that evaluates peripheral properties to check if it advertises our exact UUID.
/// Returns true if the device explicitly advertises `APP_SERVICE_UUID`.
fn filter_by_uuid(properties: &btleplug::api::PeripheralProperties) -> bool {
    // Determine the expected UUID at run-time from our static definition
    let target_uuid = Uuid::parse_str(APP_SERVICE_UUID).unwrap_or_default();
    
    // Check if the UUID list in the advertisement matches our target
    properties.services.contains(&target_uuid)
}
