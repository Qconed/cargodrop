use crate::error::{AirdropError, Result};
use crate::models::Message;
use crate::network::protocol::ProtocolHandler;
use crate::network::crypto::CryptoManager;
use crate::network::cert::CertificateManager;
use quinn::{Endpoint, ServerConfig, TransportConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Server {
    port: u16,
    endpoint: Arc<RwLock<Option<Endpoint>>>,
    active_connections: Arc<RwLock<Vec<String>>>,
}

impl Server {
    pub async fn new(port: u16) -> Result<Self> {
        Ok(Self {
            port,
            endpoint: Arc::new(RwLock::new(None)),
            active_connections: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Générer certificat auto-signé
        let (cert_pem, key_pem) = CertificateManager::generate_self_signed_cert("airdrop")?;
        let (cert_chain, private_key) = CertificateManager::load_certificate(&cert_pem, &key_pem)?;

        // Configurer le serveur QUIC
        let mut server_config = ServerConfig::with_single_cert(cert_chain, private_key)
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        let mut transport = TransportConfig::default();
        transport.max_idle_timeout(
            Some(std::time::Duration::from_secs(30).try_into().unwrap())
        );
        server_config.transport_config(Arc::new(transport));

        // Créer l'endpoint
        let addr: std::net::SocketAddr = format!("0.0.0.0:{}", self.port)
            .parse()
            .map_err(|e: std::net::AddrParseError| AirdropError::Network(e.to_string()))?;

        // ✅ Créer UdpSocket au lieu de passer SocketAddr
        let socket = std::net::UdpSocket::bind(addr)
            .map_err(|e| AirdropError::Network(e.to_string()))?;
        socket.set_nonblocking(true)
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        let endpoint = Endpoint::new(
            Default::default(),
            Some(server_config),
            socket,
            Arc::new(quinn::TokioRuntime),
        )
        .map_err(|e| AirdropError::Network(e.to_string()))?;

        println!("🚀 QUIC Server listening on {}", addr);
        *self.endpoint.write().await = Some(endpoint.clone());

        // Accepter les connexions
        self.accept_connections(endpoint).await?;
        Ok(())
    }

    async fn accept_connections(&self, endpoint: Endpoint) -> Result<()> {
        loop {
            if let Some(conn) = endpoint.accept().await {
                println!("✅ New QUIC connection incoming");

                let connections = Arc::clone(&self.active_connections);

                tokio::spawn(async move {
                    if let Err(e) = Self::handle_connection(conn, connections).await {
                        eprintln!("❌ Connection error: {}", e);
                    }
                });
            }
        }
    }

    async fn handle_connection(
        conn: quinn::Incoming,
        connections: Arc<RwLock<Vec<String>>>,
    ) -> Result<()> {
        let conn = conn
            .await
            .map_err(|e| AirdropError::Network(e.to_string()))?;

        let addr = conn.remote_address().to_string();
        println!("🔗 QUIC client connected: {}", addr);
        connections.write().await.push(addr.clone());

        loop {
            tokio::select! {
                stream = conn.accept_bi() => {
                    match stream {
                        Ok((mut send, mut recv)) => {
                            println!("📡 New bidirectional stream");

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_stream(&mut send, &mut recv).await {
                                    eprintln!("❌ Stream error: {}", e);
                                }
                            });
                        }
                        Err(_) => {
                            println!("❌ Connection closed: {}", addr);
                            break; // ✅ Juste break, return Result n'est pas nécessaire
                        }
                    }
                }
            }
        }
        // Connection fermée
        Ok(())
    }

    async fn handle_stream(
        send: &mut quinn::SendStream,
        recv: &mut quinn::RecvStream,
    ) -> Result<()> {
        let message = ProtocolHandler::receive_on_stream(recv).await?;
        println!("📨 Received: {:?}", message);

        let response = Self::process_message(message).await?;
        ProtocolHandler::send_on_stream(send, &response).await?;

        Ok(())
    }

    async fn process_message(message: Message) -> Result<Message> {
        match message {
            Message::Hello {
                device_id,
                device_name,
            } => {
                println!("👋 Hello from: {}", device_name);
                Ok(Message::Hello {
                    device_id,
                    device_name,
                })
            }
            Message::KeyExchange { public_key } => {
                println!("🔑 Received DH public key ({}b)", public_key.len());
                let (_, my_public) = CryptoManager::generate_dh_keypair();
                Ok(Message::KeyExchange {
                    public_key: my_public,
                })
            }
            Message::ConfirmationCode { code } => {
                println!("✔️ Confirmation code: {}", code);
                Ok(Message::ConfirmationCode { code })
            }
            _ => Ok(Message::Error {
                message: "Unknown message type".to_string(),
            }),
        }
    }
}