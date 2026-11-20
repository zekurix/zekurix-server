use axum_test::TestServer;
use dotenvy::dotenv;

use zekurix_server::Application;
use zekurix_server::router::routes;
use zekurix_server::secrets::Secrets;
use zekurix_server::settings::Settings;

use crate::common::TempDatabase;
use crate::input::settings::fixtures::test_settings;

use super::authentication::TestAuthentication;

pub(super) struct TestHarness {
    // Keep the temporary database alive for the lifetime of the test harness.
    _temp_database: TempDatabase,
    pub(super) server: TestServer,
    pub(super) settings: Settings,
    pub(super) authentication: TestAuthentication,
}

impl TestHarness {
    pub(super) async fn new() -> Self {
        dotenv().ok();

        let mut settings = test_settings();
        let secrets = Secrets::load().unwrap();

        let temp_database = TempDatabase::new(&settings.database, &secrets.database)
            .create()
            .await
            .unwrap();
        settings.database.database = temp_database.database().to_string();

        let authentication = TestAuthentication::new(&settings.authentication);
        let application = Application::new(settings.clone()).await.unwrap();
        let router = routes::router(application);
        let server = TestServer::new(router);

        TestHarness {
            _temp_database: temp_database,
            server,
            settings,
            authentication,
        }
    }
}
