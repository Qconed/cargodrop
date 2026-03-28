use std::net::{IpAddr, UdpSocket};

/// Détecte l'adresse IP locale en créant une socket UDP fictive
/// vers Google DNS (8.8.8.8:80)
/// 
/// En cas d'erreur (pas de connexion WiFi), retourne localhost (127.0.0.1)
pub fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
    match try_get_local_ip() {
        Ok(ip) => Ok(ip),
        Err(e) => {
            eprintln!("Avertissement: Impossible de détecter l'IP locale: {}. Utilisation de localhost.", e);
            Ok(IpAddr::from([127, 0, 0, 1]))
        }
    }
}

/// Tente de détecter l'adresse IP locale via UDP
fn try_get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip())
}
