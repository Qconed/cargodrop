use crate::rendezvous::Peer;
use std::collections::HashMap;


pub enum PeerEvent {
    NewPeer(Peer, String),
    PeerLost(Peer, String),
}

pub trait InteractionHandler: Send + Sync {
    fn display_peers_list(&self, peers: &HashMap<String, Peer>);
    fn handle_peer_event(&self, event: PeerEvent);
}
