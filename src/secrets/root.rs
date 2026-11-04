use crate::error::Result;

use super::DatabaseSecrets;

#[derive(Default)]
pub struct Secrets {
    pub database: DatabaseSecrets,
}

impl Secrets {
    pub fn env_vars() -> Vec<&'static str> {
        let mut vars = Vec::new();

        vars.extend(DatabaseSecrets::env_vars());

        vars
    }

    pub fn load() -> Result<Self> {
        Ok(Self {
            database: DatabaseSecrets::load()?,
        })
    }
}
