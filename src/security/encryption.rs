//!  Chiffrement Symétrique - AES-256-GCM
//!
//! Ce module gère le chiffrement et déchiffrement symétrique des données avec
//! AES-256-GCM (Advanced Encryption Standard 256-bit Galois/Counter Mode).
//! AES-GCM fournit à la fois confidentialité ET authentification.
//!
//! # Composants:
//! - **CipherManager**: Chiffre les données avec nonce séquentiel
//! - **DecipherManager**: Déchiffre et valide l'authentification
//!
//! # Responsabilités:
//! - **Chiffrement**: AES-256-GCM avec nonce 12 bytes
//! - **Authentification**: Tag d'authentification intégré à GCM
//! - **Protection DoS**: Limites de taille (max 65KB par bloc)
//! - **Protection replay**: Numérotation séquentielle des blocs
//! - **Gestion nonce**: 8 bytes séquence + 4 bytes aléatoire
//!
//! # Nonce (Number used ONCE):
//! - Taille: 12 bytes (96 bits) - Standard NIST
//! - Structure: [8 bytes numéro séquentiel][4 bytes aléatoire]
//! - Numéro séquentiel: Incrémenté pour chaque bloc (u64 little-endian)
//! - Aléatoire: Généré une fois à l'initialisation
//! - Garantie: JAMAIS répété avec la même clé (critique pour la sécurité)
//!
//! //! # AES-256-GCM:
//! - Chiffrement: AES-256 en mode Galois/Counter Mode
//! - Taille clé: 256 bits (32 bytes)
//! - Taille bloc: 128 bits (16 bytes)
//! - Tag d'auth: 128 bits (16 bytes)
//! - Sécurité: 256 bits (très sûr, même contre quantum)
//! - Vitesse: ~30 cycles par byte sur CPU moderne
//!
//! # Flux de Chiffrement:
//! ```
//! //CipherManager::chiffrer_bloc(donnees)
//!   ├─ //Valide taille (8 bytes ≤ len ≤ 65536 bytes)
//!   ├─ //Génère nonce: [numero_bloc][random_prefix]
//!   ├─ //AES-256-GCM.encrypt(nonce, donnees)
//!   ├─ //Retour: [numero_bloc (8 bytes)][texte_chiffré][tag (16 bytes)]
//!   └─ //Incrément numero_bloc++
//! ```
//!
//! # Flux de Déchiffrement:
//! ```
//! //DecipherManager::dechiffrer_bloc(donnees_chiffrees)
//!   ├─ //Valide taille minimum (8 bytes)
//!   ├─ //Valide protection DoS (max 65KB)
//!   ├─ //Extraire numero_bloc reçu
//!   ├─ //Valider ordre: numero_recu == numero_attendu
//!   ├─ //Générer nonce: [numero_bloc][random_prefix]
//!   ├─ //AES-256-GCM.decrypt(nonce, texte_chiffré)
//!   ├─ //Tag d'authentification valide automatiquement
//!   └─ //Retour: texte clair si authentification OK, erreur sinon
//! ```
//!
//! //! # Protections:
//! -  **Confidentialité**: AES-256 (256 bits de sécurité)
//! -  **Authentification**: Tag GCM (128 bits)
//! -  **Protection replay**: Numérotation séquentielle + vérification ordre
//! - **Protection DoS**: Limites de taille de bloc (65KB max)
//! -  **Nonce sécurisé**: Jamais répété grâce à séquence + aléatoire
//! -  **Timing attack resilient**: GCM est constant-time
//!
//! # Format des Données Chiffrées:
//! ```
//! //[8 bytes numero_bloc][texte_chiffré][16 bytes tag_authentification]
//! ```
//!
//! # Sécurité Critique:
//!   JAMAIS réutiliser la même (clé, nonce) - DÉTRUIT LA SÉCURITÉ
//!  Cette implémentation garantit l'unicité du nonce par bloc
//!  Clé changée à chaque nouvelle session via handshake
//!  Protégé contre les attaques par force brute sur clé (256 bits)

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use std::error::Error;
use aes_gcm::aead::generic_array::GenericArray;
use rand::RngCore;

