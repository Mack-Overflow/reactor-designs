use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel_migrations::MigrationHarness;

use crate::services;
use crate::MIGRATIONS;

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

fn run_migrations(pool: &crate::DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection for migrations");
    log::info!("Running database migrations...");
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(applied) if applied.is_empty() => log::info!("No pending migrations"),
        Ok(applied) => {
            log::info!("Applied {} migration(s):", applied.len());
            for m in applied {
                log::info!("  - {}", m);
            }
        }
        Err(e) => panic!("Failed to run database migrations: {}", e),
    }
}

fn spawn_pruner(pool: crate::DbPool) {
    tokio::spawn(async move {
        loop {
            let wait = services::pruning::duration_until_next_midnight();
            log::info!(
                "Simulation pruner scheduled to run in {:.1} hours",
                wait.as_secs_f64() / 3600.0
            );
            tokio::time::sleep(wait).await;

            let retention_days: i64 = std::env::var("SIMULATION_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30);

            let pool = pool.clone();
            match tokio::task::spawn_blocking(move || {
                services::pruning::prune_old_simulations(&pool, retention_days)
            })
            .await
            {
                Ok(Ok(n)) => log::info!("Pruned {} old simulation run(s) (retention={}d)", n, retention_days),
                Ok(Err(e)) => log::error!("Simulation pruner error: {}", e),
                Err(e) => log::error!("Simulation pruner task panicked: {}", e),
            }
        }
    });
}

pub async fn run() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT/API_PORT must be a valid port number");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool");

    run_migrations(&pool);
    spawn_pruner(pool.clone());

    log::info!("Starting server at {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pool.clone()))
            .configure(crate::routes::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
