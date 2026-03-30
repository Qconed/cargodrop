use crate::network::file_transfer::PeerInfo;
use crate::rendezvous::Peer;
use crate::ui::interaction::{InteractionHandler, PeerEvent};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;

pub struct CliHandler;

impl InteractionHandler for CliHandler {
    // ===== Peer Discovery & Management =====
    fn display_peers_list(&self, peers: &HashMap<String, Peer>) {
        if peers.is_empty() {
            println!("{:<15} | {:<15} | {:<6}", "Username", "IP Address", "Port");
            println!("{:-<42}", "");
            println!("No peers discovered yet.");
            println!("{:-<42}\n", "");
            return;
        }

        println!("{:<15} | {:<15} | {:<6}", "Username", "IP Address", "Port");
        println!("{:-<42}", "");

        for peer in peers.values() {
            let ip_str = format!(
                "{}.{}.{}.{}",
                peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]
            );
            println!("{:<15} | {:<15} | {:<6}", peer.username, ip_str, peer.port);
        }
        println!("{:-<42}\n", "");
    }

    fn handle_peer_event(&self, event: PeerEvent) {
        match event {
            PeerEvent::NewPeer(_peer, time) => {
                println!("\n[{}] --- 📡 PEER DETECTED ---", time);
            }
            PeerEvent::PeerLost(_peer, time) => {
                println!("\n[{}] --- ❌ PEER DISCONNECTED ---", time);
            }
        }
    }

    fn select_peer(&self, peers: &[PeerInfo]) -> Option<PeerInfo> {
        if peers.is_empty() {
            println!("No peers available to select.");
            return None;
        }

        println!("\n--- Select a Peer to Send File ---");
        for (i, peer) in peers.iter().enumerate() {
            println!(
                "{}. {} ({}:{})",
                i + 1,
                peer.device_name,
                peer.ip,
                peer.port
            );
        }

        print!("\nEnter peer number (or 'q' to cancel): ");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input.to_lowercase() == "q" {
            return None;
        }

        if let Ok(choice) = input.parse::<usize>() {
            if choice > 0 && choice <= peers.len() {
                return Some(peers[choice - 1].clone());
            }
        }

        println!("Invalid selection.");
        None
    }

    // ===== File Selection & Transfer =====
    fn select_file_to_send(&self) -> Option<String> {
        print!("\nEnter file path to send (or 'q' to cancel): ");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input.to_lowercase() == "q" {
            return None;
        }

        if Path::new(input).exists() {
            return Some(input.to_string());
        }

        println!("File not found: {}", input);
        None
    }

    fn confirm_transfer(&self, sender: &str, filename: &str, size: u64) -> bool {
        println!();
        println!("Incoming file transfer request:");
        println!("  Sender: {}", sender);
        println!("  File: {}", filename);
        println!("  Size: {} bytes", size);
        println!("Accept transfer? [y/N]");
        print!("\n> ");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let choice = input.trim().to_lowercase();

        matches!(choice.as_str(), "y" | "yes")
    }

    // ===== Progress & Status Updates =====
    fn show_transfer_progress(&self, filename: &str, percent: f64, sent: u64, total: u64) {
        println!(
            "[{}] Transferring '{}': {:.0}% ({} / {} bytes)",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            filename,
            percent,
            sent,
            total
        );
    }

    fn show_app_status(&self, status: &str) {
        println!("\n=== {} ===", status);
    }

    fn show_receiver_listening(&self, port: u16) {
        println!("\n📡 Receiver listening on port {}", port);
        println!("Waiting for incoming file transfers...");
    }

    // ===== Messages (Error & Success) =====
    fn show_error(&self, message: &str) {
        eprintln!("\n❌ Error: {}", message);
    }

    fn show_success(&self, message: &str) {
        println!("\n✅ Success: {}", message);
    }

    fn show_info(&self, message: &str) {
        println!("\nℹ️  {}", message);
    }

    // ===== File Management =====
    fn show_received_files(&self, files: &[String]) {
        if files.is_empty() {
            println!("\nNo files received yet.");
            return;
        }

        println!("\n--- Received Files ---");
        for (i, file) in files.iter().enumerate() {
            println!("{}. {}", i + 1, file);
        }
        println!();
    }

    fn request_save_location(&self, filename: &str) -> Option<String> {
        print!("\nSave file as (default: {}): ", filename);
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input.is_empty() {
            return Some(filename.to_string());
        }

        Some(input.to_string())
    }
}
