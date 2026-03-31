use clap::{Parser, Subcommand};
use crate::use_cases::AppUseCases;

/// A command line interface for CargoDrop
#[derive(Parser)]
#[command(name = "cargodrop")]
#[command(about = "A command line interface for CargoDrop", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[command(rename_all = "lowercase")]
pub enum Commands { // Note : DON'T DELETE THE /// COMMENTS: they are the documentation of the commands !!
    /// Launch the GUI application
    Gui,
    /// Start CargoDrop in Advertiser Mode
    Advertise,
    /// Start CargoDrop in Discovery Mode
    Discover,
    /// Send a file
    Send {
        /// Receiver's IP address
        #[arg(short, long)]
        ip: String,
        /// Receiver's port
        #[arg(short, long)]
        port: Option<u16>,
        /// Path to the file to send
        #[arg(short, long)]
        file: String,
    },
    /// Receive a file
    Receive {
        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Send a file interactively (trigger a discovery, and choose to whom to send)
    Sendinter {
        /// Path to the file to send
        #[arg(short, long)]
        file: String,
    },
    /// Get local IP address
    GetIp,
    /// Get current username
    GetName,
    /// Set username (max 14 characters)
    #[command(group(clap::ArgGroup::new("set-name-action").required(true).args(["name", "default"])))]
    SetName {
        /// The new username
        #[arg(value_name = "NAME", conflicts_with = "default")]
        name: Option<String>,
        /// Reset to system hostname
        #[arg(long)]
        default: bool,
    },
    /// Get configured HTTP transfer port
    GetPort,
    /// Set HTTP transfer port
    #[command(group(clap::ArgGroup::new("set-port-action").required(true).args(["port", "default"])))]
    SetPort {
        /// The new port number
        #[arg(value_name = "PORT", conflicts_with = "default")]
        port: Option<u16>,
        /// Reset to default port (8080)
        #[arg(long)]
        default: bool,
    },
    /// Display all user configuration info
    Info,
}

/// The cli component uses dependency inversion of the App use cases to run
impl Cli {
    pub async fn run<T: AppUseCases>(self, use_cases: &T) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::Gui => {
                return Err("GUI mode should be handled in main".into());
            }
            Commands::Advertise => {
                println!("Starting CargoDrop in Advertiser Mode...");
                use_cases.advertise().await?;
            }
            Commands::Discover => {
                println!("Starting CargoDrop in Discovery Mode...");
                use_cases.discover().await?;
            }
            Commands::Send { ip, port, file } => {
                println!("Starting CargoDrop in Send Mode...");
                use_cases.send(ip, port, file).await?;
            }
            Commands::Receive { port } => {
                println!("Starting CargoDrop in Receive Mode (with background advertisement)...");
                use_cases.advertise_and_receive(port).await?;
            }
            Commands::Sendinter { file } => {
                println!("Starting CargoDrop in Interactive Send Mode...");
                println!("Running discovery for 20 seconds...");
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    use_cases.discover()
                ).await;
                use_cases.interactive_send(file).await?;
            }
            Commands::GetIp => {
                use_cases.get_ip().await?;
            }
            Commands::GetName => {
                use_cases.get_name().await?;
            }
            Commands::SetName { name, default } => {
                if default {
                    use_cases.set_name_default().await?;
                } else {
                    use_cases.set_name(name.unwrap()).await?;
                }
            }
            Commands::GetPort => {
                use_cases.get_port().await?;
            }
            Commands::SetPort { port, default } => {
                if default {
                    use_cases.set_port_default().await?;
                } else {
                    use_cases.set_port(port.unwrap()).await?;
                }
            }
            Commands::Info => {
                use_cases.info().await?;
            }
        }
        Ok(())
    }
}
