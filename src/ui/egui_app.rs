use egui::{Color32, RichText, ScrollArea, ProgressBar};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::rendezvous::Peer;

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
        }
    }
}

/// The main egui application
pub struct CargodropApp {
    pub state: GuiAppState,
}

impl Default for CargodropApp {
    fn default() -> Self {
        Self {
            state: GuiAppState::default(),
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
            let active_tab = ui.horizontal(|ui| {
                let mut active = 0;
                for (idx, tab_name) in tabs.iter().enumerate() {
                    if ui.selectable_label(idx == active, *tab_name).clicked() {
                        active = idx;
                    }
                }
                active
            }).inner;

            ui.separator();

            // Render active tab
            match active_tab {
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
            ui.label(RichText::new("Discovered Peers").strong());
            
            if ui.button("🔍 Start Discovery").clicked() {
                let state = self.state.clone();
                
                // Spawn a task to discover peers
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Ok(mut status) = state.app_status.lock() {
                            *status = "Discovering peers for 20 seconds...".to_string();
                        }
                        
                        // TODO: Call App::discover() here once we have access to App
                        // For now, just simulate
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        
                        if let Ok(mut status) = state.app_status.lock() {
                            *status = "Discovery complete".to_string();
                        }
                    });
                });
            }
            
            if let Ok(peers) = self.state.peers.lock() {
                if peers.is_empty() {
                    ui.label("No peers discovered. Start discovery to find peers.");
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
            ui.label(RichText::new("Send File").strong());

            // File selection
            ui.horizontal(|ui| {
                if let Ok(file) = self.state.selected_file.lock() {
                    if let Some(f) = file.as_ref() {
                        ui.label(format!("File: {}", f));
                    } else {
                        ui.label("No file selected");
                    }
                }
                
                if ui.button("📁 Browse").clicked() {
                    // TODO: Open file picker when rfd is available
                    ui.label("(File picker coming soon)");
                }
            });

            ui.separator();

            // Peer selection
            ui.label(RichText::new("Select Recipient").strong());
            
            if let Ok(peers) = self.state.peers.lock() {
                if peers.is_empty() {
                    ui.label("No peers available. Run discovery first.");
                } else {
                    let peer_names: Vec<String> = peers.keys().cloned().collect();
                    let selected = if let Ok(guard) = self.state.selected_peer.lock() {
                        (*guard).clone()
                    } else {
                        None
                    };
                    
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
                }
            }

            ui.separator();

            // Send button
            if ui.button("➤ Send File").clicked() {
                // TODO: Trigger actual send
                if let Ok(mut status) = self.state.app_status.lock() {
                    *status = "Sending file...".to_string();
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
            ui.label(RichText::new("Receive Files").strong());
            
            if let Ok(port) = self.state.listening_port.lock() {
                if let Some(p) = *port {
                    ui.label(format!("📡 Listening on port {}", p));
                    ui.label("Waiting for incoming transfers...");
                } else {
                    if ui.button("🎧 Start Listening").clicked() {
                        let state = self.state.clone();
                        
                        // Spawn a blocking task to start receiving
                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                if let Ok(mut status) = state.app_status.lock() {
                                    *status = "Starting receiver on port 5001...".to_string();
                                }
                                
                                // Simulate receiving (would call App::receive() here)
                                // For now, just update the port
                                if let Ok(mut port_guard) = state.listening_port.lock() {
                                    *port_guard = Some(5001);
                                }
                                
                                if let Ok(mut status) = state.app_status.lock() {
                                    *status = "Listening on port 5001".to_string();
                                }
                            });
                        });
                    }
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
