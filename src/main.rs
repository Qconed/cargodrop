use std::error::Error;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time;
use futures::stream::StreamExt;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the Bluetooth manager
    let manager = Manager::new().await?;

    // Get and choose an adapter (physical bluetooth chip)
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .nth(0)
        .expect("No Bluetooth adapters found. Please make sure Bluetooth is enabled.");

    // Create an event stream BEFORE starting the scan
    let mut events = central.events().await?;

    println!("Starting Bluetooth scan to find actively broadcasting devices...");
    
    // Start scanning for nearby devices -------------------------
    central.start_scan(ScanFilter::default()).await?;
    
    // We will listen for events for 10 seconds
    let timeout = time::sleep(Duration::from_secs(10));
    tokio::pin!(timeout); // give timer ownership to tokio for time management

    let mut discovered_ids = HashSet::new();

    // Listen to the event stream to capture only devices that are broadcasting *right now*
    loop {
        tokio::select! { // run multiple async tasks and wait for the first one to complete. 
            // if the timer completes first, break the loop
            _ = &mut timeout => {
                break;
            }
            // if an event is received, process it
            Some(event) = events.next() => {
                match event {
                    // whatever the event, grab the id of device.
                    CentralEvent::DeviceDiscovered(id)
                    | CentralEvent::DeviceUpdated(id)
                    | CentralEvent::DeviceConnected(id)
                    | CentralEvent::DeviceDisconnected(id)
                    | CentralEvent::ManufacturerDataAdvertisement { id, .. } // other parameters are ignored with ".."
                    | CentralEvent::ServiceDataAdvertisement { id, .. }
                    | CentralEvent::ServicesAdvertisement { id, .. } => {
                        discovered_ids.insert(id);
                    }
                    _ => {} // safely ignore other events
                }
            }
        }
    }
    
    println!("Found {} devices that emitted an event. Filtering for actively nearby ones...", discovered_ids.len());

    let mut nearby_count = 0;

    // Iterate over the actively discovered devices
    for id in discovered_ids {
        if let Ok(peripheral) = central.peripheral(&id).await { 
            // right side of the operation returns a Result<Peripheral, Error>
            //  => if the result is Ok, the value is bound to peripheral 
            let properties = peripheral.properties().await?;
            
            if let Some(props) = properties {
                // Test if a device is *nearby* (and not a cached OS device) :
                // via the presence of a recent RSSI (Signal Strength) value. Cached devices have no RSSI.
                if let Some(rssi) = props.rssi {
                    nearby_count += 1;
                    let name = props.local_name.unwrap_or_else(|| String::from("(unknown)")); // in case
                    let addr = props.address.to_string(); // Bluetooth Device Address (BDAddr) = unique identifier

                    println!(
                        "  - Name: {:<20} | Address: {} | RSSI: {} dBm",
                        name, addr, rssi
                    );
                }
            }
        }
    }

    println!("Total nearby broadcasting devices: {}", nearby_count);
    println!("(Disclaimer: Unnamed devices can appear, these are devices like smart tv, light bulbs...)");
    
    // Stop scanning safely after we are completely finished getting properties
    central.stop_scan().await?;

    Ok(())
}
