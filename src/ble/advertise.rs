use std::error::Error;
use tokio::sync::mpsc;

use ble_peripheral_rust::{
    gatt::{
        characteristic::Characteristic,
        properties::{AttributePermission, CharacteristicProperty},
        service::Service,
    },
    Peripheral, PeripheralImpl,
};
use uuid::Uuid;

use super::APP_SERVICE_UUID;

/// Starts advertising the application's Bluetooth Low Energy (BLE) service
/// to make it discoverable by nearby devices scanning for `APP_SERVICE_UUID`.
pub async fn advertise_app_service() -> Result<(), Box<dyn Error>> {
    println!("Initializing BLE peripheral for advertising...");

    // Parse our custom 128-bit service UUID from the constant defined in mod.rs
    let service_uuid = Uuid::parse_str(APP_SERVICE_UUID)?;

    // Define the GATT service we want to advertise.
    // GATT (Generic Attribute Profile) defines how two BLE devices transfer data.
    // A service is a collection of characteristics (which hold data).
    let service = Service {
        uuid: service_uuid,
        primary: true, // Primary services are discoverable over BLE
        characteristics: vec![
            // We can define custom characteristics here if we want to exchange data later.
            // For now, we only need a basic characteristic to comply with standard service structures.
            Characteristic {
                uuid: Uuid::new_v4(), // Random identifier for this characteristic
                properties: vec![CharacteristicProperty::Read, CharacteristicProperty::Write],
                permissions: vec![AttributePermission::Readable, AttributePermission::Writeable],
                ..Default::default()
            },
        ],
    };

    // Creating an event channel to listen for incoming BLE peripheral events.
    // Even if we don't handle events actively yet, ble_peripheral_rust requires this channel.
    let (sender_tx, mut receiver_rx) = mpsc::channel(256);

    // Instantiate the peripheral device (interacting with bluez on Linux under the hood)
    let mut peripheral = Peripheral::new(sender_tx).await?;

    // We can handle updates in a separate async task if we want to reply to read/write requests.
    // For now, we just consume the events silently.
    tokio::spawn(async move {
        while let Some(_event) = receiver_rx.recv().await {
            // Here we would process ReadRequest or WriteRequest events.
        }
    });

    println!("Ensuring Bluetooth adapter is powered on...");
    // Wait asynchronously until the OS Bluetooth adapter is fully powered on
    while !peripheral.is_powered().await? {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Register our service onto the local GATT server
    peripheral.add_service(&service).await?;
    println!("GATT Service added locally.");

    // Start advertising our existence to the world using the device name "CargoDrop"
    // and specifying our service_uuid so others can filter for it.
    peripheral.start_advertising("CargoDrop", &[service_uuid]).await?;
    println!("Now actively advertising as CargoDrop. Other devices can discover us.");

    // The advertising runs in the background. We loop indefinitely to keep the program alive.
    // You could replace this loop with a proper cancellation token or integration into a main loop.
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
