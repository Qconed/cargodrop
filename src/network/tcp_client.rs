use std::error::Error;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use crate::network::file_transfer::{FileTransfer, PeerInfo, TransferRequest, TransferResponse};

/// TCP sender that connects to a peer and streams one file.
pub struct TcpClient {
    peer: PeerInfo,
    local_device_name: String,
}

impl TcpClient {
    /// Creates a new sender client.
    pub fn new(peer: PeerInfo, local_device_name: String) -> Self {
        Self {
            peer,
            local_device_name,
        }
    }

    /// Connects to the peer, performs handshake, and sends the file bytes.
    pub fn send_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let address = format!("{}:{}", self.peer.ip, self.peer.port);
        println!(
            "[{}] Connecting to {} ({})",
            FileTransfer::timestamp(),
            address,
            self.peer.device_name
        );

        let mut stream = TcpStream::connect(&address)?;
        let file_header = FileTransfer::build_file_header(file_path)?;

        let request = TransferRequest {
            device_name: self.local_device_name.clone(),
            file_header: file_header.clone(),
        };
        FileTransfer::send_json_message(&mut stream, &request)?;

        let response: TransferResponse = FileTransfer::read_json_message(&mut stream)?;
        if !response.accepted {
            return Err(format!("Transfer refused by peer: {}", response.message).into());
        }

        println!(
            "[{}] Handshake accepted by '{}' - sending '{}' ({}).",
            FileTransfer::timestamp(),
            response.device_name,
            file_header.filename,
            FileTransfer::human_bytes(file_header.file_size)
        );

        let total_size = file_header.file_size;
        let (progress_tx, progress_rx) = mpsc::channel::<u64>();

        let progress_thread = thread::spawn(move || {
            while let Ok(done) = progress_rx.recv() {
                let percent = FileTransfer::percentage(done, total_size);
                println!(
                    "[{}] Sending... {:.0}% ({} / {})",
                    FileTransfer::timestamp(),
                    percent,
                    FileTransfer::human_bytes(done),
                    FileTransfer::human_bytes(total_size)
                );
            }
        });

        FileTransfer::send_file_bytes(&mut stream, file_path, progress_tx)?;

        if let Err(err) = progress_thread.join() {
            eprintln!(
                "[{}] Progress thread ended unexpectedly: {:?}",
                FileTransfer::timestamp(),
                err
            );
        }

        println!("[{}] File sent successfully.", FileTransfer::timestamp());
        Ok(())
    }
}