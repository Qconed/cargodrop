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
pub enum Commands { // Note : DON'T DELETE THE /// COMMENTS: they are the documentation of the commands !!
    /// Start CargoDrop in Advertiser Mode
    Advertise,
    /// Start CargoDrop in Discovery Mode
    Discover,
    /// Send a file
    Send {
        /// Receiver's IP address
        #[arg(short, long)]
        ip: String,
        /// Receiver's port (default: 5001)
        #[arg(short, long, default_value_t = 5001)]
        port: u16,
        /// Path to the file to send
        #[arg(short, long)]
        file: String,
    },
    /// Receive a file
    Receive {
        /// Port to listen on (default: 5001)
        #[arg(short, long, default_value_t = 5001)]
        port: u16,
    },
    /// Send a file interactively (trigger a discovery, and choose to whom to send)
    Sendinter {
        /// Path to the file to send
        #[arg(short, long)]
        file: String,
    },
}

/// The cli component uses dependency inversion of the App use cases to run
impl Cli {
    pub async fn run<T: AppUseCases>(self, use_cases: &T) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
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
                println!("Starting CargoDrop in Receive Mode...");
                use_cases.receive(port).await?;
            }
            Commands::Sendinter { file } => {
                // here in the cli for the sendinter, discovery is done as a one off thing
                // but for the GUI, this discovery will probably be a background task
                println!("Starting CargoDrop in Interactive Send Mode...");
                println!("Running discovery for 20 seconds...");
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    use_cases.discover()
                ).await;
                
                use_cases.interactive_send(file).await?;
            }
        }
        Ok(())
    }
}
