mod rendezvous;
mod cli;
mod use_cases;
mod network;
mod user_info;

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::env;
use std::error::Error;
use user_info::UserInfo;

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;

struct App;

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

/// Handle user info commands (getip, getname, setname, getport, setport, info)
async fn handle_user_info_command(args: &[String]) -> Result<bool, Box<dyn Error>> {
    if args.is_empty() {
        return Ok(false);
    }

    match args[0].as_str() {
        "getip" => {
            let user = UserInfo::load().await?;
            println!("Local IP: {}", user.local_ip);
            Ok(true)
        }
        "getname" => {
            let user = UserInfo::load().await?;
            println!("Username: {}", user.username);
            Ok(true)
        }
        "setname" => {
            if args.len() < 2 {
                eprintln!("Usage: cargo run -- setname <username>");
                return Err("Missing username argument".into());
            }
            let mut user = UserInfo::load().await?;
            user.set_username(args[1].clone()).await?;
            println!("Username changed to: {}", user.username);
            Ok(true)
        }
        "getport" => {
            let user = UserInfo::load().await?;
            println!("Port: {}", user.port);
            Ok(true)
        }
        "setport" => {
            if args.len() < 2 {
                eprintln!("Usage: cargo run -- setport <port>");
                return Err("Missing port argument".into());
            }
            let port: u16 = args[1].parse()?;
            let mut user = UserInfo::load().await?;
            user.set_port(port).await?;
            println!("Port changed to: {}", port);
            Ok(true)
        }
        "info" => {
            let user = UserInfo::load().await?;
            user.display();
            Ok(true)
        }
        _ => Ok(false), // Not a user info command
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if it's a user info command (before clap parsing)
    if args.len() > 1 {
        let user_info_args = &args[1..];
        if handle_user_info_command(user_info_args).await? {
            return Ok(());
        }
    }

    // Otherwise, use clap CLI for advertise/discover/send/receive commands
    let cli = Cli::parse();
    let app = App;

    cli.run(&app).await?;

    Ok(())
}
