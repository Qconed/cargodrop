use std::net::{IpAddr, UdpSocket};

/// Détecte l'adresse IP locale en créant une socket UDP fictive
/// vers Google DNS (8.8.8.8:80)
pub fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip())
}

/// Alternative: détecte l'IP en utilisant le hostname
/// (moins fiable que get_local_ip)
#[allow(dead_code)]
pub fn get_local_ip_by_hostname() -> Result<IpAddr, Box<dyn std::error::Error>> {
    use std::net::ToSocketAddrs;
    
    let hostname = hostname::get()?
        .into_string()
        .map_err(|_| "Impossible de convertir le hostname")?;
    
    let socket_addr = format!("{}:0", hostname)
        .to_socket_addrs()?
        .next()
        .ok_or("Impossible de résoudre le hostname")?;
    
    Ok(socket_addr.ip())
}
