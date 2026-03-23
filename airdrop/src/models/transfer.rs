use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TransferSession {
    pub session_id: Uuid,
    pub remote_device_id: Uuid,
    pub confirmation_code: String,
    pub is_authenticated: bool,
    pub shared_secret: Option<Vec<u8>>,
}

impl TransferSession {
    pub fn new(remote_device_id: Uuid) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            remote_device_id,
            confirmation_code: String::new(),
            is_authenticated: false,
            shared_secret: None,
        }
    }

    pub fn set_shared_secret(&mut self, secret: Vec<u8>) {
        self.shared_secret = Some(secret);
    }

    pub fn set_confirmation_code(&mut self, code: String) {
        self.confirmation_code = code;
    }

    pub fn authenticate(&mut self) {
        self.is_authenticated = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_session() {
        let device_id = Uuid::new_v4();
        let mut session = TransferSession::new(device_id);
        
        session.set_shared_secret(vec![1, 2, 3, 4]);
        assert!(session.shared_secret.is_some());
        
        session.authenticate();
        assert!(session.is_authenticated);
    }
}