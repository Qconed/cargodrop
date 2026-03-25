mod rendezvous;
mod cli;
mod use_cases;

use use_cases::AppUseCases;
use cli::Cli;
use clap::Parser;
use std::error::Error;

struct App;

/// Use cases dependency passed to the cli component to run it
impl AppUseCases for App {
    async fn advertise(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::advertise_manage().await
    }

    async fn discover(&self) -> Result<(), Box<dyn Error>> {
        rendezvous::RendezvousManager::discover_manage().await
    }

    async fn send(&self) -> Result<(), Box<dyn Error>> {
        println!("Send logic not implemented yet.");
        Ok(())
    }

    async fn receive(&self) -> Result<(), Box<dyn Error>> {
        println!("Receive logic not implemented yet.");
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
