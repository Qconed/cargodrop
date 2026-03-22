pub mod ble_rendezvous;
pub mod lan_rendezvous;

use std::error::Error;
use uuid::Uuid;
use btleplug::api::PeripheralProperties;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    pub ip: [u8; 4],
    pub port: u16,
    pub name: String,
}

// @TODO
// The RendezVousManager will be in charge of handling the multiple "P2P" discovery means, and will enable switching between implementations
// LAN is the preffered method, but DNS-SD is blocked over some networks
// => when LAN impossible, fall back to bluetooth detection.
pub enum RendezvousImpl {
    Lan,
    Bluetooth,
}

pub struct RendezvousManager;

impl RendezvousManager {
    // Uniquely generated UUID used to identify CargoDrop's BLE service.
    pub const APP_SERVICE_UUID: &str = "d59218d6-6b22-4a0b-9ba7-70e28148b488";
    
    // The current rendezvous implementation in use.
    pub const RENDEZVOUS_IMPL: RendezvousImpl = RendezvousImpl::Bluetooth;

    // A filter function that evaluates peripheral properties to check if it advertises a the app UUID
    fn is_matching_uuid(properties: &btleplug::api::PeripheralProperties) -> bool {
        let target_uuid = Uuid::parse_str(Self::APP_SERVICE_UUID).unwrap_or_default();
        properties.services.contains(&target_uuid)
    }

    // discover devices using relevant implementation (by order of preference)
    pub async fn discover_manage() -> Result<(), Box<dyn Error>> {
        match Self::RENDEZVOUS_IMPL {
            RendezvousImpl::Lan => lan_rendezvous::LanRendezvous::discover().await,
            RendezvousImpl::Bluetooth => ble_rendezvous::BleRendezvous::discover().await,
        }
    }

    // advertise presence to others using relevant implementation (by order of preference)
    pub async fn advertise_manage() -> Result<(), Box<dyn Error>> {
        match Self::RENDEZVOUS_IMPL {
            RendezvousImpl::Lan => lan_rendezvous::LanRendezvous::advertise().await,
            RendezvousImpl::Bluetooth => ble_rendezvous::BleRendezvous::advertise().await,
        }
    }
}

// traits defining a rendezvous engine (allowing for discovery and advertising)
pub trait RendezvousTrait {
    async fn discover() -> Result<(), Box<dyn Error>>;
    async fn advertise() -> Result<(), Box<dyn Error>>;
}
