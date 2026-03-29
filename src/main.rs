mod rendezvous;
mod cli;
mod use_cases;
mod network;
mod user_info;
//securite
mod security;
//securite

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::error::Error;
use user_info::UserInfo;
//securite
use tokio::sync::Mutex;
use std::sync::Arc;
use security::SecureSession;
//securite

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;

//securite
lazy_static::lazy_static! {
    pub static ref SECURE_SESSION: Arc<Mutex<Option<SecureSession>>> = Arc::new(Mutex::new(None));
}
//securite

struct App;

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        //securite
        let session = SecureSession::new("cargodrop-advertiser".to_string()).await?;
        *SECURE_SESSION.lock().await = Some(session);
        //securite
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        //securite
        let session = SecureSession::new("cargodrop-discoverer".to_string()).await?;
        *SECURE_SESSION.lock().await = Some(session);
        //securite
        rendezvous::RendezvousManager::discover_manage().await
    }

    async fn send(&self, ip: String, port: u16, file_path: String) -> Result<(), Box<dyn Error>> {
         //securite
        let mut session = SECURE_SESSION.lock().await;
        if session.is_none() {
            *session = Some(SecureSession::new("cargodrop-sender".to_string()).await?);
        }
        
        let session = session.as_mut().ok_or("Session non disponible")?;
        
        // Activation du chiffrement
        let (_, cle_chiffrement_vec) = session.initier_handshake()?;
        let mut cle_array = [0u8; 32];
        cle_array.copy_from_slice(&cle_chiffrement_vec);
        session.activer_chiffrement(&cle_array);
        
        println!(" Chiffrement activé avec: {}", hex::encode(&cle_array[..8]));
         //securite
        let peer = PeerInfo { // @todo: this should be discovered in future versions
            ip,
            port,
            device_name: "receiver".to_string(),
        };

        let client = TcpClient::new(peer, "DEFAULT_NAME".to_string());
        client.send_file(&file_path)
    }

    async fn receive(&self, port: u16) -> Result<(), Box<dyn Error>> {
        //securite
        // Vérifier et initialiser SANS garder le lock
        let needs_init = {
            let guard = SECURE_SESSION.lock().await;
            guard.is_none()
        }; 
        
        if needs_init {
            let session = SecureSession::new("cargodrop-receiver".to_string()).await?;
            let mut guard = SECURE_SESSION.lock().await;
            *guard = Some(session);
        } 
        
        // Initier le handshake
        let cle_chiffrement = {
            let guard = SECURE_SESSION.lock().await;
            let session = guard.as_ref().ok_or("Session non disponible")?;
            let (_, cle) = session.initier_handshake()?;
            cle
        }; 
        
        //  Activer le chiffrement
        {
            let mut guard = SECURE_SESSION.lock().await;
            let session = guard.as_mut().ok_or("Session non disponible")?;
            session.activer_chiffrement(&cle_chiffrement);
            println!("🔐 Chiffrement activé avec: {}", hex::encode(&cle_chiffrement[..8]));
        } 
        //securite
    
        let server = TcpServer::new(port, "DEFAULT_NAME".to_string());
        server.start()
    }

    // User info use cases
    async fn get_ip(&self) -> Result<(), Box<dyn Error>> {
        let user = UserInfo::load().await?;
        println!("Local IP: {}", user.local_ip);
        Ok(())
    }

    async fn get_name(&self) -> Result<(), Box<dyn Error>> {
        let user = UserInfo::load().await?;
        println!("Username: {}", user.username);
        Ok(())
    }

    async fn set_name(&self, name: String) -> Result<(), Box<dyn Error>> {
        let mut user = UserInfo::load().await?;
        user.set_username(name).await?;
        println!("Username changed to: {}", user.username);
        Ok(())
    }

    async fn set_name_default(&self) -> Result<(), Box<dyn Error>> {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "cargo-user".to_string());
        
        let mut user = UserInfo::load().await?;
        user.set_username(hostname.clone()).await?;
        println!("Username reset to hostname: {}", user.username);
        Ok(())
    }

    async fn get_port(&self) -> Result<(), Box<dyn Error>> {
        let user = UserInfo::load().await?;
        println!("Port: {}", user.port);
        Ok(())
    }

    async fn set_port(&self, port: u16) -> Result<(), Box<dyn Error>> {
        let mut user = UserInfo::load().await?;
        user.set_port(port).await?;
        println!("Port changed to: {}", port);
        Ok(())
    }

    async fn set_port_default(&self) -> Result<(), Box<dyn Error>> {
        const DEFAULT_PORT: u16 = 8080;
        let mut user = UserInfo::load().await?;
        user.set_port(DEFAULT_PORT).await?;
        println!("Port reset to default: {}", DEFAULT_PORT);
        Ok(())
    }

    async fn info(&self) -> Result<(), Box<dyn Error>> {
        let user = UserInfo::load().await?;
        user.display();
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let app = App;

    cli.run(&app).await?;

    Ok(())
}
