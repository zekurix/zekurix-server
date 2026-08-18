use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use dotenv::dotenv;
use sqlx::migrate::MigrateDatabase;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tokio::sync::OnceCell;
use tower::util::ServiceExt;
use uuid::Uuid;

use zekurix_server::application::Application;
use zekurix_server::router::build_router;
use zekurix_server::secrets::Secrets;
use zekurix_server::settings::Settings;

static POSTGRES_CONTAINER: OnceCell<ContainerAsync<Postgres>> = OnceCell::const_new();

pub struct TestApp {
    pub router: Router,
}

impl TestApp {
    async fn get_postgres_container(
        settings: &Settings,
        secrets: &Secrets,
    ) -> &'static ContainerAsync<Postgres> {
        POSTGRES_CONTAINER
            .get_or_init(|| async {
                Postgres::default()
                    .with_user(settings.database.username.as_deref().unwrap())
                    .with_password(secrets.database.password().unwrap())
                    .start()
                    .await
                    .unwrap()
            })
            .await
    }

    async fn create_postgres_table(settings: &Settings, secrets: &Secrets) {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            settings.database.username.as_deref().unwrap(),
            secrets.database.password().unwrap(),
            settings.database.host,
            settings.database.port,
            settings.database.database,
        );

        sqlx::Postgres::create_database(&url).await.unwrap();
    }

    async fn init_database(settings: Settings) -> Settings {
        let mut settings = settings;
        let secrets = Secrets::load();

        settings.database.username = Some("postgres".to_string());
        settings.database.database = format!(
            "{}_test_{}",
            settings.database.database,
            Uuid::now_v7().simple()
        );

        let postgres_container = Self::get_postgres_container(&settings, &secrets).await;
        settings.database.host = postgres_container.get_host().await.unwrap().to_string();
        settings.database.port = postgres_container.get_host_port_ipv4(5432).await.unwrap();

        Self::create_postgres_table(&settings, &secrets).await;
        settings.database.migrate = true;

        settings
    }

    pub async fn new() -> Self {
        dotenv().ok();

        let settings = Settings::default();
        let settings = Self::init_database(settings).await;
        let application = Application::new(settings).await.unwrap();
        let router = build_router(application);

        TestApp { router }
    }

    pub async fn request(&self, req: Request<Body>) -> Response<Body> {
        self.router.clone().oneshot(req).await.unwrap()
    }
}
