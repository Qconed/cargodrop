mod rendezvous;
mod cli;
mod use_cases;
mod network;

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::error::Error;

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct App {
    peers: rendezvous::PeerMap,
}

impl App {
    fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        let peers_clone = self.peers.clone();
        
        // Spawn the monitoring task
        let monitor_peers_list = self.peers.clone();
        tokio::spawn(async move {
            monitor_peers(monitor_peers_list).await;
        });

        rendezvous::RendezvousManager::discover_manage(peers_clone).await
    }

    async fn send(&self, ip: String, port: u16, file_path: String) -> Result<(), Box<dyn Error>> {
        let peer = PeerInfo { // @todo: this should be discovered in future versions
            ip,
            port,
            device_name: "receiver".to_string(),
        };

        let client = TcpClient::new(peer, "DEFAULT_NAME".to_string());
        client.send_file(&file_path)
    }

    async fn receive(&self, port: u16) -> Result<(), Box<dyn Error>> {
        let server = TcpServer::new(port, "DEFAULT_NAME".to_string());
        server.start()
    }
}

/// display the detected peers list whenever a detection/disconnection happens
async fn monitor_peers(peers: rendezvous::PeerMap) {
    let mut last_size = 0;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let p = peers.read().await;
        let current_size = p.len();
        
        if current_size != last_size {
            let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
            
            if current_size > last_size {
                println!("\n[{}] --- 📡 PEER DETECTED (Total: {}) ---", time_str, current_size);
            } else if current_size < last_size {
                println!("\n[{}] --- ❌ PEER DISCONNECTED (Total: {}) ---", time_str, current_size);
            }

            // Display the current snapshot of all active peers
            println!("{:<15} | {:<15} | {:<6}", "Username", "IP Address", "Port");
            println!("{:-<42}", ""); // Separator line
            
            for peer in p.values() {
                let ip_str = format!("{}.{}.{}.{}", peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
                println!("{:<15} | {:<15} | {:<6}", peer.username, ip_str, peer.port);
            }
            println!("{:-<42}\n", "");
            
            last_size = current_size;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let app = App::new();

    cli.run(&app).await?;

    Ok(())
}