use crate::rendezvous::Peer;
use crate::network::file_transfer::PeerInfo;
use std::collections::HashMap;

/// Events related to the management of Peers
/// (using events like so allows to have one function handle_event instead of multiple ones)
pub enum PeerEvent {
    NewPeer(Peer, String),
    PeerLost(Peer, String),
}

/// Trait for UI interactions: displaying information, requesting info from the user...
/// The CLI and GUI will both implement this, allowing handler methods to be called freely throughout the app
pub trait InteractionHandler: Send + Sync {
    // ===== Peer Discovery & Management =====
    fn display_peers_list(&self, peers: &HashMap<String, Peer>);
    fn handle_peer_event(&self, event: PeerEvent);
    fn select_peer(&self, peers: &[PeerInfo]) -> Option<PeerInfo>;

    // ===== File Selection & Transfer =====
    fn select_file_to_send(&self) -> Option<String>;
    fn confirm_transfer(&self, sender: &str, filename: &str, size: u64) -> bool;

    // ===== Progress & Status Updates =====
    fn show_transfer_progress(&self, filename: &str, percent: f64, sent: u64, total: u64);
    fn show_app_status(&self, status: &str);
    fn show_receiver_listening(&self, port: u16);

    // ===== Messages (Error & Success) =====
    fn show_error(&self, message: &str);
    fn show_success(&self, message: &str);
    fn show_info(&self, message: &str);

    // ===== File Management =====
    fn show_received_files(&self, files: &[String]);
    fn request_save_location(&self, filename: &str) -> Option<String>;
}
