mod rendezvous;
mod cli;
mod use_cases;
mod network;
mod user_info;
mod ui;

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::error::Error;
use user_info::UserInfo;

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;
use ui::egui_app::{CargodropApp, GuiAppState};

struct App;

impl Clone for App {
    fn clone(&self) -> Self {
        App
    }
    
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        // Create empty peer map for now
        let peers: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, rendezvous::Peer>>> = 
            std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
        
        // Use CLI handler temporarily
        let handler: std::sync::Arc<dyn crate::ui::interaction::InteractionHandler> = 
            std::sync::Arc::new(ui::cli_handler::CliHandler {});
        
        rendezvous::RendezvousManager::discover_manage(peers, handler).await
    }

    async fn interactive_send(&self, file_path: String) -> Result<(), Box<dyn Error>> {
        println!("Interactive send initiated for: {}", file_path);
        Ok(())
    }

    async fn send(&self, ip: String, port: u16, file_path: String) -> Result<(), Box<dyn Error>> {
        let peer = PeerInfo { // @todo: this should be discovered in future versions
            ip,
            port,
            device_name: "receiver".to_string(),
        };

        let client = TcpClient::new(peer, "DEFAULT_NAME".to_string());
        client.send_file(&file_path)
    }

    async fn receive(&self, port: u16) -> Result<(), Box<dyn Error>> {
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

    if cli.gui {
        // Launch GUI mode with egui/eframe
        run_gui_mode().await?;
    } else {
        // Launch CLI mode
        cli.run(&app).await?;
    }

    Ok(())
}

/// Run the GUI application with egui/eframe
async fn run_gui_mode() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    // Run the egui application
    eframe::run_native(
        "CargoDrop",
        options,
        Box::new(|_cc| Box::new(CargodropApp::default())),
    )
    .map_err(|e| format!("egui error: {}", e).into())
}

