use anyhow::{Result, anyhow, ensure};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

use crate::cli::Cli;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub timeout: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            timeout: 10,
        }
    }
}

impl Settings {
    pub fn merge(&mut self, cli: &Cli) {
        if let Some(bind) = cli.bind {
            self.host = bind.ip().to_string();
            self.port = bind.port();
        }
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
                (1..=360).contains(&self.timeout),
                "Server timeout must be between 1 and 360 seconds"
               );
        Ok(())
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        let host = self.host.trim();

        ensure!(!host.is_empty(), "host is empty");
        ensure!(
            !host.chars().any(|c| c.is_whitespace()),
            "host contains whitespace"
        );
        ensure!(
            !host.parse::<SocketAddr>().is_ok(),
            "host must not include a port"
        );

        if host == "localhost" {
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), self.port))
        } else if let Ok(ip) = host.parse::<IpAddr>() {
            Ok(SocketAddr::new(ip, self.port))
        } else {
            Err(anyhow!("host must be an IP literal or 'localhost'"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_default_settings() {
        let settings = Settings::default();

        assert_eq!(settings.host, "127.0.0.1");
        assert_eq!(settings.port, 8080);
    }

    #[test]
    fn should_build_socket_addr_ipv4() {
        let settings = Settings {
            host: "127.0.0.1".into(),
            port: 8080,
            ..Default::default()
        };
        let addr = settings.socket_addr().unwrap();

        assert_eq!(addr, "127.0.0.1:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn should_build_socket_addr_ipv6() {
        let settings = Settings {
            host: "::1".into(),
            port: 8080,
            ..Default::default()
        };
        let addr = settings.socket_addr().unwrap();

        assert_eq!(addr, "[::1]:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn should_reject_empty_host() {
        let settings = Settings {
            host: "".into(),
            port: 8080,
            ..Default::default()
        };

        assert!(settings.socket_addr().is_err());
    }

    #[test]
    fn should_reject_invalid_host_with_space() {
        let settings = Settings {
            host: "invalid host".into(),
            port: 8080,
            ..Default::default()
        };

        assert!(settings.socket_addr().is_err());
    }

    #[test]
    fn should_reject_host_with_port() {
        let settings = Settings {
            host: "127.0.0.1:8080".into(),
            port: 8080,
            ..Default::default()
        };

        assert!(settings.socket_addr().is_err());
    }

    #[test]
    fn supports_port_boundaries() {
        let s0 = Settings {
            host: "127.0.0.1".into(),
            port: 0,
            ..Default::default()
        };
        let smax = Settings {
            host: "127.0.0.1".into(),
            port: 65535,
            ..Default::default()
        };

        assert!(s0.socket_addr().is_ok());
        assert!(smax.socket_addr().is_ok());
    }

    #[test]
    fn merge_with_bind_overrides_settings() {
        let mut settings = Settings::default();
        let cli = Cli {
            bind: Some("10.0.0.5:4242".parse::<SocketAddr>().unwrap()),
            ..Default::default()
        };

        settings.merge(&cli);

        assert_eq!(settings.host, "10.0.0.5");
        assert_eq!(settings.port, 4242);
    }

    #[test]
    fn merge_with_none_leaves_settings_unchanged() {
        let mut settings = Settings::default();
        let original = Settings::default();
        let cli = Cli {
            bind: None,
            ..Default::default()
        };

        settings.merge(&cli);

        assert_eq!(settings.host, original.host);
        assert_eq!(settings.port, original.port);
    }

    #[test]
    fn test_validate_default_settings() {
        let settings = Settings::default();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn should_validate_default_settings() {
        let settings = Settings::default();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn should_accept_minimum_timeout() {
        let settings = Settings {
            timeout: 1,
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn should_accept_maximum_timeout() {
        let settings = Settings {
            timeout: 360,
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn should_reject_timeout_below_minimum() {
        let settings = Settings {
            timeout: 0,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn should_reject_timeout_above_maximum() {
        let settings = Settings {
            timeout: 361,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }
}
