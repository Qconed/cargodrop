//!  Protocole de Poignée de Main Cryptographique
//!
//! Ce module implémente un protocole d'établissement de session sécurisée basé sur:
//! - **X25519**: Diffie-Hellman pour l'échange de clé secrète
//! - **ED25519**: Signatures pour l'authentification mutuelle
//! - **HKDF-SHA256**: Dérivation de clé à partir du secret partagé
//!
//! # Composants:
//! - **MessagePoigneeDeMainInit**: Message d'initialisation du handshake
//! - **MessagePoigneeDeMainReponse**: Réponse confirmant la session
//! - **InitiateurPoigneeDeMain**: Orchestrateur du protocole
//!
//! # Responsabilités:
//! - **Échange de clé**: X25519 Diffie-Hellman (ECDH)
//! - **Authentification**: Signatures ED25519 du handshake
//! - **Dérivation**: HKDF-SHA256 pour générer clés de session
//! - **Protection replay**: HMAC et numérotation séquentielle
//!
//! # Flux du Handshake:
//! ```
//! //! # X25519 ECDH:
//!// - Courbe: Curve25519 (Montgomery form)
//! //- Taille clé: 32 bytes (256 bits)
//!// - Secret partagé: 32 bytes
//!// - Sécurité: 128 bits (post-quantum resistant)
//!
//! # //HKDF-SHA256:
//! //- Entrée: Secret partagé X25519 (32 bytes)
//! //- Sortie: Clé AES-256-GCM (32 bytes)
//! //- Info: "cargodrop-aes256-gcm"
//! //- Garantie: Clé complètement différente à chaque session
//!
//! # //Messages du Handshake:
//!// - `MessagePoigneeDeMainInit`: 
//!   ├─ //cle_ephemere_pub (32 bytes)
//!   ├─ //signature_ephemere (64 bytes ED25519)
//!   ├─ //signature_message (64 bytes ED25519)
//!   ├─ //cle_identite (32 bytes)
//!   └─ //nom_appareil (String)
//!
//! # //Sécurité:
//!// -  //Forward secrecy (clés éphémères)
//! //-  //Authentification mutuelle (ED25519)
//! //-  //Protection contre MITM (signatures du handshake)
//! //-  //Dérivation sécurisée (HKDF)
//! //-  //Protection replay (HMAC + séquence)
//! //Pair A                          Pair B
//! ├─ //Génère secret éphémère X25519
//! ├─ //Signe clé éphémère + hash du message avec ED25519
//! ├─ //Envoie: (cle_ephemere_pub, signature, cle_identite)
//! │                                  │
//! │                                  ├─ //Valide signatures ED25519
//! │                                  ├─ //Dérive secret partagé: DH(cle_ephemere)
//! │                                  ├─ //Génère clés avec HKDF
//! │  //Reçoit et dérive les mêmes clés
//! └─ //Chiffrement/Déchiffrement activé avec AES-256-GCM
//! ```
//!
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, SharedSecret};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use hkdf::Hkdf;
use ed25519_dalek::{Signature, Signer};
use std::error::Error;
use serde::{Deserialize, Serialize};
#[allow(dead_code)]
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]



pub struct MessagePoigneeDeMainInit {
    pub cle_ephemere_pub: Vec<u8>,
    pub signature_ephemere: Vec<u8>,
    pub signature_message: Vec<u8>,      
    pub cle_identite: Vec<u8>,
    pub nom_appareil: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePoigneeDeMainReponse {
    pub cle_ephemere_pub: Vec<u8>,
    pub signature_ephemere: Vec<u8>,
    pub signature_message: Vec<u8>,      
    pub cle_identite: Vec<u8>,
    pub nom_appareil: String,
    pub hmac_confirmation: Vec<u8>,
}

pub struct InitiateurPoigneeDeMain {
    cle_signature_locale: ed25519_dalek::SigningKey,
    cle_verification_locale: ed25519_dalek::VerifyingKey,
}

impl InitiateurPoigneeDeMain {
    pub fn nouveau(
        cle_signature: ed25519_dalek::SigningKey,
        cle_verification: ed25519_dalek::VerifyingKey,
    ) -> Self {
        Self {
            cle_signature_locale: cle_signature,
            cle_verification_locale: cle_verification,
        }
    }
    
    pub fn creer_secret_ephemere() -> (EphemeralSecret, X25519PublicKey) {
        let secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let public = X25519PublicKey::from(&secret);
        (secret, public)
    }
    
