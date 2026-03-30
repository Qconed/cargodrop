mod rendezvous;
mod cli;
mod use_cases;
mod network;
mod ui;
mod user_info;

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::error::Error;
use user_info::UserInfo;

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use ui::interaction::InteractionHandler;
use ui::cli_handler::CliHandler;
use ui::gui_handler::GuiHandler;
use ui::egui_app::{CargodropApp, GuiAppState};

struct App {
    peers: rendezvous::PeerMap,
    handler: Arc<dyn InteractionHandler>,
}

impl App {
    fn new(handler: Arc<dyn InteractionHandler>) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            handler,
        }
    }
}

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        let peers_clone = self.peers.clone();
        let handler_clone = self.handler.clone();
        
        rendezvous::RendezvousManager::discover_manage(peers_clone, handler_clone).await
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

    async fn interactive_send(&self, file_path: String) -> Result<(), Box<dyn Error>> {
        let peer_infos: Vec<PeerInfo> = {
            let peers_guard = self.peers.read().await;
            peers_guard.values().map(|p| PeerInfo {
                ip: format!("{}.{}.{}.{}", p.ip[0], p.ip[1], p.ip[2], p.ip[3]),
                port: p.port,
                device_name: p.username.clone(),
            }).collect()
        };

        // once peer have been searched, called the UI handler to select a peer
        // behavior will be different if handler = CLI, or GUI, but it will still produce the same result
        if let Some(selected_peer) = self.handler.select_peer(&peer_infos) {
            self.send(selected_peer.ip, selected_peer.port, file_path).await
        } else {
            println!("No peer selected or operation cancelled.");
            Ok(())
        }
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

    // Check if GUI mode is requested
    if cli.gui {
        // Launch GUI mode with egui
        run_gui_mode().await?;
    } else {
        // Launch CLI mode
        let handler = Arc::new(CliHandler);
        let app = App::new(handler);
        cli.run(&app).await?;
    }

    Ok(())
}

/// Run the GUI application with egui/eframe
async fn run_gui_mode() -> Result<(), Box<dyn Error>> {
    let state = GuiAppState::default();
    let _handler = Arc::new(GuiHandler::new(state.clone()));
    let _app = App::new(_handler);

    // Options for eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    let state_clone = state.clone();
    // Run the egui application
    let _ = eframe::run_native(
        "CargoDrop 📦",
        options,
        Box::new(move |_cc| Box::new(CargodropApp {
            state: state_clone.clone(),
        })),
    );

    Ok(())
}
