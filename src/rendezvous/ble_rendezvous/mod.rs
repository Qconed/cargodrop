use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub mod advertise;

pub struct BleRendezvous {}

impl RendezvousTrait for BleRendezvous {
    async fn advertise() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting advertisement...");
        advertise::advertise_rendezvous().await
    }
    
    async fn discover() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting discovery...");
        Ok(())
    }
}