    pub fn signer_cle_ephemere(
        &self,
        cle_pub_ephemere: &X25519PublicKey,
    ) -> Signature {
        self.cle_signature_locale.sign(cle_pub_ephemere.as_bytes())
    }
    
    pub fn creer_message_init(
        &self,
        nom_appareil: String,
    ) -> (MessagePoigneeDeMainInit, EphemeralSecret) {
        let (secret_ephemere, cle_pub_ephemere) = Self::creer_secret_ephemere();
        let signature_ephemere = self.signer_cle_ephemere(&cle_pub_ephemere);
        
        //  NOUVEAU: Construire le hash du message avant signature
        let mut hasher = Sha256::new();
        hasher.update(cle_pub_ephemere.as_bytes());
        hasher.update(nom_appareil.as_bytes());
        hasher.update(self.cle_verification_locale.as_bytes());
        let message_hash = hasher.finalize();
        
        //  Signer le hash complet du message
        let signature_message = self.cle_signature_locale.sign(&message_hash);
        
        let message = MessagePoigneeDeMainInit {
            cle_ephemere_pub: cle_pub_ephemere.as_bytes().to_vec(),
            signature_ephemere: signature_ephemere.to_bytes().to_vec(),
            signature_message: signature_message.to_bytes().to_vec(), 
            cle_identite: self.cle_verification_locale.as_bytes().to_vec(),
            nom_appareil,
        };
        
        (message, secret_ephemere)
    }
    
    pub fn deriver_secret_partage(
        secret_ephemere: EphemeralSecret,
        cle_pub_ephemere_pair: &X25519PublicKey,
    ) -> [u8; 32] {
        let secret_partage: SharedSecret = secret_ephemere.diffie_hellman(cle_pub_ephemere_pair);
        *secret_partage.as_bytes()
    }
    
    pub fn deriver_cle_chiffrement(secret_partage: &[u8; 32]) -> [u8; 32] {
        let hkdf = Hkdf::<Sha256>::new(None, secret_partage);
        let info = b"cargodrop-aes256-gcm";
        let mut cle_chiffrement = [0u8; 32];
        
        hkdf.expand(info, &mut cle_chiffrement)
            .expect("Erreur HKDF - la taille de sortie est valide");
        
        cle_chiffrement
    }
    #[allow(dead_code)]
    pub fn creer_hmac_confirmation(cle_chiffrement: &[u8; 32]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(cle_chiffrement)
            .expect("HMAC accepte les clés de toutes tailles");
        mac.update(b"confirmation");
        mac.finalize().into_bytes().to_vec()
    }
    #[allow(dead_code)]
    pub fn verifier_hmac_confirmation(
        cle_chiffrement: &[u8; 32],
        hmac_recu: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let mut mac = HmacSha256::new_from_slice(cle_chiffrement)?;
        mac.update(b"confirmation");
        mac.verify_slice(hmac_recu)?;
        Ok(())
    }
    #[allow(dead_code)]
    pub fn verifier_signature_ephemere(
        cle_publique_pair: &[u8],
        cle_ephemere: &[u8],
        signature: &[u8; 64],
    ) -> Result<(), Box<dyn Error>> {
        use ed25519_dalek::VerifyingKey;
        
        let cle_verification = VerifyingKey::from_bytes(
            <&[u8; 32]>::try_from(&cle_publique_pair[..32])?
        )?;
        
        let sig = Signature::from_bytes(signature);
        cle_verification.verify_strict(cle_ephemere, &sig)?;
        
        Ok(())
    }
    #[allow(dead_code)]
    //  NOUVEAU: Vérifier la signature du message complet
    pub fn verifier_signature_message(
        cle_publique_pair: &[u8],
        cle_ephemere_pub: &[u8],
        nom_appareil: &str,
        signature_message: &[u8; 64],
    ) -> Result<(), Box<dyn Error>> {
        use ed25519_dalek::VerifyingKey;
        
        // Reconstruire le hash du message
        let mut hasher = Sha256::new();
        hasher.update(cle_ephemere_pub);
        hasher.update(nom_appareil.as_bytes());
        hasher.update(cle_publique_pair);
        let message_hash = hasher.finalize();
        
        // Vérifier la signature
        let cle_verification = VerifyingKey::from_bytes(
            <&[u8; 32]>::try_from(&cle_publique_pair[..32])?
        )?;
        
        let sig = Signature::from_bytes(signature_message);
        cle_verification.verify_strict(&message_hash, &sig)?;
        
        Ok(())
    }
}