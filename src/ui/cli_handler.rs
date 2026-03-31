use crate::network::file_transfer::{FileTransfer, PeerInfo};
use crate::rendezvous::Peer;
use crate::ui::interaction::{InteractionHandler, PeerEvent};
use std::collections::HashMap;
use std::io::{self, Write};

pub struct CliHandler;

impl InteractionHandler for CliHandler {
    fn display_peers_list(&self, peers: &HashMap<String, Peer>) {
        if peers.is_empty() {
            println!("{:<15} | {:<15} | {:<6}", "Username", "IP Address", "Port");
            println!("{:-<42}", "");
            println!("No peers discovered yet.");
            println!("{:-<42}\n", "");
            return;
        }

        println!("{:<15} | {:<15} | {:<6}", "Username", "IP Address", "Port");
        println!("{:-<42}", ""); // Separator line

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

    fn on_advertising_start(&self, username: &str, ip: [u8; 4], port: u16, device_name_payload: &str) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("[{}] --- ADVERTISING ACTIVE ---", time_str);
        println!(
            "  Broadcasting Username: '{}', IP: {}.{}.{}.{}, Port: {} inside Base64 Name: '{}'",
            username, ip[0], ip[1], ip[2], ip[3], port, device_name_payload
        );
    }

    fn update_progress(&self, message: &str, done: u64, total: u64) {
        let percent = FileTransfer::percentage(done, total);
        let bar_width = 25;
        let filled_width = (percent / 100.0 * bar_width as f64) as usize;
        let empty_width = bar_width - filled_width;
        let bar = format!(
            "[{}{}]",
            "=".repeat(filled_width),
            "-".repeat(empty_width)
        );

        print!(
            "\r[{}] {} {:>3.0}% {} {} / {}",
            FileTransfer::timestamp(),
            message,
            percent,
            bar,
            FileTransfer::human_bytes(done),
            FileTransfer::human_bytes(total)
        );
        io::stdout().flush().ok();

        if done == total {
            println!();
        }
    }
}
