use crate::error::Result;
use crate::models::Message;

pub struct ProtocolHandler;

impl ProtocolHandler {
    pub fn serialize_message(message: &Message) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(message)?;
        Ok(serialized)
    }

    pub fn deserialize_message(data: &[u8]) -> Result<Message> {
        let message = bincode::deserialize(data)?;
        Ok(message)
    }

    pub async fn send_on_stream(
        send: &mut quinn::SendStream,
        message: &Message,
    ) -> Result<()> {
        let serialized = Self::serialize_message(message)?;
        send.write_all(&serialized).await
            .map_err(|e| crate::error::AirdropError::Network(e.to_string()))?;
        Ok(())
    }

    pub async fn receive_on_stream(
        recv: &mut quinn::RecvStream,
    ) -> Result<Message> {
        let mut buffer = vec![0u8; 65536];
        match recv.read(&mut buffer).await {
            Ok(Some(n)) => {
                buffer.truncate(n);
                Self::deserialize_message(&buffer)
            }
            Ok(None) => Err(crate::error::AirdropError::Network("Stream closed".to_string())),
            Err(e) => Err(crate::error::AirdropError::Network(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_message_protocol() {
        let msg = Message::Hello {
            device_id: Uuid::new_v4(),
            device_name: "Test".to_string(),
        };

        let serialized = ProtocolHandler::serialize_message(&msg).unwrap();
        let deserialized = ProtocolHandler::deserialize_message(&serialized).unwrap();

        match deserialized {
            Message::Hello { device_name, .. } => assert_eq!(device_name, "Test"),
            _ => panic!("Wrong message"),
        }
    }
}