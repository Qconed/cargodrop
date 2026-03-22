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

const MAX_SCAN_RESPONSE_USERNAME_BYTES: usize = 20;

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

/// Builds the scan-response username payload while keeping UTF-8 boundaries intact.
fn build_scan_response_name(username: &str) -> String {
    let mut end = 0;
    for (idx, ch) in username.char_indices() {
        let next = idx + ch.len_utf8();
        if next > MAX_SCAN_RESPONSE_USERNAME_BYTES {
            break;
        }
        end = next;
    }

    if end == 0 {
        String::new()
    } else {
        username[..end].to_string()
    }
}

/// ble-peripheral-rust currently exposes a single local_name field, so we encode
/// both pieces in one deterministic value. The scanner parses the Base64 prefix first.
fn build_combined_local_name(device_name_payload: &str, scan_response_name: &str) -> String {
    if scan_response_name.is_empty() {
        return device_name_payload.to_string();
    }

    format!("{}|{}", device_name_payload, scan_response_name)
}

async fn start_advertising(
    peripheral: &mut Peripheral,
    service_uuid: Uuid,
    local_name_payload: &str,
) -> Result<(), Box<dyn Error>> {
    peripheral
        .start_advertising(local_name_payload, &[service_uuid])
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
    scan_response_name: &str,
    local_name_payload: &str,
    config: AdvertiseConfig,
) -> Result<(), Box<dyn Error>> {
    loop {
        log_heartbeat(ip, port, device_name_payload);
        println!(
            "  Username payload prepared: '{}' (truncated to {} bytes max)",
            scan_response_name, MAX_SCAN_RESPONSE_USERNAME_BYTES
        );
        println!(
            "  BLE local_name in use: '{}' (scan-response fallback encoding)",
            local_name_payload
        );
        sleep(config.heartbeat_interval).await;
    }
}

/// The main advertising loop that continuously advertises the custom network rendezvous payload.
pub async fn advertise_rendezvous(username: &str) -> Result<(), Box<dyn Error>> {
    let config = AdvertiseConfig::default();
    let service_uuid = Uuid::parse_str(crate::ble::APP_SERVICE_UUID)?;

    // 1. Prepare payload components
    let (ip, port, device_name_payload) = build_advertisement_payload();
    let scan_response_name = build_scan_response_name(username);
    let local_name_payload = build_combined_local_name(&device_name_payload, &scan_response_name);

    println!(
        "Prepared BLE username payload for scan response compatibility: '{}'",
        scan_response_name
    );

    // 2. Initialize BLE Peripheral & Service
    let mut peripheral = init_ble_peripheral(service_uuid).await?;
    wait_until_adapter_powered(&mut peripheral, config).await?;

    // 3. Start continuously advertising
    start_advertising(&mut peripheral, service_uuid, &local_name_payload).await?;

    // 4. Keep process alive and expose liveness heartbeat.
    run_advertise_heartbeat(
        ip,
        port,
        &device_name_payload,
        &scan_response_name,
        &local_name_payload,
        config,
    )
    .await
}
