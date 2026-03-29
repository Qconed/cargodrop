use crate::ui::interaction::{InteractionHandler, PeerEvent};
use crate::rendezvous::Peer;
use std::collections::HashMap;

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
            let ip_str = format!("{}.{}.{}.{}", peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
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
}
