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
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
struct AdvertiseConfig {
    adapter_power_poll: Duration,
    adapter_power_max_wait: Duration,
    heartbeat_interval: Duration,
}

impl Default for AdvertiseConfig {
    fn default() -> Self {
        Self {
            adapter_power_poll: Duration::from_millis(50),
            adapter_power_max_wait: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(5),
        }
    }
}

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

async fn wait_until_adapter_powered(
    peripheral: &mut Peripheral,
    config: AdvertiseConfig,
) -> Result<(), Box<dyn Error>> {
    println!("Ensuring Bluetooth adapter is powered on...");

    let start = tokio::time::Instant::now();
    while !peripheral.is_powered().await? {
        if start.elapsed() >= config.adapter_power_max_wait {
            return Err("Timed out waiting for Bluetooth adapter to be powered on".into());
        }
        sleep(config.adapter_power_poll).await;
    }

    Ok(())
}

/// Discovers the local network config (@todo: mocked here, should be injected or detected dynamically).
fn get_local_network_info() -> ([u8; 4], u16) {
    // Return sample local IPv4 and HTTP Port
    ([192, 168, 1, 100], 8080)
}

fn build_advertisement_payload() -> ([u8; 4], u16, String) {
    let (ip, port) = get_local_network_info();
    let device_name_payload = encode_network_info_to_name(ip, port);

    println!(
        "Encoded Network payload (IP: {:?}, Port: {}) -> Name: '{}'",
        ip, port, device_name_payload
    );

    (ip, port, device_name_payload)
}

async fn start_advertising(
    peripheral: &mut Peripheral,
    service_uuid: Uuid,
    device_name_payload: &str,
) -> Result<(), Box<dyn Error>> {
    peripheral
        .start_advertising(device_name_payload, &[service_uuid])
        .await?;
    println!("Now actively advertising custom network rendezvous info...");
    Ok(())
}

fn log_heartbeat(ip: [u8; 4], port: u16, device_name_payload: &str) {
    let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
    println!("[{}] --- ADVERTISING ACTIVE ---", time_str);
    println!(
        "  Broadcasting IP: {:?}, Port: {} inside Base64 Name: '{}'",
        ip, port, device_name_payload
    );
}

async fn run_advertise_heartbeat(
    ip: [u8; 4],
    port: u16,
    device_name_payload: &str,
    config: AdvertiseConfig,
) -> Result<(), Box<dyn Error>> {
    loop {
        log_heartbeat(ip, port, device_name_payload);
        sleep(config.heartbeat_interval).await;
    }
}

/// The main advertising loop that continuously advertises the custom network rendezvous payload.
pub async fn advertise_rendezvous() -> Result<(), Box<dyn Error>> {
    let config = AdvertiseConfig::default();
    let service_uuid = Uuid::parse_str(crate::ble::APP_SERVICE_UUID)?;

    // 1. Prepare payload components
    let (ip, port, device_name_payload) = build_advertisement_payload();

    // 2. Initialize BLE Peripheral & Service
    let mut peripheral = init_ble_peripheral(service_uuid).await?;
    wait_until_adapter_powered(&mut peripheral, config).await?;

    // 3. Start continuously advertising
    start_advertising(&mut peripheral, service_uuid, &device_name_payload).await?;

    // 4. Keep process alive and expose liveness heartbeat.
    run_advertise_heartbeat(ip, port, &device_name_payload, config).await
}
