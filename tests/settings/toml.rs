use zekurix_server::settings::Settings;

use crate::cli::fixtures::cli_with_config;

#[test]
fn should_load_and_validate_example_configuration() {
	let cli = cli_with_config("zekurix.example.toml");

	let settings = Settings::load(&cli).expect("example configuration should be valid");
    let default = Settings::default();

	assert_eq!(settings.logging.level, default.logging.level);
	assert_eq!(settings.logging.format, default.logging.format);

	assert_eq!(settings.server.host, default.server.host);
	assert_eq!(settings.server.port, default.server.port);
	assert_eq!(settings.server.timeout, default.server.timeout);

	assert_eq!(settings.database.username.as_deref(), Some("postgres"));
	assert_eq!(settings.database.host, default.database.host);
	assert_eq!(settings.database.port, default.database.port);
	assert_eq!(settings.database.database, default.database.database);
	assert_eq!(settings.database.max_connections, default.database.max_connections);
	assert_eq!(settings.database.min_connections, default.database.min_connections);
	assert_eq!(settings.database.acquire_timeout, default.database.acquire_timeout);
	assert_eq!(settings.database.idle_timeout, default.database.idle_timeout);
	assert_eq!(settings.database.max_lifetime, default.database.max_lifetime);
    assert_eq!(settings.database.migrate, default.database.migrate);
}


#[test]
fn should_load_and_validate_test_configuration() {
	let cli = cli_with_config("zekurix.test.toml");

	let settings = Settings::load(&cli).expect("test configuration should be valid");

	assert!(settings.database.migrate);
}
