use crate::error::Result;
use crate::models::Message;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const CHUNK_SIZE: u32 = 1024 * 64; // 64 KB par chunk

pub struct FileTransfer;

impl FileTransfer {
    pub async fn send_file(path: &Path) -> Result<Vec<u8>> {
        let data = std::fs::read(path)
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
        
        println!("File size: {} bytes", data.len());
        Ok(data)
    }

    pub async fn receive_file(data: Vec<u8>, path: &Path) -> Result<()> {
        std::fs::write(path, data)
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
        
        println!(" File saved to {:?}", path);
        Ok(())
    }
}

/// Envoie un fichier à un serveur distant
pub async fn send_file_direct(
    target_addr: &str,
    target_port: u16,
    file_path: &str,
) -> Result<()> {
    let addr = format!("{}:{}", target_addr, target_port);
    println!("📡 Connexion à {}...", addr);

    // Établir la connexion TCP
    let mut stream = TcpStream::connect(&addr).await
        .map_err(|e| crate::error::AirdropError::Transfer(format!("Connexion échouée: {}", e)))?;

    println!("✅ Connecté!");

    // Lire les métadonnées du fichier
    let file_metadata = fs::metadata(file_path).await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
    let file_name = Path::new(file_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    println!("📁 Fichier: {} ({}KB)", file_name, file_metadata.len() / 1024);

    // Envoyer le message de transfert initial
    let transfer_msg = Message::FileTransfer {
        file_name: file_name.clone(),
        file_size: file_metadata.len(),
        file_hash: "TBD".to_string(),
    };

    let serialized = bincode::serialize(&transfer_msg)
        .map_err(|e| crate::error::AirdropError::Transfer(format!("Sérialisation échouée: {}", e)))?;

    // Envoyer la taille du message en premier (4 bytes)
    stream.write_all(&(serialized.len() as u32).to_le_bytes()).await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
    stream.write_all(&serialized).await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
    stream.flush().await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;

    println!("📤 Envoi du fichier...");

    // Lire et envoyer le fichier par chunks
    let mut file = fs::File::open(file_path).await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
    let mut buffer = vec![0; CHUNK_SIZE as usize];
    let mut chunk_id = 0u32;
    let mut total_sent = 0u64;

    loop {
        let n = file.read(&mut buffer).await
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
        if n == 0 {
            break;
        }

        let chunk_msg = Message::FileChunk {
            chunk_id,
            chunk_size: n as u32,
            data: buffer[..n].to_vec(),
        };

        let chunk_serialized = bincode::serialize(&chunk_msg)
            .map_err(|e| crate::error::AirdropError::Transfer(format!("Sérialisation échouée: {}", e)))?;

        // Envoyer la taille du message
        stream.write_all(&(chunk_serialized.len() as u32).to_le_bytes()).await
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
        stream.write_all(&chunk_serialized).await
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;

        total_sent += n as u64;
        let percent = (total_sent * 100) / file_metadata.len();
        println!("⏳ Progression: {}%", percent);

        chunk_id += 1;
    }

    // Envoyer le message de completion
    let complete_msg = Message::TransferComplete {
        file_name,
        success: true,
    };

    let complete_serialized = bincode::serialize(&complete_msg)
        .map_err(|e| crate::error::AirdropError::Transfer(format!("Sérialisation échouée: {}", e)))?;

    stream.write_all(&(complete_serialized.len() as u32).to_le_bytes()).await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
    stream.write_all(&complete_serialized).await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
    stream.flush().await
        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;

    println!("✅ Fichier envoyé avec succès!");

    Ok(())
}

/// Reçoit un fichier d'un client
pub async fn receive_file_direct(
    mut stream: TcpStream,
    download_dir: &str,
) -> Result<()> {
    let peer_addr = stream.peer_addr().ok();
    println!("🔗 Client connecté: {:?}", peer_addr);

    let mut buffer = vec![0; 4];
    let mut file_path = String::new();
    let mut file = None;

    loop {
        // Lire la taille du message (4 bytes)
        if stream.read_exact(&mut buffer).await.is_err() {
            break;
        }

        let msg_size = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        let mut msg_buffer = vec![0; msg_size];

        if stream.read_exact(&mut msg_buffer).await.is_err() {
            break;
        }

        let message: Message = bincode::deserialize(&msg_buffer)
            .map_err(|e| crate::error::AirdropError::Transfer(format!("Désérialisation échouée: {}", e)))?;

        match message {
            Message::FileTransfer {
                file_name,
                file_size,
                ..
            } => {
                file_path = format!("{}/{}", download_dir, file_name);
                file = Some(fs::File::create(&file_path).await
                    .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?);
                println!("📥 Réception: {} ({}KB)", file_name, file_size / 1024);
            }

            Message::FileChunk {
                chunk_id,
                data,
                ..
            } => {
                if let Some(ref mut f) = file {
                    f.write_all(&data).await
                        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
                    println!("⏳ Chunk {}: {} bytes reçus", chunk_id, data.len());
                }
            }

            Message::TransferComplete { success, .. } => {
                if let Some(f) = file.take() {
                    f.sync_all().await
                        .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
                }
                if success {
                    println!("✅ Fichier reçu avec succès: {}", file_path);
                }
                break;
            }

            Message::Error { message } => {
                println!("❌ Erreur: {}", message);
                break;
            }

            _ => {}
        }
    }

    Ok(())
}