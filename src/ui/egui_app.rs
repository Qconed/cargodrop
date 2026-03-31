use egui::{Color32, RichText, ScrollArea, ProgressBar};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use crate::rendezvous::{Peer, RendezvousManager};
use crate::ui::gui_handler::GuiHandler;

/// Shared state for the GUI application
#[derive(Clone)]
pub struct GuiAppState {
    pub peers: Arc<Mutex<HashMap<String, Peer>>>,
    pub selected_peer: Arc<Mutex<Option<String>>>,
    pub selected_file: Arc<Mutex<Option<String>>>,
    pub transfer_progress: Arc<Mutex<TransferProgress>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub confirmation_pending: Arc<Mutex<PendingConfirmation>>,
    pub app_status: Arc<Mutex<String>>,
    pub listening_port: Arc<Mutex<Option<u16>>>,
    pub discovery_active: Arc<Mutex<bool>>,
    pub discovery_start_time: Arc<Mutex<Option<SystemTime>>>,
    pub advertising_active: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug)]
pub struct TransferProgress {
    pub filename: String,
    pub percent: f64,
    pub sent: u64,
    pub total: u64,
    pub is_active: bool,
}

impl Default for TransferProgress {
    fn default() -> Self {
        Self {
            filename: String::new(),
            percent: 0.0,
            sent: 0,
            total: 1,
            is_active: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    pub content: String,
    pub level: MessageLevel,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageLevel {
    Info,
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct PendingConfirmation {
    pub sender: String,
    pub filename: String,
    pub size: u64,
    pub response: Option<bool>,
}

impl Default for GuiAppState {
    fn default() -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
            selected_peer: Arc::new(Mutex::new(None)),
            selected_file: Arc::new(Mutex::new(None)),
            transfer_progress: Arc::new(Mutex::new(TransferProgress::default())),
            messages: Arc::new(Mutex::new(Vec::new())),
            confirmation_pending: Arc::new(Mutex::new(PendingConfirmation {
                sender: String::new(),
                filename: String::new(),
                size: 0,
                response: None,
            })),
            app_status: Arc::new(Mutex::new("Ready".to_string())),
            listening_port: Arc::new(Mutex::new(None)),
            discovery_active: Arc::new(Mutex::new(false)),
            discovery_start_time: Arc::new(Mutex::new(None)),
            advertising_active: Arc::new(Mutex::new(false)),
        }
    }
}

/// The main egui application
pub struct CargodropApp {
    pub state: GuiAppState,
    pub active_tab: usize,  // Track which tab is active (0=Status, 1=Discover, 2=Send, 3=Receive)
    pub app: Option<Arc<dyn std::any::Any + Send + Sync>>, // Will be set to the App instance
    pub gui_handler: Arc<GuiHandler>,
}

impl Default for CargodropApp {
    fn default() -> Self {
        let state = GuiAppState::default();
        Self {
            gui_handler: Arc::new(GuiHandler::new(state.clone())),
            state,
            active_tab: 0,
            app: None,
        }
    }
}

impl eframe::App for CargodropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme
        let mut style = egui::Style::default();
        style.visuals = egui::Visuals::dark();
        ctx.set_style(style);

        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("📦 CargoDrop").heading().color(Color32::from_rgb(100, 200, 250)));
            
            ui.separator();

            // Tab system
            let tabs = vec!["Status", "Discover", "Send", "Receive"];
            ui.horizontal(|ui| {
                for (idx, tab_name) in tabs.iter().enumerate() {
                    if ui.selectable_label(self.active_tab == idx, *tab_name).clicked() {
                        self.active_tab = idx;
                    }
                }
            });

            ui.separator();

            // Render active tab
            match self.active_tab {
                0 => self.render_status_tab(ui),
                1 => self.render_discover_tab(ui),
                2 => self.render_send_tab(ui),
                3 => self.render_receive_tab(ui),
                _ => {}
            }

            // Render confirmation dialog if needed
            if let Ok(confirmation) = self.state.confirmation_pending.lock() {
                if confirmation.response.is_none() && !confirmation.filename.is_empty() {
                    drop(confirmation); // Release lock before showing dialog
                    self.render_confirmation_dialog(ui);
                }
            }

            // Render messages
            self.render_messages(ui);
        });
    }
}

