use std::error::Error;

use crate::rendezvous::RendezvousTrait;

pub struct LanRendezvous {}

impl RendezvousTrait for LanRendezvous {
    async fn advertise() -> Result<(), Box<dyn Error>> {
        println!("LAN Rendezvous: Starting advertisement ...");
        Ok(())
    }
    
    async fn discover(_peers: crate::rendezvous::PeerMap) -> Result<(), Box<dyn Error>> {
        println!("LAN Rendezvous: Starting discovery ...");
        Ok(())
    }
}
