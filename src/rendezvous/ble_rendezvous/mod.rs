use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub mod advertise;
pub mod discover;

pub(crate) const APP_SERVICE_UUID: &str = "d59218d6-6b22-4a0b-9ba7-70e28148b488";

// Constants to define the structure of the BLE advertisement payload (stored in the "device name" field).
pub(crate) const NETWORK_INFO_BYTES: usize = 6;
pub(crate) const USERNAME_LEN_BYTES: usize = 1;
pub(crate) const USERNAME_LEN_OFFSET: usize = NETWORK_INFO_BYTES;
pub(crate) const USERNAME_OFFSET: usize = NETWORK_INFO_BYTES + USERNAME_LEN_BYTES;
pub(crate) const MAX_RAW_PAYLOAD_BYTES: usize = 16;

pub struct BleRendezvous {}

impl RendezvousTrait for BleRendezvous {
    async fn advertise() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting advertisement...");
        advertise::advertise_rendezvous().await
    }
    
    async fn discover(peers: crate::rendezvous::PeerMap) -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting discovery...");
        let service = discover::BleDiscoveryService::new(peers);
        service.run().await
    }
}
