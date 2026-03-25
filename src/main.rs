mod rendezvous;
<<<<<<< HEAD
mod cli;
mod use_cases;
mod network;

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::error::Error;
use std::process;
=======
mod user_info;

use std::env;
use std::error::Error;
use user_info::UserInfo;
>>>>>>> b702590 (feat: add userinfo component)

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;

<<<<<<< HEAD
struct App;
=======
    // User management commands
    if args.len() > 1 {
        match args[1].as_str() {
            "getip" => {
                let user = UserInfo::load().await?;
                println!("Local IP: {}", user.local_ip);
                return Ok(());
            }
            "getname" => {
                let user = UserInfo::load().await?;
                println!("Username: {}", user.username);
                return Ok(());
            }
            "setname" => {
                if args.len() < 3 {
                    eprintln!("Usage: cargo run -- setname <username>");
                    return Err("Missing username argument".into());
                }
                let mut user = UserInfo::load().await?;
                user.set_username(args[2].clone()).await?;
                println!("Username changed to: {}", user.username);
                return Ok(());
            }
            "getport" => {
                let user = UserInfo::load().await?;
                println!("Port: {}", user.port);
                return Ok(());
            }
            "setport" => {
                if args.len() < 3 {
                    eprintln!("Usage: cargo run -- setport <port>");
                    return Err("Missing port argument".into());
                }
                let port: u16 = args[2].parse()?;
                let mut user = UserInfo::load().await?;
                user.set_port(port).await?;
                println!("Port changed to: {}", port);
                return Ok(());
            }
            "info" => {
                let user = UserInfo::load().await?;
                user.display();
                return Ok(());
            }
            "advertise" => {
                println!("Starting CargoDrop in Advertiser Mode...");
                rendezvous::RendezvousManager::advertise_manage().await?;
                return Ok(());
            }
            "discover" => {
                println!("Starting CargoDrop in Discovery Mode...");
                rendezvous::RendezvousManager::discover_manage().await?;
                return Ok(());
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                print_usage();
                return Err("Invalid command".into());
            }
        }
    } else {
        // Default behavior: both modes concurrently
        println!("Starting CargoDrop in Dual Mode (Advertise + Discover)...");
        println!("Attempting to run both concurrently for demonstration...");
>>>>>>> b702590 (feat: add userinfo component)

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::discover_manage().await
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
}
<<<<<<< HEAD
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let app = App;

    cli.run(&app).await?;

    Ok(())
}
=======

fn print_usage() {
    println!("\n=== CargoDrop - Usage ===");
    println!("\nUser Management:");
    println!("  cargo run -- getip              Get local IP address");
    println!("  cargo run -- getname            Get current username");
    println!("  cargo run -- setname <name>     Set username (max 9 chars)");
    println!("  cargo run -- getport            Get configured port");
    println!("  cargo run -- setport <port>     Set HTTP transfer port");
    println!("  cargo run -- info               Display all user info");
    println!("\nMain Modes:");
    println!("  cargo run -- advertise          Start in advertiser mode");
    println!("  cargo run -- discover           Start in discover mode");
    println!("  cargo run                       Run both modes concurrently");
    println!("========================\n");
}
>>>>>>> b702590 (feat: add userinfo component)
