use super::{RoleName, RoleQuery};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoleFilter {
    pub name: Option<RoleName>,
}

impl From<RoleQuery> for RoleFilter {
    fn from(query: RoleQuery) -> Self {
        Self { name: query.name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_filter_from_query() {
        let name = RoleName::new("admin").unwrap();

        let query = RoleQuery {
            name: Some(name.clone()),
        };

        let filter = RoleFilter::from(query);

        assert_eq!(filter.name, Some(name));
    }
}
