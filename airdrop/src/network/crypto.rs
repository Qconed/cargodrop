use crate::error::Result;
use rand::Rng;
use sha2::{Sha256, Digest};

pub struct CryptoManager;

impl CryptoManager {
    pub fn generate_dh_keypair() -> (Vec<u8>, Vec<u8>) {
        let mut secret_bytes = [0u8; 32];
        rand::thread_rng().fill(&mut secret_bytes);
        
        secret_bytes[0] &= 248;
        secret_bytes[31] &= 127;
        secret_bytes[31] |= 64;

        let public = x25519_dalek::x25519(secret_bytes, [9; 32]);
        (secret_bytes.to_vec(), public.to_vec())
    }

    pub fn compute_shared_secret(
        private_key: &[u8],
        public_key: &[u8],
    ) -> Result<Vec<u8>> {
        if private_key.len() != 32 || public_key.len() != 32 {
            return Err(crate::error::AirdropError::Crypto(
                "Invalid key size".to_string(),
            ));
        }

        let mut private_bytes = [0u8; 32];
        private_bytes.copy_from_slice(private_key);

        let mut public_bytes = [0u8; 32];
        public_bytes.copy_from_slice(public_key);

        let shared_secret = x25519_dalek::x25519(private_bytes, public_bytes);

        Ok(shared_secret.to_vec())
    }

    pub fn generate_confirmation_code(shared_secret: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        let result = hasher.finalize();

        format!("{:X}", result)
            .chars()
            .take(6)
            .collect()
    }

    pub fn hash_data(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:X}", hasher.finalize())
    }

    pub fn generate_nonce() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut nonce = vec![0u8; 32];
        rng.fill(&mut nonce[..]);
        nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dh_key_generation() {
        let (private, public) = CryptoManager::generate_dh_keypair();
        assert_eq!(private.len(), 32);
        assert_eq!(public.len(), 32);
    }

    #[test]
    fn test_shared_secret() {
        let (private1, public1) = CryptoManager::generate_dh_keypair();
        let (private2, public2) = CryptoManager::generate_dh_keypair();

        let secret1 = CryptoManager::compute_shared_secret(&private1, &public2).unwrap();
        let secret2 = CryptoManager::compute_shared_secret(&private2, &public1).unwrap();

        assert_eq!(secret1, secret2);
    }

    #[test]
    fn test_confirmation_code() {
        let secret = vec![1, 2, 3, 4, 5];
        let code = CryptoManager::generate_confirmation_code(&secret);
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = CryptoManager::generate_nonce();
        let nonce2 = CryptoManager::generate_nonce();
        
        assert_eq!(nonce1.len(), 32);
        assert_eq!(nonce2.len(), 32);
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_hash_data() {
        let data = b"test data";
        let hash = CryptoManager::hash_data(data);
        
        assert!(!hash.is_empty());
        assert!(hash.len() > 0);
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"consistent data";
        let hash1 = CryptoManager::hash_data(data);
        let hash2 = CryptoManager::hash_data(data);
        
        assert_eq!(hash1, hash2);
    }
}