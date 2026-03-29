//!  Gestion des Contacts de Confiance
//!
//! Ce module gère la persistance et vérification des contacts de confiance.
//! Un contact de confiance est un pair dont on a vérifié manuellement l'empreinte
//! et avec qui on a établi une relation de confiance durable.
//!
//! # Composants:
//! - **ContactDeConfiance**: Structure représentant un contact approuvé
//! - **GestionnaireContacts**: Gère la persistance (chargement/sauvegarde JSON)
//! - **DialogueApprobation**: Interface interactive pour approuver nouveaux pairs
//!
//! # Responsabilités:
//! - **Persistance**: Sauvegarde contacts dans ~/.cargodrop/security/contacts_de_confiance.json
//! - **Vérification**: Vérifie qu'un pair est dans la liste de confiance
//! - **Audit**: Enregistre quand/comment on a approuvé chaque contact
//! - **Dialogue**: Demande confirmation interactive pour nouveaux pairs
//!
//! # Structure ContactDeConfiance:
//! ```json
//! {
//!   "nom_appareil": "MacBook-A",
//!   "empreinte_cle_publique": "a85d75dc55641f63",
//!   "cle_publique": [bytes...],
//!   "confiance_depuis": 1234567890,
//!   "vu_dernierement": 1234567900
//! }
//! ```
//!
//! //! # Flux d'Approbation d'un Nouveau Pair:
//! ```
//! //1. Pair inconnu envoie message avec empreinte "a85d75dc55641f63"
//! //2. DialogueApprobation affiche:
//!    ┌─────────────────────────────────┐
//!    │  //🔐 NOUVEAU PAIR DÉTECTÉ        │
//!    │  //Nom: MacBook-B                 │
//!    │  //Empreinte: a85d75dc55641f63    │
//!    │  //⚠️  Vérifiez avec le pair!     │
//!    │  //O) Faire confiance             │
//!    │  //N) Refuser                     │
//!    └─────────────────────────────────┘
//! //3. Utilisateur vérifie empreinte en personne ou par appel
//! //4. Si OK: Ajoute à contacts_de_confiance.json
//! //5. Futures communications avec ce pair sont authentifiées
//! ```
//!
//! # Persistance:
//! - **Chemin**: ~/.cargodrop/security/contacts_de_confiance.json
//! - **Format**: JSON array de ContactDeConfiance
//! - **Permissions**: Fichier lisible en local (sécurisé par OS)
//! - **Sauvegarde**: Automatique après chaque approbation
//!
//! # Vérification de Confiance:
//! - Match: nom_appareil + empreinte_cle_publique
//! - Utilisé pour: Valider signatures du peer dans handshake
//! - Fail: Rejette la connexion avec MITM potentiel
//!
//! # Audit Trail:
//! - `confiance_depuis`: Timestamp de première approbation
//! - `vu_dernierement`: Timestamp du dernier contact actif
//! - Permet de: Détecter changements d'identité suspects
//!
//! # Sécurité:
//! - Vérification out-of-band (en personne ou appel)
//! - Empreinte courte (16 chars) mais impossible à mémoriser sans erreur
//! - Persistance locale (confiance durable entre sessions)
//! - Protection contre MITM (si vérification faite correctement)
//! -  Audit complet (quand/comment approuvé chaque contact)
//!
//! # Limitation Connue:
//!  Empreinte 16 chars = 64 bits = ~4.3 milliards de possibilités
//!  Suffisant pour prévenir les collisions accidentelles
//!  Attaquant très motivé pourrait brute-force (rare en pratique)
//!  Vérification humaine ("c'est vraiment lui?") est la défense principale

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDeConfiance {
    pub nom_appareil: String,
    pub empreinte_cle_publique: String,
    pub cle_publique: Vec<u8>,
    pub confiance_depuis: i64,
    pub vu_dernierement: i64,
}

pub struct GestionnaireContacts {
    fichier_contacts: PathBuf,
}

impl GestionnaireContacts {
    pub fn nouveau(repertoire_config: &str) -> Result<Self, Box<dyn Error>> {
        let repertoire = PathBuf::from(repertoire_config);
        fs::create_dir_all(&repertoire)?;
        
        let fichier_contacts = repertoire.join("contacts_de_confiance.json");
        
        Ok(Self {
            fichier_contacts,
        })
    }
    
