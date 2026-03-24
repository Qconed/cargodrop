use crate::error::{AirdropError, Result};
use crate::models::{Device, Message};
use crate::network::protocol::ProtocolHandler;
use quinn::Endpoint;
use std::sync::Arc;

pub struct Client {
    #[allow(dead_code)]
    local_device: Device,
}

impl Client {
    pub fn new(local_device: Device) -> Self {
        Self { local_device }
    }

    pub async fn connect_to(&self, remote_device: &Device) -> Result<ClientConnection> {
        let addr = remote_device.socket_addr();
        println!("🔗 Connecting to {} via QUIC...", remote_device.name);

        let socket = std::net::UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| AirdropError::Network(e.to_string()))?;
        socket.set_nonblocking(true)
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        let endpoint = Endpoint::new(
            Default::default(),
            None,
            socket,
            Arc::new(quinn::TokioRuntime),
        )
        .map_err(|e| AirdropError::Network(e.to_string()))?;

        // Se connecter
        let conn = endpoint
            .connect(addr, &remote_device.name)
            .map_err(|e| AirdropError::Network(e.to_string()))?
            .await
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        println!("✅ Connected to {} via QUIC!", remote_device.name);

        Ok(ClientConnection {
            connection: conn,
            endpoint,
            remote_device: remote_device.clone(),
        })
    }
}

pub struct ClientConnection {
    connection: quinn::Connection,
    #[allow(dead_code)]
    endpoint: Endpoint,
    remote_device: Device,
}

impl ClientConnection {
    pub async fn send_message(&self, message: &Message) -> Result<()> {
        let (mut send, _recv) = self
            .connection
            .open_bi()
            .await
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        ProtocolHandler::send_on_stream(&mut send, message).await?;
        
        send.finish()
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        println!("📤 Message sent to {}", self.remote_device.name);
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<Message> {
        let (_send, mut recv) = self
            .connection
            .accept_bi()
            .await
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        let message = ProtocolHandler::receive_on_stream(&mut recv).await?;
        Ok(message)
    }
}