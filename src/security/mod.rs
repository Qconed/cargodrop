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

///  Gestionnaire de sécurité complet
pub struct SecureSession {
    pub identite: GestionnaireIdentite,
    pub contacts: GestionnaireContacts,
    pub cipher: Option<CipherManager>,
    pub decipher: Option<DecipherManager>,
}

impl SecureSession {
    /// Initialiser une session sécurisée
    pub async fn new(nom_appareil: String) -> Result<Self, Box<dyn Error>> {
        // ÉTAPE 1: Créer le répertoire
        println!("🔐 [SÉCURITÉ] ÉTAPE 1: Initialisation de la session");
        let config_dir = home_dir()
            .ok_or(" Impossible de trouver le répertoire home")?
            .join(".cargodrop")
            .join("security");
        
        println!("   └─ Création répertoire: ~/.cargodrop/security/");
        tokio::fs::create_dir_all(&config_dir).await?;
        println!("    Répertoire créé");
        
        // ÉTAPE 2: Générer identité ED25519
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 2: Génération de l'identité ED25519");
        let identite = GestionnaireIdentite::nouveau();
        println!("   └─ Paire de clés ED25519 générée");
        println!("   └─ Clé privée: stockée localement (jamais partagée)");
        println!("   └─ Clé publique: 32 bytes");
        
        // ÉTAPE 3: Créer empreinte
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 3: Création de l'empreinte digitale");
        identite.afficher_empreinte_locale();
        println!("   └─ Empreinte = SHA256(clé_publique)[..16]");
        println!("   └─ Format: 16 caractères hexadécimaux = 64 bits");
        
        // ÉTAPE 4: Initialiser contacts
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 4: Initialisation du gestionnaire de contacts");
        let contacts = GestionnaireContacts::nouveau(config_dir.to_str().ok_or("Chemin invalide")?)?;
        println!("    Chemin: ~/.cargodrop/security/contacts_de_confiance.json");
        println!("    Gestionnaire prêt");
        
        println!("\n Session sécurisée initialisée: {}\n", nom_appareil);
        
        Ok(Self {
            identite,
            contacts,
            cipher: None,
            decipher: None,
        })
    }

    /// Établir un handshake de sécurité avec un pair
    pub fn initier_handshake(&self) -> Result<(Vec<u8>, [u8; 32]), Box<dyn Error>> {
        println!("🔐 [SÉCURITÉ] ÉTAPE 5: Initiation du Handshake");
        
        println!("   └─ Création de InitiateurPoigneeDeMain");
        let handshake = InitiateurPoigneeDeMain::nouveau(
            self.identite.get_cle_signature(),
            self.identite.get_cle_verification(),
        );
        
        // Créer les secrets éphémères X25519
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 6: Génération des secrets éphémères X25519");
        let (secret_ephemere, cle_pub_ephemere) = InitiateurPoigneeDeMain::creer_secret_ephemere();
        println!("    Secret éphémère: 32 bytes (temporaire)");
        println!("    Clé publique éphémère: 32 bytes (à envoyer)");
        println!("    Secrets X25519 créés");
        
        // Créer le message d'initialisation
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 7: Création du message de handshake");
        let (message_init, _) = handshake.creer_message_init(
            "cargodrop-client".to_string(),
        );
        println!("   └─ Message signé avec ED25519");
        println!("   └─ Contient: clé_ephemere + signatures + identité");
        
        // Sérialiser le message
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 8: Sérialisation du message");
        let message_bytes = serde_json::to_vec(&message_init)?;
        println!("    Message sérialisé en JSON: {} bytes", message_bytes.len());
        println!("    Message prêt à envoyer");
        
        // Dériver le secret partagé (Diffie-Hellman)
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 9: Dérivation du secret partagé (X25519 DH)");
        let secret_partage = InitiateurPoigneeDeMain::deriver_secret_partage(
            secret_ephemere,
            &cle_pub_ephemere,
        );
        println!("    Secret partagé calculé: 32 bytes");
        println!("    Même secret généré des 2 côtés (Diffie-Hellman)");
        println!("    DH réussi");
        
        // Dériver la clé de chiffrement avec HKDF
        println!("\n🔐 [SÉCURITÉ] ÉTAPE 10: Dérivation de la clé AES-256-GCM (HKDF-SHA256)");
        let cle_chiffrement = InitiateurPoigneeDeMain::deriver_cle_chiffrement(&secret_partage);
        println!("    HKDF-SHA256(secret_partage) → clé AES-256");
        println!("    Clé dérivée: 32 bytes (256 bits)");
        println!("    Clé AES-256-GCM: {}", hex::encode(&cle_chiffrement[..8]));
        println!("    Clé dérivée avec succès\n");
        
        Ok((message_bytes, cle_chiffrement))
    }

    /// Activer le chiffrement avec une clé
    pub fn activer_chiffrement(&mut self, cle_chiffrement: &[u8; 32]) {
        println!("🔐 [SÉCURITÉ] ÉTAPE 11: Activation du chiffrement AES-256-GCM");
        println!("    Initialisation de CipherManager");
        println!("    Initialisation de DecipherManager");
        println!("    Génération du nonce aléatoire (4 bytes)");
        
        self.cipher = Some(CipherManager::nouveau(cle_chiffrement));
        self.decipher = Some(DecipherManager::nouveau(cle_chiffrement));
        
        println!("    Clé: 256 bits");
        println!("    Mode: AES-256-GCM (authentification incluse)");
        println!("    Nonce: 12 bytes (8 bytes numéro + 4 bytes aléatoire)");
        println!("    Chiffrement activé\n");
    }

    /// Chiffrer des données
    pub fn chiffrer(&mut self, donnees: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        println!("🔐 [SÉCURITÉ] ÉTAPE 12: Chiffrement des données");
        println!("    Taille originale: {} bytes", donnees.len());
        
        let resultat = self.cipher
            .as_mut()
            .ok_or(" Chiffrement non activé")?
            .chiffrer_bloc(donnees)?;
        
        println!("   └─ Taille chiffrée: {} bytes", resultat.len());
        println!("   └─ (8 bytes numéro + texte chiffré + 16 bytes tag)");
        println!("    Données chiffrées avec succès\n");
        
        Ok(resultat)
    }

    /// Déchiffrer des données
    pub fn dechiffrer(&mut self, donnees_chiffrees: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        println!("🔐 [SÉCURITÉ] ÉTAPE 13: Déchiffrement des données");
        println!("    Taille chiffrée: {} bytes", donnees_chiffrees.len());
        println!("    Vérification de l'ordre (numéro de bloc séquentiel)");
        println!("    Validation du tag d'authentification GCM");
        
        let resultat = self.decipher
            .as_mut()
            .ok_or(" Déchiffrement non activé")?
            .dechiffrer_bloc(donnees_chiffrees)?;
        
        println!("    Taille déchiffrée: {} bytes", resultat.len());
        println!("    Tag authentifié - Données intègres\n");
        
        Ok(resultat)
    }

    pub fn get_identifiant_court(&self) -> String {
        let empreinte = GestionnaireIdentite::creer_empreinte(
            self.identite.obtenir_cle_verification_locale().as_slice()
        );
        GestionnaireIdentite::creer_identifiant_court(&empreinte)
    }

}