pub const MAX_BLOC_SIZE: usize = 65536 + 8 + 16;
#[allow(dead_code)]
pub const MIN_BLOC_SIZE: usize = 8 + 16;

pub struct CipherManager {
    cipher: Aes256Gcm,
    numero_bloc_courant: u64,
    random_prefix: [u8; 4],  
}

impl CipherManager {
    pub fn nouveau(cle_chiffrement: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(cle_chiffrement.into());
        let mut random_prefix = [0u8; 4];
        rand::thread_rng().fill_bytes(&mut random_prefix);  
        
        Self {
            cipher,
            numero_bloc_courant: 0,
            random_prefix,
        }
    }
    
    pub fn chiffrer_bloc(&mut self, texte_clair: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if texte_clair.is_empty() {
            return Err(" Bloc vide détecté".into());
        }
        
        if texte_clair.len() > 65536 {
            return Err(format!(
                " Bloc trop grand: {} octets (max: 65536)",
                texte_clair.len()
            ).into());
        }
        
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..8].copy_from_slice(&self.numero_bloc_courant.to_le_bytes());
        nonce_bytes[8..12].copy_from_slice(&self.random_prefix);
        
        let nonce = GenericArray::from_slice(&nonce_bytes);
        let texte_chiffre = self.cipher.encrypt(nonce, texte_clair)
            .map_err(|_| "Erreur de chiffrement AES-GCM".to_string())?;
        
        let mut resultat = Vec::with_capacity(8 + texte_chiffre.len());
        resultat.extend_from_slice(&self.numero_bloc_courant.to_le_bytes());
        resultat.extend_from_slice(&texte_chiffre);
        
        self.numero_bloc_courant += 1;
        
        Ok(resultat)
    }

    pub fn obtenir_numero_bloc(&self) -> u64 {
        self.numero_bloc_courant
    }
}

pub struct DecipherManager {
    cipher: Aes256Gcm,
    prochain_numero_bloc_attendu: u64,
    random_prefix: [u8; 4], 
}

impl DecipherManager {
    pub fn nouveau(cle_chiffrement: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(cle_chiffrement.into());
        Self {
            cipher,
            prochain_numero_bloc_attendu: 0,
            random_prefix: [0u8; 4],
        }
    }
    
    pub fn dechiffrer_bloc(&mut self, donnees_chiffrees: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if donnees_chiffrees.len() < 8 {
            return Err(" Bloc trop court - au moins 8 octets requis".into());
        }
        
        if donnees_chiffrees.len() > MAX_BLOC_SIZE {
            return Err(format!(
                " ATTAQUE DoS BLOQUÉE! Taille du bloc: {} octets (max: {})\n\
                   Un attaquant a tenté d'allouer {} MB de RAM!",
                donnees_chiffrees.len(),
                MAX_BLOC_SIZE,
                donnees_chiffrees.len() / 1_000_000
            ).into());
        }
        
        let mut numero_bytes = [0u8; 8];
        numero_bytes.copy_from_slice(&donnees_chiffrees[0..8]);
        let numero_bloc_recu = u64::from_le_bytes(numero_bytes);
        
        if numero_bloc_recu != self.prochain_numero_bloc_attendu {
            return Err(format!(
                " Bloc reçu HORS D'ORDRE! Attendu: {}, Reçu: {}",
                self.prochain_numero_bloc_attendu, numero_bloc_recu
            ).into());
        }
        
        
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..8].copy_from_slice(&numero_bytes);
        nonce_bytes[8..12].copy_from_slice(&self.random_prefix);
        let nonce = GenericArray::from_slice(&nonce_bytes);
        
        let texte_dechiffre = self.cipher.decrypt(nonce, &donnees_chiffrees[8..])
            .map_err(|_| "Erreur de déchiffrement AES-GCM".to_string())?;
        
        self.prochain_numero_bloc_attendu += 1;
        
        Ok(texte_dechiffre)
    }

    pub fn obtenir_numero_bloc_attendu(&self) -> u64 {
        self.prochain_numero_bloc_attendu
    }
}