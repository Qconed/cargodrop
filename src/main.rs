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

struct App;

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::discover_manage().await
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let app = App;

    cli.run(&app).await?;

    Ok(())
}