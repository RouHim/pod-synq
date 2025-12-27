mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod repository;
mod routes;
mod services;
mod state;
mod utils;

use sqlx::SqlitePool;
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::middleware::AuthService;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().map_err(|e| anyhow::anyhow!("Config error: {}", e))?;
    config
        .validate()
        .map_err(|e| anyhow::anyhow!("Config validation error: {}", e))?;

    init_logging(&config.log_level);

    tracing::info!("Starting PodSynq v0.1.0");
    tracing::info!("Database path: {}", config.db_path);

    ensure_database_exists(&config.db_path);
    let pool = create_database_pool(&config.db_path).await?;
    run_migrations(&pool).await?;

    let state = AppState::new(pool.clone());
    let auth_service = AuthService::new(state.user_service.clone(), state.session_service.clone());

    initialize_admin_user(&state, &config).await?;

    let routes = create_app(auth_service, state, config.clone());

    let addr = ([0, 0, 0, 0], config.port);
    tracing::info!("Server listening on http://0.0.0.0:{}", config.port);

    warp::serve(routes).run(addr).await;

    Ok(())
}

fn init_logging(log_level: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn ensure_database_exists(db_path: &str) {
    if !Path::new(db_path).exists() {
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
    }
}

async fn create_database_pool(db_path: &str) -> anyhow::Result<SqlitePool> {
    let database_url = format!("sqlite:{}", db_path);
    let pool = SqlitePool::connect(&database_url).await?;
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let migration_sql = include_str!("../migrations/001_initial.sql");

    tracing::info!("Running database migrations");

    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await?;
        }
    }

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

async fn initialize_admin_user(state: &AppState, config: &Config) -> anyhow::Result<()> {
    let admin_username = config.admin_username.as_deref();
    let admin_password = config.admin_password.as_deref();

    if let (Some(username), Some(password)) = (admin_username, admin_password) {
        let created = state
            .user_service
            .initialize_admin_if_needed(username, password)
            .await?;

        if created {
            tracing::info!("Initialized admin user: {}", username);
        }
    } else {
        let is_empty = state.user_service.is_empty().await?;
        if is_empty {
            tracing::warn!(
                "No users exist in database. Set PODSYNQ_ADMIN_USERNAME and PODSYNQ_ADMIN_PASSWORD to create initial admin user."
            );
        }
    }

    Ok(())
}

fn create_app(
    auth_service: AuthService,
    state: AppState,
    config: Config,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    crate::routes::create_routes(auth_service, state, config)
}
