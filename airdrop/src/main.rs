use airdrop::{Device, DeviceType, NetworkManager};
use std::net::IpAddr;

#[tokio::main]
async fn main() -> airdrop::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("mdns_sd", log::LevelFilter::Warn)
        .init();

    // Créer l'appareil local
    let local_device = Device::new(
        "My-Device".to_string(),
        "127.0.0.1".parse::<IpAddr>().unwrap(),
        9999,
        DeviceType::Desktop,
    );

    println!("🎉 Starting Airdrop P2P Application");
    println!("📱 Device: {} ({})", local_device.name, local_device.id);

    // Créer et démarrer le gestionnaire réseau
    let network = NetworkManager::new(local_device).await?;
    network.start().await?;

    // Garder l'application active
    println!("⏳ Press Ctrl+C to exit...");
    tokio::signal::ctrl_c().await?;
    println!("👋 Shutting down...");

    Ok(())
}