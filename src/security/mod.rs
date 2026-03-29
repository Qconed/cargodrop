// src/security/mod.rs
pub mod identity;
pub mod handshake;
pub mod encryption;
pub mod contact;

pub use identity::GestionnaireIdentite;
pub use handshake::InitiateurPoigneeDeMain;
pub use encryption::{CipherManager, DecipherManager};
pub use contact::GestionnaireContacts;

use std::error::Error;
use dirs::home_dir;

/// 🔐 Gestionnaire de sécurité complet
pub struct SecureSession {
    pub identite: GestionnaireIdentite,
    pub contacts: GestionnaireContacts,
    pub cipher: Option<CipherManager>,
    pub decipher: Option<DecipherManager>,
}

impl SecureSession {
    /// Initialiser une session sécurisée
    pub async fn new(nom_appareil: String) -> Result<Self, Box<dyn Error>> {
        // Créer le répertoire de config
        let config_dir = home_dir()
            .ok_or("❌ Impossible de trouver le répertoire home")?
            .join(".cargodrop")
            .join("security");
        
        tokio::fs::create_dir_all(&config_dir).await?;
        
        // Initialiser l'identité
        let identite = GestionnaireIdentite::nouveau();
        
        // Initialiser les contacts
        let contacts = GestionnaireContacts::nouveau(config_dir.to_str().ok_or("Chemin invalide")?)?;
        
        // Afficher l'empreinte au démarrage
        identite.afficher_empreinte_locale();
        println!("✅ Session sécurisée initialisée: {}\n", nom_appareil);
        
        Ok(Self {
            identite,
            contacts,
            cipher: None,
            decipher: None,
        })
    }
    
    /// Établir un handshake de sécurité avec un pair
    pub fn initier_handshake(&self) -> Result<(Vec<u8>, [u8; 32]), Box<dyn Error>> {
        let handshake = InitiateurPoigneeDeMain::nouveau(
            self.identite.get_cle_signature(),
            self.identite.get_cle_verification(),
        );
        
        // Créer les secrets éphémères
        let (secret_ephemere, cle_pub_ephemere) = InitiateurPoigneeDeMain::creer_secret_ephemere();
        
        // Créer le message d'initialisation
        let (message_init, _) = handshake.creer_message_init(
            "cargodrop-client".to_string(),
        );
        
        // Sérialiser le message
        let message_bytes = serde_json::to_vec(&message_init)?;
        
        // Dériver le secret partagé
        let secret_partage = InitiateurPoigneeDeMain::deriver_secret_partage(
            secret_ephemere,
            &cle_pub_ephemere,
        );
        
        // Dériver la clé de chiffrement
        let cle_chiffrement = InitiateurPoigneeDeMain::deriver_cle_chiffrement(&secret_partage);
        
        Ok((message_bytes, cle_chiffrement))
    }
    
    /// Activer le chiffrement avec une clé
    pub fn activer_chiffrement(&mut self, cle_chiffrement: &[u8; 32]) {
        self.cipher = Some(CipherManager::nouveau(cle_chiffrement));
        self.decipher = Some(DecipherManager::nouveau(cle_chiffrement));
    }
    
    /// Chiffrer des données
    pub fn chiffrer(&mut self, donnees: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        self.cipher
            .as_mut()
            .ok_or("❌ Chiffrement non activé")?
            .chiffrer_bloc(donnees)
    }
    
    /// Déchiffrer des données
    pub fn dechiffrer(&mut self, donnees_chiffrees: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        self.decipher
            .as_mut()
            .ok_or("❌ Déchiffrement non activé")?
            .dechiffrer_bloc(donnees_chiffrees)
    }
}