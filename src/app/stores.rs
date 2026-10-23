use std::sync::Arc;

use crate::identity::{VerificationKeySet, VerificationKeyStore};

pub struct Stores {
    pub verification_key: Arc<VerificationKeyStore>,
}

impl Stores {
    pub fn new() -> Self {
        let verification_key = Arc::new(VerificationKeyStore::new(VerificationKeySet::default()));

        Self { verification_key }
    }
}
