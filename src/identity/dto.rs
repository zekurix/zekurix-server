use serde::Deserialize;

use super::{Issuer, Subject};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub issuer: Issuer,
    pub subject: Subject,
}