    pub fn charger_contacts(&self) -> Result<Vec<ContactDeConfiance>, Box<dyn Error>> {
        if !self.fichier_contacts.exists() {
            return Ok(Vec::new());
        }
        
        let donnees = fs::read_to_string(&self.fichier_contacts)?;
        let contacts = serde_json::from_str(&donnees)?;
        Ok(contacts)
    }
    
    pub fn ajouter_contact(&self, contact: ContactDeConfiance) -> Result<(), Box<dyn Error>> {
        let mut contacts = self.charger_contacts().unwrap_or_default();
        
        contacts.retain(|c| c.nom_appareil != contact.nom_appareil);
        contacts.push(contact);
        
        let json = serde_json::to_string_pretty(&contacts)?;
        fs::write(&self.fichier_contacts, json)?;
        
        Ok(())
    }
    
    pub fn est_de_confiance(&self, nom_appareil: &str, empreinte: &str) -> bool {
        self.charger_contacts()
            .unwrap_or_default()
            .iter()
            .any(|c| {
                c.nom_appareil == nom_appareil && c.empreinte_cle_publique == empreinte
            })
    }
    
    pub fn obtenir_contact(&self, nom_appareil: &str) -> Result<Option<ContactDeConfiance>, Box<dyn Error>> {
        let contacts = self.charger_contacts()?;
        Ok(contacts.into_iter().find(|c| c.nom_appareil == nom_appareil))
    }
    
    pub fn mettre_a_jour_vu_dernierement(
        &self,
        nom_appareil: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut contacts = self.charger_contacts()?;
        
        if let Some(contact) = contacts.iter_mut().find(|c| c.nom_appareil == nom_appareil) {
            contact.vu_dernierement = chrono::Local::now().timestamp();
            
            let json = serde_json::to_string_pretty(&contacts)?;
            fs::write(&self.fichier_contacts, json)?;
        }
        
        Ok(())
    }
    
    pub fn afficher_contacts(&self) -> Result<(), Box<dyn Error>> {
        let contacts = self.charger_contacts()?;
        
        if contacts.is_empty() {
            println!(" Aucun contact de confiance trouvé.");
            return Ok(());
        }
        
        println!("\n╔════════════════════════════════════════════════════╗");
        println!("║         CONTACTS DE CONFIANCE                      ║");
        println!("╠════════════════════════════════════════════════════╣");
        
        for contact in contacts {
            println!("║ 📱 {}                        ║", contact.nom_appareil);
            println!("║    Empreinte: {}               ║", contact.empreinte_cle_publique);
            println!("╟────────────────────────────────────────────────────╢");
        }
        
        println!("╚════════════════════════════════════════════════════╝\n");
        
        Ok(())
    }
}

/// Dialogue interactif pour approuver un nouveau pair
pub struct DialogueApprobation;

impl DialogueApprobation {
    pub fn demander_approbation(
        nom_appareil: &str,
        empreinte: &str,
    ) -> Result<bool, Box<dyn Error>> {
        println!("\n╔════════════════════════════════════════════════════╗");
        println!("║              🔐 NOUVEAU PAIR DÉTECTÉ              ║");
        println!("╠════════════════════════════════════════════════════╣");
        println!("║                                                    ║");
        println!("║  Nom: {}                              ║", format!("{:<40}", nom_appareil));
        println!("║  Empreinte: {}                   ║", empreinte);
        println!("║                                                    ║");
        println!("║  ⚠️  Vérifiez cette empreinte avec votre collègue! ║");
        println!("║                                                    ║");
        println!("╠════════════════════════════════════════════════════╣");
        println!("║  O) Faire confiance                                ║");
        println!("║  N) Refuser (défaut)                               ║");
        println!("╚════════════════════════════════════════════════════╝\n");
        
        print!("Votre choix (O/N): ");
        io::stdout().flush()?;
        
        let mut reponse = String::new();
        io::stdin().read_line(&mut reponse)?;
        
        Ok(reponse.trim().to_lowercase() == "o")
    }
}