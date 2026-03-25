use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use crate::network::file_transfer::{FileTransfer, TransferRequest, TransferResponse};

/// TCP receiver that accepts incoming file transfers.
pub struct TcpServer {
    bind_port: u16,
    device_name: String,
}

impl TcpServer {
    /// Creates a new TCP server bound to 0.0.0.0:port.
    pub fn new(port: u16, device_name: String) -> Self {
        Self {
            bind_port: port,
            device_name,
        }
    }

    /// Starts listening and spawns one thread per incoming connection.
    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let address = format!("0.0.0.0:{}", self.bind_port);
        let listener = TcpListener::bind(&address)?;

        println!(
            "[{}] Receiver listening on {}",
            FileTransfer::timestamp(),
            address
        );

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let device_name = self.device_name.clone();
                    thread::spawn(move || {
                        if let Err(err) = Self::handle_connection(stream, device_name) {
                            eprintln!("[{}] Connection error: {}", FileTransfer::timestamp(), err);
                        }
                    });
                }
                Err(err) => {
                    eprintln!("[{}] Accept error: {}", FileTransfer::timestamp(), err);
                }
            }
        }

        Ok(())
    }

    fn handle_connection(mut stream: TcpStream, device_name: String) -> Result<(), Box<dyn Error>> {
        let peer_addr = stream.peer_addr()?;
        println!(
            "[{}] Incoming connection from {}",
            FileTransfer::timestamp(),
            peer_addr
        );

        let request: TransferRequest = FileTransfer::read_json_message(&mut stream)?;
        println!(
            "[{}] Handshake from '{}' for file '{}' ({}).",
            FileTransfer::timestamp(),
            request.device_name,
            request.file_header.filename,
            FileTransfer::human_bytes(request.file_header.file_size)
        );

        let response = TransferResponse {
            device_name,
            accepted: true,
            message: "Ready to receive".to_string(),
        };
        FileTransfer::send_json_message(&mut stream, &response)?;

        std::fs::create_dir_all("received")?;
        let output_path = format!("received/{}", request.file_header.filename);
        let mut output_file = File::create(&output_path)?;

        let total_size = request.file_header.file_size;
        let (progress_tx, progress_rx) = mpsc::channel::<u64>();

        let progress_thread = thread::spawn(move || {
            while let Ok(done) = progress_rx.recv() {
                let percent = FileTransfer::percentage(done, total_size);
                println!(
                    "[{}] Receiving... {:.0}% ({} / {})",
                    FileTransfer::timestamp(),
                    percent,
                    FileTransfer::human_bytes(done),
                    FileTransfer::human_bytes(total_size)
                );
            }
        });

        let mut reader = BufReader::new(stream);
        FileTransfer::receive_file_bytes(&mut reader, &mut output_file, total_size, progress_tx)?;

        if let Err(err) = progress_thread.join() {
            eprintln!(
                "[{}] Progress thread ended unexpectedly: {:?}",
                FileTransfer::timestamp(),
                err
            );
        }

        println!(
            "[{}] File received successfully and saved to './{}'.",
            FileTransfer::timestamp(),
            output_path
        );

        Ok(())
    }
}