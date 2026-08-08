use std::fmt;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::cli::Cli;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Full,
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub level: Level,
    pub format: Format,
}

impl Settings {
    pub fn merge(&mut self, cli: &Cli) {
        if cli.verbose {
            self.level = Level::Debug;
        } else if cli.quiet {
            self.level = Level::Warn;
        }
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;

    #[test]
    fn test_default_enum_defaults() {
        let level = Level::default();
        let format = Format::default();

        assert_eq!(level, Level::Info);
        assert_eq!(format, Format::Full);
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert_eq!(settings.level, Level::Info);
        assert_eq!(settings.format, Format::Full);
    }

    #[test]
    fn test_level_to_string_all_variants() {
        assert_eq!(Level::Trace.to_string(), "trace");
        assert_eq!(Level::Debug.to_string(), "debug");
        assert_eq!(Level::Info.to_string(), "info");
        assert_eq!(Level::Warn.to_string(), "warn");
        assert_eq!(Level::Error.to_string(), "error");
    }

    #[test]
    fn test_merge_with_verbose() {
        let mut settings = Settings::default();
        let cli = Cli {
            verbose: true,
            quiet: false,
            ..Default::default()
        };

        settings.merge(&cli);

        assert_eq!(settings.level, Level::Debug);
        assert_eq!(settings.format, Format::Full);
    }

    #[test]
    fn test_merge_with_quiet() {
        let mut settings = Settings::default();
        let cli = Cli {
            verbose: false,
            quiet: true,
            ..Default::default()
        };

        settings.merge(&cli);

        assert_eq!(settings.level, Level::Warn);
        assert_eq!(settings.format, Format::Full);
    }

    #[test]
    fn test_merge_with_no_flags() {
        let mut settings = Settings::default();
        let cli = Cli {
            verbose: false,
            quiet: false,
            ..Default::default()
        };

        settings.merge(&cli);

        assert_eq!(settings.level, Level::Info);
        assert_eq!(settings.format, Format::Full);
    }

    #[test]
    fn test_merge_preserves_format() {
        let mut settings = Settings {
            level: Level::Info,
            format: Format::Pretty,
        };
        let cli = Cli {
            verbose: true,
            quiet: false,
            ..Default::default()
        };

        settings.merge(&cli);

        assert_eq!(settings.level, Level::Debug);
        assert_eq!(settings.format, Format::Pretty);
    }

    #[test]
    fn test_validate_default_settings() {
        let settings = Settings::default();
        assert!(settings.validate().is_ok());
    }
}
