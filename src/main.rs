mod ble;

use std::error::Error;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Collect command line arguments to decide which mode to run in.
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "advertise" {
        println!("Starting CargoDrop in Advertiser Mode...");
        // Call our new decentralized advertising logic
        ble::advertise::advertise_app_service().await?;
    } else if args.len() > 1 && args[1] == "discover" {
        println!("Starting CargoDrop in Discovery Mode...");
        // Call our new decentralized discovery logic
        ble::discover::discover_app_peripherals().await?;
    } else {
        // Provide usage instructions if no argument is given
        println!("Usage: cargo run -- [advertise|discover]");
        println!("Attempting to run both concurrently for demonstration...");

        // tokio::spawn allows us to spin up an asynchronous task in the background.
        // NOTE: Running both advertising and discovery simultaneously on the same 
        // physical Bluetooth adapter might fail or be completely ignored depending 
        // on the specific hardware controller's capabilities.
        let advertiser = tokio::spawn(async {
            if let Err(e) = ble::advertise::advertise_app_service().await {
                eprintln!("Advertiser error: {}", e);
            }
        });

        // Give the advertiser a second to initialize properly before we start scanning
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let discoverer = tokio::spawn(async {
            if let Err(e) = ble::discover::discover_app_peripherals().await {
                eprintln!("Discoverer error: {}", e);
            }
        });

        // tokio::join! waits for multiple futures (our spawned tasks) to complete.
        let _ = tokio::join!(advertiser, discoverer);
    }

    Ok(())
}
