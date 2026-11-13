use super::{RoleId, RoleName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSummary {
    id: RoleId,
    name: RoleName,
}

impl RoleSummary {
    pub fn new(name: RoleName) -> Self {
        Self {
            id: RoleId::new(),
            name,
        }
    }

    pub fn from_parts(id: RoleId, name: RoleName) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> RoleId {
        self.id
    }

    pub fn name(&self) -> &RoleName {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role_name() -> RoleName {
        RoleName::new("test-role").unwrap()
    }

    #[test]
    fn new_creates_a_role_summary_with_its_name() {
        let name = role_name();

        let summary = RoleSummary::new(name.clone());

        assert_ne!(summary.id(), RoleId::nil());
        assert_eq!(summary.name(), &name);
    }

    #[test]
    fn from_parts_preserves_the_role_summary_data() {
        let id = RoleId::nil();
        let name = role_name();

        let summary = RoleSummary::from_parts(id, name.clone());

        assert_eq!(summary.id(), id);
        assert_eq!(summary.name(), &name);
    }
}
