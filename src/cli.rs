use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Result, bail};

use clap::parser::ValueSource;
use clap::{CommandFactory, Parser};

#[cfg(target_os = "linux")]
const DEFAULT_CONFIG_PATH: &str = "/etc/zekurix/zekurix.toml";

#[cfg(target_os = "macos")]
const DEFAULT_CONFIG_PATH: &str = "/usr/local/etc/zekurix.toml";

#[cfg(target_os = "windows")]
const DEFAULT_CONFIG_PATH: &str = r"C:\ProgramData\Zekurix\zekurix.toml";

/// Zekurix server.
///
/// Zero-Knowledge Collaboration Backend.
///
/// The Zekurix server provides REST APIs, encrypted storage and
/// permission enforcement for privacy-preserving applications.
/// Sensitive data remains end-to-end encrypted, ensuring that the
/// server never has access to plaintext content.
#[derive(Parser)]
#[command(version, propagate_version = true, about, long_about)]
pub struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, value_name = "FILE", default_value = DEFAULT_CONFIG_PATH)]
    pub config: PathBuf,

    /// Address to bind the server to.
    #[arg(short, long, value_name = "HOST:PORT")]
    pub bind: Option<SocketAddr>,

    /// Enable verbose logging.
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Reduce logging output to warnings and errors only.
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            config: DEFAULT_CONFIG_PATH.into(),
            bind: None,
            verbose: false,
            quiet: false,
        }
    }
}

impl Cli {
    pub fn build() -> Result<Self> {
        let matches = Self::command().get_matches();
        let is_config_present = matches.value_source("config") == Some(ValueSource::CommandLine);
        let cli = Self::parse();

        if is_config_present && !cli.config.exists() {
            bail!(
                "configuration file '{}' does not exist",
                cli.config.display()
            );
        }

        Ok(cli)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn should_use_default_config_file() {
        let cli = Cli::try_parse_from(["zekurix-server"]).unwrap();
        assert_eq!(cli.config, PathBuf::from(DEFAULT_CONFIG_PATH));
    }

    #[test]
    fn should_override_config_file() {
        let tmp = NamedTempFile::new().expect("failed to create temporary file");
        let tmp_path = tmp.path().to_string_lossy().to_string();
        let cli = Cli::try_parse_from(["zekurix-server", "--config", &tmp_path]).unwrap();
        assert_eq!(cli.config, PathBuf::from(&tmp_path));
    }

    #[test]
    fn should_parse_bind_ipv4_address() {
        let cli = Cli::try_parse_from(["zekurix-server", "--bind", "192.168.1.1:8080"]).unwrap();
        assert_eq!(cli.bind, Some("192.168.1.1:8080".parse().unwrap()));
    }

    #[test]
    fn should_parse_bind_loopback() {
        let cli = Cli::try_parse_from(["zekurix-server", "--bind", "[::1]:8080"]).unwrap();
        assert_eq!(cli.bind, Some("[::1]:8080".parse().unwrap()));
    }

    #[test]
    fn should_parse_bind_localhost() {
        let cli = Cli::try_parse_from(["zekurix-server", "--bind", "127.0.0.1:8080"]).unwrap();
        assert_eq!(cli.bind, Some("127.0.0.1:8080".parse().unwrap()));
    }

    #[test]
    fn should_parse_bind_ipv6_address() {
        let cli = Cli::try_parse_from(["zekurix-server", "--bind", "[2001:db8::1]:8080"]).unwrap();
        assert_eq!(cli.bind, Some("[2001:db8::1]:8080".parse().unwrap()));
    }

    #[test]
    fn should_reject_invalid_bind_address() {
        let result = Cli::try_parse_from(["zekurix-server", "--bind", "foobar"]);
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_verbose_and_quiet_together() {
        let result = Cli::try_parse_from(["zekurix-server", "--verbose", "--quiet"]);
        assert!(result.is_err());
    }
}
