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
#[allow(dead_code)]
pub struct DialogueApprobation;

impl DialogueApprobation {
    #[allow(dead_code)]
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