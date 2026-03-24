use crate::error::{AirdropError, Result};
use rcgen::generate_simple_self_signed;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::Cursor;

pub struct CertificateManager;

impl CertificateManager {
    pub fn generate_self_signed_cert(name: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        let subject_alt_names = vec![name.to_string(), "localhost".to_string()];

        let cert = generate_simple_self_signed(subject_alt_names)
            .map_err(|e| AirdropError::Network(format!("Certificate generation failed: {}", e)))?;

        let cert_pem = cert.serialize_pem()
            .map_err(|e| AirdropError::Network(format!("PEM serialization failed: {}", e)))?;

        let key_pem = cert.serialize_private_key_pem();

        Ok((cert_pem.into_bytes(), key_pem.into_bytes()))
    }

    pub fn load_certificate(
        cert_pem: &[u8],
        key_pem: &[u8],
    ) -> Result<(
        Vec<rustls::pki_types::CertificateDer<'static>>,
        rustls::pki_types::PrivateKeyDer<'static>,
    )> {
        let mut cert_reader = Cursor::new(cert_pem);
        let cert_chain: Vec<_> = certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AirdropError::Network(format!("Failed to parse certificate: {:?}", e)))?
            .into_iter()
            .map(|c| rustls::pki_types::CertificateDer::from(c.to_vec()))
            .collect();

        let mut key_reader = Cursor::new(key_pem);
        let keys: Vec<_> = pkcs8_private_keys(&mut key_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AirdropError::Network(format!("Failed to parse key: {:?}", e)))?;

        if keys.is_empty() {
            return Err(AirdropError::Network("No private key found".to_string()));
        }

        
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(keys[0].clone_key());

        Ok((cert_chain,key))

    }
    

}