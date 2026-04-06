use actix_web::web;
use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/api/health", web::get().to(crate::kernel::health))
        .service(
            web::scope("/api/reactors")
                .route("", web::get().to(handlers::reactors::list_reactors))
                .route("", web::post().to(handlers::reactors::create_reactor))
                .route("/{id}", web::get().to(handlers::reactors::get_reactor))
                .route("/{id}", web::delete().to(handlers::reactors::delete_reactor)),
        )
        .service(
            web::scope("/api/simulations")
                .route("", web::post().to(handlers::simulations::launch_simulation))
                .route("/stream", web::post().to(handlers::simulations::stream_simulation))
                .route("/{id}", web::get().to(handlers::simulations::get_simulation))
                .route("/{id}/results", web::get().to(handlers::simulations::get_simulation_results))
                .route("/{id}/waste", web::get().to(handlers::simulations::get_simulation_waste)),
        )
        .route("/api/compare", web::post().to(handlers::compare::compare_runs))
        .route("/api/ingest", web::post().to(handlers::ingest::ingest_file));
}
