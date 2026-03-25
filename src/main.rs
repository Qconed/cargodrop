use std::env;
use std::error::Error;
use std::process;

mod network;

use network::file_transfer::PeerInfo;
use network::tcp_client::TcpClient;
use network::tcp_server::TcpServer;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "receive" => run_receive_mode(&args),
        "send" => run_send_mode(&args),
        _ => {
            print_usage(&args[0]);
            process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        print_usage(&args[0]);
        process::exit(1);
    }

    Ok(())
}

fn run_receive_mode(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut port: u16 = 5001;
    let mut index = 2;

    while index < args.len() {
        match args[index].as_str() {
            "--port" => {
                index += 1;
                if index >= args.len() {
                    return Err("Missing value for --port".into());
                }
                port = args[index].parse()?;
            }
            _ => {
                return Err(format!("Unknown argument for receive mode: {}", args[index]).into());
            }
        }
        index += 1;
    }

    let device_name = resolve_device_name();
    let server = TcpServer::new(port, device_name);
    server.start()
}

fn run_send_mode(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut ip: Option<String> = None;
    let mut file_path: Option<String> = None;
    let mut port: u16 = 5001;

    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--ip" => {
                index += 1;
                if index >= args.len() {
                    return Err("Missing value for --ip".into());
                }
                ip = Some(args[index].clone());
            }
            "--port" => {
                index += 1;
                if index >= args.len() {
                    return Err("Missing value for --port".into());
                }
                port = args[index].parse()?;
            }
            "--file" => {
                index += 1;
                if index >= args.len() {
                    return Err("Missing value for --file".into());
                }
                file_path = Some(args[index].clone());
            }
            _ => {
                return Err(format!("Unknown argument for send mode: {}", args[index]).into());
            }
        }
        index += 1;
    }

    let ip = ip.ok_or("Missing required argument: --ip")?;
    let file_path = file_path.ok_or("Missing required argument: --file")?;

    let peer = PeerInfo {
        ip,
        port,
        device_name: "receiver".to_string(),
    };

    let client = TcpClient::new(peer, resolve_device_name());
    client.send_file(&file_path)
}

fn resolve_device_name() -> String {
    env::var("HOSTNAME").unwrap_or_else(|_| "unknown-device".to_string())
}

fn print_usage(bin: &str) {
    println!("Usage:");
    println!("  {} receive --port <PORT>", bin);
    println!("  {} send --ip <IP> --port <PORT> --file <FILE_PATH>", bin);
    println!();
    println!("Subcommands:");
    println!("  receive   Run in receiver mode");
    println!("    --port <PORT>         Port to listen on (default: 5001)");
    println!("  send      Run in sender mode");
    println!("    --ip <IP>             Receiver's IP address");
    println!("    --port <PORT>         Receiver's port (default: 5001)");
    println!("    --file <FILE_PATH>    Path to the file to send");
}
