use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use dotenv::dotenv;
use tower::util::ServiceExt;

use zekurix_server::application::Application;
use zekurix_server::router::build_router;
use zekurix_server::settings::Settings;

pub struct TestApp {
    pub router: Router,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenv().ok();

        let mut settings = Settings::default();
        settings.database.username = Some("postgres".to_string());
        settings.database.migrate = true;

        let application = Application::new(settings).await.unwrap();
        let router = build_router(application);

        TestApp { router }
    }

    pub async fn request(&self, req: Request<Body>) -> Response<Body> {
        self.router.clone().oneshot(req).await.unwrap()
    }
}
