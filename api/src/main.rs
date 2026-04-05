use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub mod schema;
pub mod models;
pub mod handlers;
pub mod services;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn run_migrations(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("API_PORT must be a valid port number");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool");

    run_migrations(&pool);

    log::info!("Starting server at {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .route("/api/health", web::get().to(health))
            .service(
                web::scope("/api/reactors")
                    .route("", web::get().to(handlers::reactors::list_reactors))
                    .route("", web::post().to(handlers::reactors::create_reactor))
                    .route("/{id}", web::get().to(handlers::reactors::get_reactor))
                    .route("/{id}", web::delete().to(handlers::reactors::delete_reactor))
            )
            .service(
                web::scope("/api/simulations")
                    .route("", web::post().to(handlers::simulations::launch_simulation))
                    .route("/stream", web::post().to(handlers::simulations::stream_simulation))
                    .route("/{id}", web::get().to(handlers::simulations::get_simulation))
                    .route("/{id}/results", web::get().to(handlers::simulations::get_simulation_results))
                    .route("/{id}/waste", web::get().to(handlers::simulations::get_simulation_waste))
            )
            .route("/api/compare", web::post().to(handlers::compare::compare_runs))
            .route("/api/ingest", web::post().to(handlers::ingest::ingest_file))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
