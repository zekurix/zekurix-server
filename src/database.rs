use anyhow::Result;

pub struct Database {}

impl Database {
    pub async fn connect() -> Result<Self> {
        Ok(Self {})
    }
}
