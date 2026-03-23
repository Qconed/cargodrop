use crate::error::Result;
use crate::models::Message;
use std::path::Path;

pub struct FileTransfer;

impl FileTransfer {
    pub async fn send_file(path: &Path) -> Result<Vec<u8>> {
        let data = std::fs::read(path)
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
        
        println!("📤 File size: {} bytes", data.len());
        Ok(data)
    }

    pub async fn receive_file(data: Vec<u8>, path: &Path) -> Result<()> {
        std::fs::write(path, data)
            .map_err(|e| crate::error::AirdropError::Transfer(e.to_string()))?;
        
        println!("📥 File saved to {:?}", path);
        Ok(())
    }
}