pub mod discovery;
pub mod server;
pub mod client;
pub mod crypto;
pub mod protocol;
pub mod cert;
pub mod transfer;

pub use discovery::DiscoveryService;
pub use server::Server;
pub use client::{Client, ClientConnection};
pub use transfer::FileTransfer;
pub use crypto::CryptoManager;
pub use protocol::ProtocolHandler;

use crate::models::Device;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NetworkManager {
    pub local_device: Device,
    pub discovery: Arc<DiscoveryService>,
    pub server: Arc<tokio::sync::Mutex<Server>>,
    pub discovered_devices: Arc<RwLock<Vec<Device>>>,
}

impl NetworkManager {
    pub async fn new(local_device: Device) -> crate::error::Result<Self> {
        let discovery = Arc::new(DiscoveryService::new(local_device.clone()).await?);
        let server = Arc::new(tokio::sync::Mutex::new(Server::new(local_device.port).await?));

        Ok(Self {
            local_device,
            discovery,
            server,
            discovered_devices: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start(&self) -> crate::error::Result<()> {
        println!("🚀 Starting NetworkManager...");

        // Démarrer découverte
        self.discovery.start().await?;

        // Démarrer serveur QUIC
        let mut server = self.server.lock().await;
        server.start().await?;

        Ok(())
    }
}