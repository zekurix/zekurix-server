pub mod database;

#[derive(Default)]
pub struct Secrets {
    pub database: database::Secrets,
}

impl Secrets {
    pub fn env_vars() -> Vec<&'static str> {
        let mut vars = Vec::new();

        vars.extend(database::Secrets::env_vars());

        vars
    }

    pub fn load() -> Self {
        Self {
            database: database::Secrets::load(),
        }
    }
}
