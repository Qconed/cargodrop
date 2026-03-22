use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ble_peripheral_rust::{
    Peripheral, PeripheralImpl,
    gatt::{
        characteristic::Characteristic,
        properties::{AttributePermission, CharacteristicProperty},
        service::Service,
    },
};
use std::error::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Encodes the IPv4 and Port into a 6-byte array, then encodes it into Base64
/// to be compactly used as the BLE device name of the advertising packet.
fn encode_network_info_to_name(ipv4: [u8; 4], port: u16) -> String {
    let mut bytes = [0u8; 6];
    bytes[0..4].copy_from_slice(&ipv4);
    bytes[4..6].copy_from_slice(&port.to_be_bytes()); // Network byte order
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Initializes the BLE peripheral and the GATT service, returning the configured Peripheral.
async fn init_ble_peripheral(service_uuid: Uuid) -> Result<Peripheral, Box<dyn Error>> {
    let (sender_tx, mut receiver_rx) = mpsc::channel(256);
    let mut peripheral = Peripheral::new(sender_tx).await?;

    // Consume the channel events in a background task so it doesn't block.
    // For pure broadcasting, we don't care about interacting with clients via GATT requests.
    tokio::spawn(async move {
        while let Some(_event) = receiver_rx.recv().await {
            // Just dropping the events
        }
    });

    println!("Ensuring Bluetooth adapter is powered on...");
    while !peripheral.is_powered().await? {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Create a dummy service just to hold the primary app UUID.
    let service = Service {
        uuid: service_uuid,
        primary: true,
        characteristics: vec![Characteristic {
            uuid: Uuid::new_v4(),
            properties: vec![CharacteristicProperty::Read],
            permissions: vec![AttributePermission::Readable],
            ..Default::default()
        }],
    };

    peripheral.add_service(&service).await?;
    println!("GATT Service added locally.");

    Ok(peripheral)
}

/// Discovers the local network config (@todo: mocked here, should be injected or detected dynamically).
fn get_local_network_info() -> ([u8; 4], u16) {
    // Return sample local IPv4 and HTTP Port
    ([192, 168, 1, 100], 8080)
}

/// The main advertising loop that continuously advertises the custom network rendezvous payload.
pub async fn advertise_rendezvous() -> Result<(), Box<dyn Error>> {
    let service_uuid = Uuid::parse_str(crate::ble::APP_SERVICE_UUID)?;

    // 1. Prepare payload components
    let (ip, port) = get_local_network_info();

    // 2. Encode to Base64 to fit the 31-byte limit
    let device_name_payload = encode_network_info_to_name(ip, port);
    println!(
        "Encoded Network payload (IP: {:?}, Port: {}) -> Name: '{}'",
        ip, port, device_name_payload
    );

    // 3. Initialize BLE Peripheral & Service
    let mut peripheral = init_ble_peripheral(service_uuid).await?;

    // 4. Start continuously advertising
    peripheral
        .start_advertising(&device_name_payload, &[service_uuid])
        .await?;
    println!("Now actively advertising custom network rendezvous info...");

    // 5. Advertising is handled continuously by the OS in the background (typically multiple times per second).
    // We loop here to keep the application alive, and print a heartbeat to the CLI to show it's still running.
    loop {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("[{}] --- ADVERTISING ACTIVE ---", time_str);
        println!("  Broadcasting IP: {:?}, Port: {} inside Base64 Name: '{}'", ip, port, device_name_payload);
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
