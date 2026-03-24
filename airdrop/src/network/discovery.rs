use crate::error::{AirdropError, Result};
use crate::models::Device;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DiscoveryService {
    local_device: Device,
    daemon: Arc<RwLock<Option<ServiceDaemon>>>,
    discovered: Arc<RwLock<Vec<Device>>>,
}

impl DiscoveryService {
    pub async fn new(local_device: Device) -> Result<Self> {
        Ok(Self {
            local_device,
            daemon: Arc::new(RwLock::new(None)),
            discovered: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| AirdropError::Discovery(e.to_string()))?;

        let ip_addr = match self.local_device.ip_address {
            std::net::IpAddr::V4(addr) => addr,
            std::net::IpAddr::V6(_) => {
                return Err(AirdropError::Discovery("IPv6 not supported for mDNS".to_string()))
            }
        };

        let service_info = ServiceInfo::new(
            "_airdrop._tcp.local.",  
            &self.local_device.name,
            "local",
            ip_addr,
            self.local_device.port,
            HashMap::new(),
        )
        .map_err(|e| AirdropError::Discovery(e.to_string()))?;

        daemon
            .register(service_info)
            .map_err(|e| AirdropError::Discovery(e.to_string()))?;

        *self.daemon.write().await = Some(daemon);
        println!("✅ mDNS Discovery started: {}", self.local_device.name);

        Ok(())
    }

    pub async fn browse(&self) -> Result<Vec<Device>> {
        Ok(self.discovered.read().await.clone())
    }

    pub async fn add_discovered_device(&self, device: Device) {
        self.discovered.write().await.push(device);
    }
}

impl Drop for DiscoveryService {
    fn drop(&mut self) {
        println!("🔌 Stopping mDNS Discovery");
    }
}