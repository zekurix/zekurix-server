use serde::{Deserialize, Serialize};

use crate::cli::Cli;

#[derive(Debug, Serialize, Deserialize)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Format {
    Full,
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub level: Level,
    pub format: Format,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            level: Level::Info,
            format: Format::Full,
        }
    }
}

impl Settings {
    pub fn merge(&mut self, cli: &Cli) {
        if cli.verbose {
            self.level = Level::Debug;
        } else if cli.quiet {
            self.level = Level::Warn;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert!(matches!(settings.level, Level::Info));
        assert!(matches!(settings.format, Format::Full));
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

        assert!(matches!(settings.level, Level::Debug));
        assert!(matches!(settings.format, Format::Full));
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

        assert!(matches!(settings.level, Level::Warn));
        assert!(matches!(settings.format, Format::Full));
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

        assert!(matches!(settings.level, Level::Info));
        assert!(matches!(settings.format, Format::Full));
    }
}
