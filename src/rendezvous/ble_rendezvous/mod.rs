use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub mod advertise;
pub mod discover;

pub struct BleRendezvous {}

impl BleRendezvous {
    pub async fn advertise_with_username(username: &str) -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting advertisement...");
        advertise::advertise_rendezvous(username).await
    }
}

impl RendezvousTrait for BleRendezvous {
    async fn advertise() -> Result<(), Box<dyn Error>> {
        Self::advertise_with_username("CargoDrop").await
    }
    
    async fn discover() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting discovery...");
        discover::discover_rendezvous().await
    }
}
