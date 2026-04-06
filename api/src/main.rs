use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

pub mod schema;
pub mod models;
pub mod handlers;
pub mod services;
pub mod kernel;
pub mod routes;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    kernel::run().await
}
