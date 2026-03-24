use airdrop::{Device, DeviceType};
use airdrop::network::transfer::{send_file_direct, receive_file_direct};
use std::net::IpAddr;
use std::env;
use std::path::Path;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> airdrop::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("mdns_sd", log::LevelFilter::Warn)
        .init();

    let args: Vec<String> = env::args().collect();

    println!("🎉 Starting Airdrop P2P Application");
    println!("Usage:");
    println!("  Mode 1 - Server (écoute les connexions):");
    println!("    cargo run -- server <device_name> <your_ip>");
    println!("    Ex: cargo run -- server PC-1 192.168.1.10");
    println!();
    println!("  Mode 2 - Client (se connecte à un PC):");
    println!("    cargo run -- send <device_name> <your_ip> <target_ip> <target_port> <file_path>");
    println!("    Ex: cargo run -- send PC-2 192.168.1.20 192.168.1.10 9999 /path/to/file.txt");
    println!();

    if args.len() < 3 {
        println!("❌ Paramètres insuffisants. Voir usage ci-dessus.");
        return Ok(());
    }

    let command = &args[1];
    let device_name = &args[2];
    let device_ip = args.get(3).map(|s| s.as_str());

    match command.as_str() {
        "server" => {
            if device_ip.is_none() {
                println!("❌ Syntaxe: cargo run -- server <device_name> <your_ip>");
                return Ok(());
            }

            let ip = device_ip.unwrap().parse::<IpAddr>()
                .map_err(|e| airdrop::AirdropError::Network(format!("IP invalide: {}", e)))?;
            let local_device = Device::new(
                device_name.clone(),
                ip,
                9999,
                DeviceType::Desktop,
            );

            println!("📱 Device: {} ({})", local_device.name, local_device.id);
            println!("🎧 Écoute sur {}:9999", ip);

            let listener = TcpListener::bind(format!("{}:9999", ip)).await?;
            println!("✅ Serveur démarré et en attente de connexions...");

            // Créer le répertoire de téléchargement s'il n'existe pas
            let download_dir = "./downloads";
            std::fs::create_dir_all(download_dir).ok();

            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutting down...");
                        break;
                    }
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let download_dir = download_dir.to_string();
                                tokio::spawn(async move {
                                    if let Err(e) = receive_file_direct(stream, &download_dir).await {
                                        println!("❌ Erreur lors de la réception: {}", e);
                                    }
                                });
                            }
                            Err(e) => println!("❌ Erreur connexion: {}", e),
                        }
                    }
                }
            }
        }

        "send" => {
            if args.len() < 6 || device_ip.is_none() {
                println!("❌ Syntaxe: cargo run -- send <device_name> <your_ip> <target_ip> <target_port> <file_path>");
                return Ok(());
            }

            let your_ip = device_ip.unwrap().parse::<IpAddr>()
                .map_err(|e| airdrop::AirdropError::Network(format!("IP invalide: {}", e)))?;
            let target_ip = args[4].clone();
            let target_port = args[5].parse::<u16>()
                .map_err(|e| airdrop::AirdropError::Network(format!("Port invalide: {}", e)))?;
            let file_path = &args[6];

            if !Path::new(file_path).exists() {
                println!("❌ Fichier introuvable: {}", file_path);
                return Ok(());
            }

            let local_device = Device::new(
                device_name.clone(),
                your_ip,
                9999,
                DeviceType::Desktop,
            );

            println!("📱 Device: {} ({})", local_device.name, local_device.id);
            println!("📤 Connexion à {}:{}", target_ip, target_port);
            println!("📁 Envoi de fichier: {}", file_path);

            send_file_direct(&target_ip, target_port, file_path).await?;
        }

        _ => {
            println!("❌ Commande inconnue: {}", command);
            println!("Commandes disponibles: server, send");
        }
    }

    Ok(())
}