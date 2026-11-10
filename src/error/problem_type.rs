use axum::http::Uri;

pub struct ProblemType(&'static str);

impl ProblemType {
    const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub fn as_uri(&self) -> Uri {
        Uri::from_static(self.0)
    }
}

pub const INTERNAL_SERVER_ERROR: ProblemType =
    ProblemType::new("https://api.zekurix.com/problems/internal-server-error");

pub mod http {
    use super::ProblemType;

    pub const NOT_FOUND: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/http/not-found");

    pub const METHOD_NOT_ALLOWED: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/http/method-not-allowed");

    pub const GATEWAY_TIMEOUT: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/http/gateway-timeout");

    pub const UNAUTHORIZED: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/http/unauthorized");
}

pub mod json {
    use super::ProblemType;

    pub const DATA_ERROR: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/json/data-error");

    pub const SYNTAX_ERROR: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/json/syntax-error");

    pub const MISSING_CONTENT_TYPE: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/json/missing-content-type");

    pub const BYTES_REJECTION: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/json/bytes-rejection");
}

pub mod path {
    use super::ProblemType;

    pub const REJECTION: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/path/rejection");
}

pub mod query {
    use super::ProblemType;

    pub const INVALID: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/query/invalid");
}

pub mod user {
    use super::ProblemType;

    pub const NOT_FOUND: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/user/not-found");

    pub const ALREADY_EXISTS: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/user/already-exists");
}

pub mod identity {
    use super::ProblemType;

    pub const INVALID_ISSUER: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/identity/invalid-issuer");

    pub const INVALID_SUBJECT: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/identity/invalid-subject");
}

pub mod permission {
    use super::ProblemType;

    pub const INVALID: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/permission/invalid");

    pub const DUPLICATE: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/permission/duplicate");
}

pub mod role {
    use super::ProblemType;

    pub const INVALID_NAME: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/role/invalid-name");

    pub const ALREADY_EXISTS: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/role/already-exists");

    pub const NOT_FOUND: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/role/not-found");
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(INTERNAL_SERVER_ERROR, "internal-server-error" ; "internal server error")]
    #[test_case(http::NOT_FOUND, "http/not-found" ; "http not found")]
    #[test_case(http::METHOD_NOT_ALLOWED, "http/method-not-allowed" ; "http method not allowed")]
    #[test_case(http::GATEWAY_TIMEOUT, "http/gateway-timeout" ; "http gateway timeout")]
    #[test_case(http::UNAUTHORIZED, "http/unauthorized" ; "http unauthorized")]
    #[test_case(json::DATA_ERROR, "json/data-error" ; "json data error")]
    #[test_case(json::SYNTAX_ERROR, "json/syntax-error" ; "json syntax error")]
    #[test_case(json::MISSING_CONTENT_TYPE, "json/missing-content-type" ; "json missing content type")]
    #[test_case(json::BYTES_REJECTION, "json/bytes-rejection" ; "json bytes rejection")]
    #[test_case(path::REJECTION, "path/rejection" ; "path rejection")]
    #[test_case(query::INVALID, "query/invalid" ; "invalid query")]
    #[test_case(user::NOT_FOUND, "user/not-found" ; "user not found")]
    #[test_case(user::ALREADY_EXISTS, "user/already-exists" ; "user already exists")]
    #[test_case(identity::INVALID_ISSUER, "identity/invalid-issuer" ; "identity invalid issuer")]
    #[test_case(identity::INVALID_SUBJECT, "identity/invalid-subject" ; "identity invalid subject")]
    #[test_case(permission::INVALID, "permission/invalid" ; "permission invalid")]
    #[test_case(permission::DUPLICATE, "permission/duplicate" ; "permission duplicate")]
    #[test_case(role::INVALID_NAME, "role/invalid-name" ; "role invalid name")]
    #[test_case(role::ALREADY_EXISTS, "role/already-exists" ; "role already exists")]
    #[test_case(role::NOT_FOUND, "role/not-found" ; "role not found")]
    fn internal_server_error_uri(problem_type: ProblemType, expected_uri: &str) {
        let uri = format!("https://api.zekurix.com/problems/{expected_uri}");
        assert_eq!(problem_type.as_uri(), uri.parse::<Uri>().unwrap());
    }
}
