use cargodrop::security::SecureSession;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Test de session sécurisée...\n");
    
    // Créer une session
    let session = SecureSession::new("test-device".to_string()).await?;
    
    println!("✅ Session créée avec succès!");
    println!("✅ Empreinte affichée ci-dessus");
    
    Ok(())
}