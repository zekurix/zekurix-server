use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use dotenv::dotenv;
use tower::util::ServiceExt;

use zekurix_server::Application;
use zekurix_server::app::routes;
use zekurix_server::secrets::Secrets;

use super::temp_database::TempDatabase;

use crate::settings::fixtures::test_settings;

pub struct TestApp {
    // The database is dropped when the TestApp is dropped, so we need to keep it around for the lifetime of the test.
    _temp_database: TempDatabase,
    router: Router,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenv().ok();

        let mut settings = test_settings();
        let secrets = Secrets::load();

        let temp_database = TempDatabase::new(&settings.database, &secrets.database)
            .create()
            .await
            .unwrap();
        settings.database.database = temp_database.database().to_string();

        let application = Application::new(settings).await.unwrap();
        let router = routes::router(application);

        TestApp {
            _temp_database: temp_database,
            router,
        }
    }

    pub async fn request(&self, req: Request<Body>) -> Response<Body> {
        self.router.clone().oneshot(req).await.unwrap()
    }
}
