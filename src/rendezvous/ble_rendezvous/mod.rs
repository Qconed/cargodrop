use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub struct BleRendezvous {}

impl RendezvousTrait for BleRendezvous {
    async fn advertise() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting advertisement...");
        Ok(())
    }
    
    async fn discover() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting discovery...");
        Ok(())
    }
}
