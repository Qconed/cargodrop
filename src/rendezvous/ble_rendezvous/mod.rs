use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub struct BleRendezvous {}

impl RendezvousTrait for BleRendezvous {
    fn advertise() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting advertisement...");
        Ok(())
    }
    
    fn discover() -> Result<(), Box<dyn Error>> {
        println!("BLE Rendezvous: Starting discovery...");
        Ok(())
    }
}
