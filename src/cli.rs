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
pub enum Commands {
    /// Start CargoDrop in Advertiser Mode
    Advertise,
    /// Start CargoDrop in Discovery Mode
    Discover,
    /// Send a file (mockup)
    Send,
    /// Receive a file (mockup)
    Receive,
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
            Commands::Send => {
                println!("Starting CargoDrop in Send Mode (Mockup)...");
                use_cases.send().await?;
            }
            Commands::Receive => {
                println!("Starting CargoDrop in Receive Mode (Mockup)...");
                use_cases.receive().await?;
            }
        }
        Ok(())
    }
}
