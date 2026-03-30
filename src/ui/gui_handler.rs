use crate::network::file_transfer::PeerInfo;
use crate::rendezvous::Peer;
use crate::ui::interaction::{InteractionHandler, PeerEvent};
use crate::ui::egui_app::{GuiAppState, Message, MessageLevel};
use std::collections::HashMap;

/// GUI Handler for egui application
/// Communicates with GuiAppState to update the GUI
pub struct GuiHandler {
    pub state: GuiAppState,
}

impl GuiHandler {
    pub fn new(state: GuiAppState) -> Self {
        Self { state }
    }
}

impl InteractionHandler for GuiHandler {
    // ===== Peer Discovery & Management =====
    fn display_peers_list(&self, peers: &HashMap<String, Peer>) {
        if let Ok(mut state_peers) = self.state.peers.lock() {
            *state_peers = peers.clone();
        }
    }

    fn handle_peer_event(&self, event: PeerEvent) {
        let message = match event {
            PeerEvent::NewPeer(peer, time) => {
                format!("New peer detected: {} at {}", peer.username, time)
            }
            PeerEvent::PeerLost(peer, time) => {
                format!("Peer disconnected: {} at {}", peer.username, time)
            }
        };
        
        if let Ok(mut messages) = self.state.messages.lock() {
            messages.push(Message {
                content: message,
                level: MessageLevel::Info,
            });
        }
    }

    fn select_peer(&self, peers: &[PeerInfo]) -> Option<PeerInfo> {
        // Try to get selected peer from state
        if let Ok(selected) = self.state.selected_peer.lock() {
            if let Some(selected_name) = selected.as_ref() {
                // Find peer by name
                if let Some(peer) = peers.iter().find(|p| &p.device_name == selected_name) {
                    return Some(peer.clone());
                }
            }
        }
        
        // Fallback: return first peer
        peers.first().cloned()
    }

    // ===== File Selection & Transfer =====
    fn select_file_to_send(&self) -> Option<String> {
        // Return selected file from state
        if let Ok(file) = self.state.selected_file.lock() {
            file.as_ref().cloned()
        } else {
            None
        }
    }

    fn confirm_transfer(&self, sender: &str, filename: &str, size: u64) -> bool {
        // Set pending confirmation and wait for user response
        if let Ok(mut confirmation) = self.state.confirmation_pending.lock() {
            confirmation.sender = sender.to_string();
            confirmation.filename = filename.to_string();
            confirmation.size = size;
            confirmation.response = None;
        }
        
        // Wait for user response (in a real app, this would be async)
        // For now, we'll poll the state
        for _ in 0..100 {
            if let Ok(confirmation) = self.state.confirmation_pending.lock() {
                if let Some(response) = confirmation.response {
                    return response;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Timeout: reject transfer
        false
    }

    // ===== Progress & Status Updates =====
    fn show_transfer_progress(&self, filename: &str, percent: f64, sent: u64, total: u64) {
        if let Ok(mut progress) = self.state.transfer_progress.lock() {
            progress.filename = filename.to_string();
            progress.percent = percent;
            progress.sent = sent;
            progress.total = total;
            progress.is_active = true;
        }
    }

    fn show_app_status(&self, status: &str) {
        if let Ok(mut app_status) = self.state.app_status.lock() {
            *app_status = status.to_string();
        }
    }

    fn show_receiver_listening(&self, port: u16) {
        if let Ok(mut listening_port) = self.state.listening_port.lock() {
            *listening_port = Some(port);
        }
        
        if let Ok(mut status) = self.state.app_status.lock() {
            *status = format!("Listening on port {}", port);
        }
    }

    // ===== Messages (Error & Success) =====
    fn show_error(&self, message: &str) {
        if let Ok(mut messages) = self.state.messages.lock() {
            messages.push(Message {
                content: message.to_string(),
                level: MessageLevel::Error,
            });
        }
    }

    fn show_success(&self, message: &str) {
        if let Ok(mut messages) = self.state.messages.lock() {
            messages.push(Message {
                content: message.to_string(),
                level: MessageLevel::Success,
            });
        }
    }

    fn show_info(&self, message: &str) {
        if let Ok(mut messages) = self.state.messages.lock() {
            messages.push(Message {
                content: message.to_string(),
                level: MessageLevel::Info,
            });
        }
    }

    // ===== File Management =====
    fn show_received_files(&self, files: &[String]) {
        let files_list = files.join(", ");
        self.show_info(&format!("Received files: {}", files_list));
    }

    fn request_save_location(&self, filename: &str) -> Option<String> {
        // In a real app, this would show a file dialog
        // For now, return the received folder
        Some(format!("received/{}", filename))
    }
}