impl CargodropApp {
    fn render_status_tab(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Application Status").strong());
            
            if let Ok(status) = self.state.app_status.lock() {
                ui.label(format!("Status: {}", status));
            }

            if let Ok(port) = self.state.listening_port.lock() {
                if let Some(p) = *port {
                    ui.label(format!("📡 Listening on port: {}", p));
                }
            }
        });

        ui.separator();

        ui.group(|ui| {
            ui.label(RichText::new("Quick Stats").strong());
            
            if let Ok(peers) = self.state.peers.lock() {
                ui.label(format!("Connected peers: {}", peers.len()));
            }
        });
    }

    fn render_discover_tab(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("🔍 Manual Peer Discovery").strong());
            ui.label("Note: Peer discovery is now integrated in the 'Send' tab (interactive mode).");
            ui.label("Use this tab to manually discover peers without sending a file.");
            
            ui.separator();

            let mut discovery_active = false;
            if let Ok(active) = self.state.discovery_active.lock() {
                discovery_active = *active;
            }

            if discovery_active {
                ui.label("🔍 Discovering peers...");
                if let Ok(start_guard) = self.state.discovery_start_time.lock() {
                    if let Some(start_time) = *start_guard {
                        if let Ok(elapsed) = start_time.elapsed() {
                            ui.label(format!("Elapsed: {} seconds", elapsed.as_secs()));
                        }
                    }
                }
                if ui.button("⏹️ Stop Discovery").clicked() {
                    if let Ok(mut active) = self.state.discovery_active.lock() {
                        *active = false;
                    }
                }
            } else {
                if ui.button("🔍 Start Manual Discovery").clicked() {
                    let state = self.state.clone();
                    
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            if let Ok(mut active) = state.discovery_active.lock() {
                                *active = true;
                            }
                            if let Ok(mut start) = state.discovery_start_time.lock() {
                                *start = Some(SystemTime::now());
                            }

                            if let Ok(mut messages) = state.messages.lock() {
                                messages.push(Message {
                                    content: "Starting CargoDrop in Discovery Mode...".to_string(),
                                    level: MessageLevel::Info,
                                });
                            }
                            
                            // Create peer map and handler for discovery
                            let peers = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
                            let handler = Arc::new(GuiHandler::new(state.clone()));
                            
                            // Call real discovery
                            match RendezvousManager::discover_manage(peers, handler).await {
                                Ok(_) => {
                                    if let Ok(mut messages) = state.messages.lock() {
                                        messages.push(Message {
                                            content: "Discovery completed successfully".to_string(),
                                            level: MessageLevel::Success,
                                        });
                                    }
                                },
                                Err(e) => {
                                    if let Ok(mut messages) = state.messages.lock() {
                                        messages.push(Message {
                                            content: format!("Discovery error: {}", e),
                                            level: MessageLevel::Error,
                                        });
                                    }
                                }
                            }

                            if let Ok(mut active) = state.discovery_active.lock() {
                                *active = false;
                            }
                        });
                    });
                }
            }
            
            ui.separator();
            ui.label(RichText::new("Discovered Peers").strong());
            
            if let Ok(peers) = self.state.peers.lock() {
                if peers.is_empty() {
                    ui.label("No peers discovered yet.");
                } else {
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (name, peer) in peers.iter() {
                            ui.horizontal(|ui| {
                                let ip_str = format!(
                                    "{}.{}.{}.{}",
                                    peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]
                                );
                                ui.label(format!("👤 {} - {}:{}", name, ip_str, peer.port));
                            });
                        }
                    });
                }
            }
        });
    }

    fn render_send_tab(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("📤 Send File (Interactive Mode)").strong());
            ui.label("This mode discovers peers and lets you choose who to send to.");
            
            ui.separator();

            // File selection
            ui.horizontal(|ui| {
                ui.label("File to send:");
                if let Ok(file) = self.state.selected_file.lock() {
                    if let Some(f) = file.as_ref() {
                        ui.label(format!("{}", f));
                    } else {
                        ui.label("(No file selected)");
                    }
                } else {
                    ui.label("(No file selected)");
                }
            });

            ui.separator();

            // Discovery button
            let mut discovery_active = false;
            if let Ok(active) = self.state.discovery_active.lock() {
                discovery_active = *active;
            }

            if discovery_active {
                // Calculate time remaining
                let mut time_remaining: i32 = 20;
                if let Ok(start_guard) = self.state.discovery_start_time.lock() {
                    if let Some(start_time) = *start_guard {
                        if let Ok(elapsed) = start_time.elapsed() {
                            time_remaining = 20 - (elapsed.as_secs().min(20) as i32);
                        }
                    }
                }

                ui.horizontal(|ui| {
                    ui.label(format!("🔍 Discovering peers... ({} seconds remaining)", time_remaining));
                    ui.add(ProgressBar::new((1.0 - time_remaining as f32 / 20.0).clamp(0.0, 1.0)));
                });
            } else {
                if ui.button("🔍 Start Discovery").clicked() {
                    let state = self.state.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            if let Ok(mut active) = state.discovery_active.lock() {
                                *active = true;
                            }
                            if let Ok(mut start) = state.discovery_start_time.lock() {
                                *start = Some(SystemTime::now());
                            }
                            if let Ok(mut status) = state.app_status.lock() {
                                *status = "Discovering peers for 20 seconds...".to_string();
                            }
                            if let Ok(mut messages) = state.messages.lock() {
                                messages.push(Message {
                                    content: "Starting peer discovery for 20 seconds...".to_string(),
                                    level: MessageLevel::Info,
                                });
                            }

                            // Create peer map and handler for discovery
                            let peers = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
                            let handler = Arc::new(GuiHandler::new(state.clone()));

                            // Run discovery with 20 second timeout
                            let discover_future = async {
                                RendezvousManager::discover_manage(peers, handler).await
                            };

                            let result = tokio::time::timeout(
                                std::time::Duration::from_secs(20),
                                discover_future
                            ).await;

                            if let Ok(Ok(_)) = result {
                                if let Ok(mut messages) = state.messages.lock() {
                                    messages.push(Message {
                                        content: "Discovery completed! Select a peer and your file, then click 'Send File'.".to_string(),
                                        level: MessageLevel::Success,
                                    });
                                }
                            } else {
                                if let Ok(mut messages) = state.messages.lock() {
                                    messages.push(Message {
                                        content: "Discovery timeout or error. Check discovered peers and retry if needed.".to_string(),
                                        level: MessageLevel::Info,
                                    });
                                }
                            }

                            if let Ok(mut active) = state.discovery_active.lock() {
                                *active = false;
                            }
                            if let Ok(mut status) = state.app_status.lock() {
                                *status = "Ready - select peer and send file".to_string();
                            }
                        });
                    });
                }
            }

            ui.separator();

            // Peer selection
            ui.label(RichText::new("Select Recipient").strong());
            
            if let Ok(peers) = self.state.peers.lock() {
                if peers.is_empty() {
                    ui.label("No peers discovered. Click 'Start Discovery' to find peers.");
                } else {
                    let peer_names: Vec<String> = peers.keys().cloned().collect();
                    let selected = if let Ok(guard) = self.state.selected_peer.lock() {
                        (*guard).clone()
                    } else {
                        None
                    };
                    
                    ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                        for peer_name in peer_names {
                            if ui.selectable_label(
                                selected.as_ref() == Some(&peer_name),
                                format!("👤 {}", peer_name)
                            ).clicked() {
                                if let Ok(mut p) = self.state.selected_peer.lock() {
                                    *p = Some(peer_name);
                                }
                            }
                        }
                    });
                }
            }

            ui.separator();

            // Send button
            let can_send = {
                let peers = self.state.peers.lock().ok().map(|p| !p.is_empty()).unwrap_or(false);
                let peer_selected = self.state.selected_peer.lock().ok().and_then(|p| p.as_ref().cloned()).is_some();
                let file_selected = self.state.selected_file.lock().ok().and_then(|f| f.as_ref().cloned()).is_some();
                peers && peer_selected && file_selected && !discovery_active
            };

            if ui.add_enabled(can_send, egui::Button::new("➤ Send File")).clicked() {
                if let Ok(Some(peer_name)) = self.state.selected_peer.lock().map(|p| p.as_ref().cloned()) {
                    if let Ok(Some(file_path)) = self.state.selected_file.lock().map(|f| f.as_ref().cloned()) {
                        if let Ok(peers) = self.state.peers.lock() {
                            if let Some(peer) = peers.get(&peer_name) {
                                let ip = format!("{}.{}.{}.{}", peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
                                let port = peer.port;
                                let state = self.state.clone();
                                
                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        if let Ok(mut status) = state.app_status.lock() {
                                            *status = "Sending file...".to_string();
                                        }
                                        
                                        // TODO: Actually send the file using the app's send() method
                                        // For now, just show a message
                                        if let Ok(mut messages) = state.messages.lock() {
                                            messages.push(Message {
                                                content: format!("Sending '{}' to {}:{}", file_path, ip, port),
                                                level: MessageLevel::Info,
                                            });
                                        }
                                    });
                                });
                            }
                        }
                    }
                }
            }
        });

        // Progress bar
        if let Ok(progress) = self.state.transfer_progress.lock() {
            if progress.is_active {
                ui.separator();
                ui.group(|ui| {
                    ui.label(RichText::new("Transfer Progress").strong());
                    ui.label(format!("File: {}", progress.filename));
                    ui.add(ProgressBar::new((progress.percent as f32 / 100.0).clamp(0.0, 1.0)));
                    ui.label(format!(
                        "{} / {} bytes ({:.1}%)",
                        progress.sent, progress.total, progress.percent
                    ));
                });
            }
        }
    }

    fn render_receive_tab(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("📥 Receive Files (with Advertisement)").strong());
            ui.label("This mode advertises your device and listens for incoming file transfers.");
            
            ui.separator();

            let is_listening = {
                if let Ok(port) = self.state.listening_port.lock() {
                    port.is_some()
                } else {
                    false
                }
            };

            let is_advertising = {
                if let Ok(adv) = self.state.advertising_active.lock() {
                    *adv
                } else {
                    false
                }
            };

            if is_listening {
                ui.horizontal(|ui| {
                    ui.label("✅ Status: Active");
                });
                if let Ok(port) = self.state.listening_port.lock() {
                    if let Some(p) = *port {
                        ui.label(format!("📡 Listening on port: {}", p));
                    }
                }
                if is_advertising {
                    ui.label("📢 Currently advertising yourself to peers...");
                }
                ui.label("Waiting for incoming file transfers...");

                if ui.button("⏹️ Stop Listening").clicked() {
                    // Stop listening
                    if let Ok(mut port) = self.state.listening_port.lock() {
                        *port = None;
                    }
                    if let Ok(mut adv) = self.state.advertising_active.lock() {
                        *adv = false;
                    }
                    if let Ok(mut status) = self.state.app_status.lock() {
                        *status = "Receiver stopped".to_string();
                    }
                }
            } else {
                if ui.button("🎧 Start Listening (with Advertisement)").clicked() {
                    let state = self.state.clone();
                    
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            if let Ok(mut adv) = state.advertising_active.lock() {
                                *adv = true;
                            }

                            if let Ok(mut messages) = state.messages.lock() {
                                messages.push(Message {
                                    content: "Starting CargoDrop in Receive Mode (with background advertisement)...".to_string(),
                                    level: MessageLevel::Info,
                                });
                            }

                            // Set listening port (default 8080 from user info)
                            // TODO: Get actual port from app.user_info
                            let port = 8080u16;
                            if let Ok(mut listening_port) = state.listening_port.lock() {
                                *listening_port = Some(port);
                            }

                            if let Ok(mut status) = state.app_status.lock() {
                                *status = format!("Listening on port {}, advertising...", port);
                            }

                            if let Ok(mut messages) = state.messages.lock() {
                                messages.push(Message {
                                    content: format!("📡 Receiver listening on port {}", port),
                                    level: MessageLevel::Success,
                                });
                                messages.push(Message {
                                    content: "Waiting for incoming file transfers...".to_string(),
                                    level: MessageLevel::Info,
                                });
                            }

                            // TODO: Actually call app.advertise_and_receive(port) here
                            // For now, we just show the UI state
                        });
                    });
                }
            }
        });

        // Progress bar
        if let Ok(progress) = self.state.transfer_progress.lock() {
            if progress.is_active {
                ui.separator();
                ui.group(|ui| {
                    ui.label(RichText::new("Receiving Progress").strong());
                    ui.label(format!("File: {}", progress.filename));
                    ui.add(ProgressBar::new((progress.percent as f32 / 100.0).clamp(0.0, 1.0)));
                    ui.label(format!(
                        "{} / {} bytes ({:.1}%)",
                        progress.sent, progress.total, progress.percent
                    ));
                });
            }
        }
    }

    fn render_confirmation_dialog(&self, ui: &mut egui::Ui) {
        // Create a modal dialog for confirmation
        if let Ok(confirmation) = self.state.confirmation_pending.lock() {
            let conf = confirmation.clone();
            drop(confirmation);

            if conf.response.is_none() {
                egui::Window::new("⚠️ Incoming Transfer")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("From: {}", conf.sender));
                            ui.label(format!("File: {}", conf.filename));
                            ui.label(format!("Size: {} bytes", conf.size));
                            
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                if ui.button("✅ Accept").clicked() {
                                    if let Ok(mut c) = self.state.confirmation_pending.lock() {
                                        c.response = Some(true);
                                    }
                                }
                                
                                if ui.button("❌ Reject").clicked() {
                                    if let Ok(mut c) = self.state.confirmation_pending.lock() {
                                        c.response = Some(false);
                                    }
                                }
                            });
                        });
                    });
            }
        }
    }

    fn render_messages(&self, ui: &mut egui::Ui) {
        if let Ok(messages) = self.state.messages.lock() {
            if !messages.is_empty() {
                ui.separator();
                ui.group(|ui| {
                    ui.label(RichText::new("Messages").strong());
                    ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                        for msg in messages.iter() {
                            let color = match msg.level {
                                MessageLevel::Info => Color32::LIGHT_BLUE,
                                MessageLevel::Success => Color32::GREEN,
                                MessageLevel::Error => Color32::RED,
                            };
                            
                            let icon = match msg.level {
                                MessageLevel::Info => "ℹ️",
                                MessageLevel::Success => "✅",
                                MessageLevel::Error => "❌",
                            };
                            
                            ui.label(RichText::new(format!("{} {}", icon, msg.content)).color(color));
                        }
                    });
                });
            }
        }
    }
}
