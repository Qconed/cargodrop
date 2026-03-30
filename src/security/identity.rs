//! Gestion de l'Identité Cryptographique
//!
//! Ce module gère la création, le stockage et la vérification des identités locales
//! de chaque pair. Chaque identité est composée d'une paire de clés ED25519 et
//! d'une empreinte digitale SHA256 qui permet l'authentification mutuelle.
//!
//! # Composants:
//! - **GestionnaireIdentite**: Gère les clés ED25519 locales
//! - **IdentitePair**: Représente l'identité publique d'un pair
//!
//! # Responsabilités:
//! - **Génération**: Crée une nouvelle paire ED25519 aléatoire au démarrage
//! - **Signatures**: Signe les données pour authentifier les messages
//! - **Vérification**: Valide les signatures d'autres pairs
//! - **Empreinte**: Génère une empreinte SHA256 (16 caractères = 64 bits)
//! - **Persistance**: Charge/sauvegarde les clés du disque
//!
//! # ED25519:
//! - Algorithme: Signature numérique Edwards Curve 25519
//! - Taille clé: 32 bytes (256 bits)
//! - Sécurité: 128 bits équivalents (très sûr)
//! - Vitesse: Rapide, même pour gros volumes
//!
//! # Empreinte Digitale:
//! - Calcul: SHA256(clé_publique)[..16]
//! - Format: 16 caractères hexadécimaux = 64 bits
//! - Affichage: Affiché à l'utilisateur pour vérification manuelle
//! - Exemple: "a85d75dc55641f63"
//!
//! # Flux d'Authentification:
//! 1. Chaque pair génère sa paire ED25519
//! 2. Pair A affiche son empreinte: "a85d75dc55641f63"
//! 3. Pair B affiche son empreinte: "d1e63e9478885ef0"
//! 4. Les empreintes sont vérifiées out-of-band (en personne)
//! 5. Les signatures authentifient tous les messages futurs
//!
//! # Sécurité:
//! -Génération aléatoire sécurisée (rand crate)
//! -Signature déterministe (reproducible)
//! -Empreinte collision-resistant (SHA256)
//! -Protection contre le spoofing d'identité
use sha2::{Sha256, Digest};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use std::error::Error;

/// Représente l'identité cryptographique d'un pair
#[derive(Debug, Clone)]
pub struct IdentitePair {
    pub cle_publique: Vec<u8>,
    pub empreinte: String,
    pub nom_appareil: String,
}

/// Gère la génération et vérification des identités cryptographiques
#[derive(Clone)]
pub struct GestionnaireIdentite {
    cle_signature_locale: SigningKey,
    cle_verification_locale: VerifyingKey,
}

impl GestionnaireIdentite {
    pub fn nouveau() -> Self {
        let mut secret_bytes = [0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        
        let cle_signature = SigningKey::from_bytes(&secret_bytes);
        let cle_verification = cle_signature.verifying_key();
        Self {
            cle_signature_locale: cle_signature,
            cle_verification_locale: cle_verification,
        }
    }
    
    pub fn depuis_cles(
        cle_signature_bytes: &[u8; 32],
        cle_verification_bytes: &[u8; 32],
    ) -> Result<Self, Box<dyn Error>> {
        let cle_signature = SigningKey::from_bytes(cle_signature_bytes);
        let cle_verification = VerifyingKey::from_bytes(cle_verification_bytes)?;
        Ok(Self {
            cle_signature_locale: cle_signature,
            cle_verification_locale: cle_verification,
        })
    }
    
    pub fn obtenir_cle_verification_locale(&self) -> Vec<u8> {
        self.cle_verification_locale.as_bytes().to_vec()
    }
    
    pub fn creer_empreinte(cle_publique: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cle_publique);
        let hash = hasher.finalize();
        
        format!("{:x}", hash)[..16].to_string()
    }
    
    pub fn creer_identite_locale(&self, nom_appareil: String) -> IdentitePair {
        let cle_pub_bytes = self.obtenir_cle_verification_locale();
        let empreinte = Self::creer_empreinte(&cle_pub_bytes);
        
        IdentitePair {
            cle_publique: cle_pub_bytes,
            empreinte,
            nom_appareil,
        }
    }
    
    pub fn signer(&self, donnees: &[u8]) -> Signature {
        
        self.cle_signature_locale.sign(donnees)
    }
    
    pub fn verifier_signature(
        cle_publique_pair: &[u8],
        donnees: &[u8],
        signature_bytes: &[u8; 64],
    ) -> Result<(), Box<dyn Error>> {
        let cle_verification = VerifyingKey::from_bytes(
            <&[u8; 32]>::try_from(&cle_publique_pair[..32])?
        )?;
        
        let signature = Signature::from_bytes(signature_bytes);
        cle_verification.verify_strict(donnees, &signature)?;
        
        Ok(())
    }
    
    pub fn creer_identifiant_court(empreinte: &str) -> String {
        // Prendre les 4 premiers caractères de l'empreinte
        empreinte[..4].to_string()
    }

    pub fn afficher_empreinte_locale(&self) {
        let empreinte = Self::creer_empreinte(self.obtenir_cle_verification_locale().as_slice());
        println!("\n╔════════════════════════════════════════╗");
        println!("║     VOTRE EMPREINTE DE SÉCURITÉ        ║");
        println!("║                                        ║");
        println!("║  Partagez ce code avec vos collègues   ║");
        println!("║  pour vérifier votre identité          ║");
        println!("║                                        ║");
        println!("║         {}         ║", empreinte);
        println!("║                                        ║");
        println!("╚════════════════════════════════════════╝\n");
    }
    
    pub fn get_cle_signature(&self) -> SigningKey {
        self.cle_signature_locale.clone()
    }
    
    pub fn get_cle_verification(&self) -> VerifyingKey {
        self.cle_verification_locale.clone()
    }
}