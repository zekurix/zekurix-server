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

pub mod user {
    use super::ProblemType;

    pub const NOT_FOUND: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/user/not-found");

    pub const ALREADY_EXISTS: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/user/already-exists");

    pub const INVALID_USERNAME: ProblemType =
        ProblemType::new("https://api.zekurix.com/problems/user/invalid-username");
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(INTERNAL_SERVER_ERROR, "internal-server-error" ; "internal server error")]
    #[test_case(http::NOT_FOUND, "http/not-found" ; "http not found")]
    #[test_case(http::METHOD_NOT_ALLOWED, "http/method-not-allowed" ; "http method not allowed")]
    #[test_case(http::GATEWAY_TIMEOUT, "http/gateway-timeout" ; "http gateway timeout")]
    #[test_case(json::DATA_ERROR, "json/data-error" ; "json data error")]
    #[test_case(json::SYNTAX_ERROR, "json/syntax-error" ; "json syntax error")]
    #[test_case(json::MISSING_CONTENT_TYPE, "json/missing-content-type" ; "json missing content type")]
    #[test_case(json::BYTES_REJECTION, "json/bytes-rejection" ; "json bytes rejection")]
    #[test_case(user::NOT_FOUND, "user/not-found" ; "user not found")]
    #[test_case(user::ALREADY_EXISTS, "user/already-exists" ; "user already exists")]
    #[test_case(user::INVALID_USERNAME, "user/invalid-username" ; "invalid username")]
    fn internal_server_error_uri(problem_type: ProblemType, expected_uri: &str) {
        let uri = format!("https://api.zekurix.com/problems/{expected_uri}");
        assert_eq!(problem_type.as_uri(), uri.parse::<Uri>().unwrap());
    }
}
