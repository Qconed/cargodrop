use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Device {
    pub id: Uuid,
    pub name: String,
    pub ip_address: IpAddr,
    pub port: u16,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Tablet,
}

impl Device {
    pub fn new(name: String, ip_address: IpAddr, port: u16, device_type: DeviceType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            ip_address,
            port,
            device_type,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip_address, self.port)
    }

    pub fn socket_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.ip_address, self.port)
            .parse()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device::new(
            "Test Device".to_string(),
            "192.168.1.100".parse().unwrap(),
            9999,
            DeviceType::Desktop,
        );
        assert_eq!(device.name, "Test Device");
    }
}