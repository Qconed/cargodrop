use crate::network::file_transfer::PeerInfo;
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
}
