#[cfg(test)]
mod tests {
    use airdrop::{Device, DeviceType};
    use std::net::IpAddr;

    #[test]
    fn test_device_creation() {
        let device = Device::new(
            "Test Device".to_string(),
            "192.168.1.100".parse::<IpAddr>().unwrap(),
            9999,
            DeviceType::Desktop,
        );

        assert_eq!(device.name, "Test Device");
        assert_eq!(device.port, 9999);
    }

    #[test]
    fn test_device_address() {
        let device = Device::new(
            "Test".to_string(),
            "127.0.0.1".parse::<IpAddr>().unwrap(),
            9999,
            DeviceType::Laptop,
        );

        assert_eq!(device.address(), "127.0.0.1:9999");
    }
}