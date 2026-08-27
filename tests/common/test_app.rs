use axum_test::TestServer;
use dotenv::dotenv;

use zekurix_server::Application;
use zekurix_server::app::routes;
use zekurix_server::secrets::Secrets;

use super::temp_database::TempDatabase;

use crate::settings::fixtures::test_settings;

pub struct TestApplication {
    // The database is dropped when the TestApp is dropped, so we need to keep it around for the lifetime of the test.
    _temp_database: TempDatabase,
    pub server: TestServer,
}

impl TestApplication {
    pub async fn new() -> Self {
        dotenv().ok();

        let mut settings = test_settings();
        let secrets = Secrets::load().unwrap();

        let temp_database = TempDatabase::new(&settings.database, &secrets.database)
            .create()
            .await
            .unwrap();
        settings.database.database = temp_database.database().to_string();

        let application = Application::new(settings).await.unwrap();
        let router = routes::router(application);
        let server = TestServer::new(router);

        TestApplication {
            _temp_database: temp_database,
            server,
        }
    }
}
