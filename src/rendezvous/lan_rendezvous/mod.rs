use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub struct LanRendezvous {}

impl RendezvousTrait for LanRendezvous {
    fn advertise() -> Result<(), Box<dyn Error>> {
        println!("LAN Rendezvous: Starting advertisement ...");
        Ok(())
    }
    
    fn discover() -> Result<(), Box<dyn Error>> {
        println!("LAN Rendezvous: Starting discovery ...");
        Ok(())
    }
}
