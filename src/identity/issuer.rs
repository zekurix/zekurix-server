#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Issuer(String);
impl Issuer {
    pub fn new(value: String) -> Self {
        Self(value)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
