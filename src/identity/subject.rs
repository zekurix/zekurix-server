#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Subject(String);
impl Subject {
    pub fn new(value: String) -> Self {
        Self(value)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
