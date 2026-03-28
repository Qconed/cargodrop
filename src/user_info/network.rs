use std::net::{IpAddr, UdpSocket};

/// Détecte l'adresse IP locale en créant une socket UDP fictive
/// vers Google DNS (8.8.8.8:80)
pub fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip())
}
