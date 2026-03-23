use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Handshake initial
    Hello {
        device_id: Uuid,
        device_name: String,
    },

    // Diffie-Hellman
    KeyExchange {
        public_key: Vec<u8>,
    },

    // Code de confirmation
    ConfirmationCode {
        code: String,
    },

    ConfirmationCodeVerify {
        code: String,
        verified: bool,
    },

    // Transfert de fichiers
    FileTransfer {
        file_name: String,
        file_size: u64,
        file_hash: String,
    },

    FileChunk {
        chunk_id: u32,
        chunk_size: u32,
        data: Vec<u8>,
    },

    TransferComplete {
        file_name: String,
        success: bool,
    },

    Error {
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::Hello {
            device_id: Uuid::new_v4(),
            device_name: "Test".to_string(),
        };
        let serialized = bincode::serialize(&msg).unwrap();
        let deserialized: Message = bincode::deserialize(&serialized).unwrap();
        
        match deserialized {
            Message::Hello { device_name, .. } => assert_eq!(device_name, "Test"),
            _ => panic!("Wrong message type"),
        }
    }
}