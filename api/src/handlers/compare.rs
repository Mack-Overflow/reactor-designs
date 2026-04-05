use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::*;
use crate::schema::{reactor_designs, simulation_results, simulation_runs};
use crate::DbPool;

// POST /api/compare — compare multiple simulation runs side-by-side.
// Body: { "run_ids": [UUID, ...] }
#[derive(serde::Deserialize)]
pub struct CompareRequest {
    pub run_ids: Vec<Uuid>,
}

#[derive(serde::Serialize)]
pub struct CompareEntry {
    pub run_id: Uuid,
    pub reactor_name: String,
    pub reactor_design_type: String,
    pub results: Vec<SimulationResult>,
}

pub async fn compare_runs(
    pool: web::Data<DbPool>,
    body: web::Json<CompareRequest>,
) -> HttpResponse {
    let run_ids = &body.run_ids;
    if run_ids.is_empty() || run_ids.len() > 6 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Provide between 1 and 6 run_ids"
        }));
    }

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut entries = Vec::new();

    for run_id in run_ids {
        // Get the run and its reactor info
        let run = match simulation_runs::table
            .find(run_id)
            .first::<SimulationRun>(&mut conn)
        {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Run {} not found", run_id)
                }));
            }
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        };

        let reactor = match reactor_designs::table
            .find(run.reactor_id)
            .first::<ReactorDesign>(&mut conn)
        {
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        };

        let results = match simulation_results::table
            .filter(simulation_results::run_id.eq(run_id))
            .order(simulation_results::time_step.asc())
            .load::<SimulationResult>(&mut conn)
        {
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        };

        entries.push(CompareEntry {
            run_id: *run_id,
            reactor_name: reactor.name,
            reactor_design_type: reactor.design_type,
            results,
        });
    }

    HttpResponse::Ok().json(entries)
}
