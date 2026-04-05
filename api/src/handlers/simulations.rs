use actix_web::{web, HttpResponse};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use std::str::FromStr;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::models::*;
use crate::schema::{simulation_results, simulation_runs, waste_isotope_inventory};
use crate::services::simulation::engine::{self, SimulationParams, StepResult};
use crate::DbPool;

#[derive(serde::Deserialize)]
pub struct LaunchRequest {
    pub reactor_id: Uuid,
    pub params: SimulationParams,
}

// Persist a single step result + its isotope inventory to the DB.
fn persist_step(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    run_id: Uuid,
    step: &StepResult,
) -> Result<(), String> {
    let new_result = NewSimulationResult {
        run_id,
        time_step: step.time_step as i32,
        time_years: Some(BigDecimal::from_str(&step.time_years.to_string()).unwrap()),
        fuel_burnup_gwd_t: Some(BigDecimal::from_str(&step.fuel.burnup_gwd_t.to_string()).unwrap()),
        fuel_remaining_pct: Some(BigDecimal::from_str(&step.fuel.fuel_remaining_pct.to_string()).unwrap()),
        coolant_temp_inlet_c: Some(BigDecimal::from_str(&step.fluid.inlet_temp_c.to_string()).unwrap()),
        coolant_temp_outlet_c: Some(BigDecimal::from_str(&step.fluid.outlet_temp_c.to_string()).unwrap()),
        coolant_flow_rate_kg_s: Some(BigDecimal::from_str(&step.fluid.flow_rate_kg_s.to_string()).unwrap()),
        thermal_power_mw: Some(BigDecimal::from_str(&step.fuel.effective_thermal_power_mw.to_string()).unwrap()),
        electric_power_mw: Some(BigDecimal::from_str(&step.power.electric_power_mw.to_string()).unwrap()),
        capacity_factor: Some(BigDecimal::from_str(&step.power.capacity_factor.to_string()).unwrap()),
        waste_actinides_kg: Some(BigDecimal::from_str(&step.waste.total_actinides_kg.to_string()).unwrap()),
        waste_fission_products_kg: Some(BigDecimal::from_str(&step.waste.total_fission_products_kg.to_string()).unwrap()),
        waste_total_activity_bq: Some(BigDecimal::from_str(&step.waste.total_activity_bq.to_string()).unwrap()),
        extra_data: None,
    };

    let inserted: SimulationResult = diesel::insert_into(simulation_results::table)
        .values(&new_result)
        .get_result(conn)
        .map_err(|e| e.to_string())?;

    for (isotope_name, isotope_state) in &step.waste.isotopes {
        let new_isotope = NewWasteIsotopeInventory {
            result_id: inserted.id,
            isotope: isotope_name.clone(),
            mass_kg: Some(BigDecimal::from_str(&isotope_state.mass_kg.to_string()).unwrap()),
            activity_bq: Some(BigDecimal::from_str(&isotope_state.activity_bq.to_string()).unwrap()),
            half_life_years: Some(BigDecimal::from_str(&isotope_state.half_life_years.to_string()).unwrap()),
        };
        if let Err(e) = diesel::insert_into(waste_isotope_inventory::table)
            .values(&new_isotope)
            .execute(conn)
        {
            log::warn!("Failed to insert isotope {}: {}", isotope_name, e);
        }
    }

    Ok(())
}

// Verify the reactor exists, create a run record, return the run.
fn create_run(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    reactor_id: Uuid,
    params: &SimulationParams,
) -> Result<SimulationRun, HttpResponse> {
    use crate::schema::reactor_designs;

    let count = reactor_designs::table
        .find(reactor_id)
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})))?;

    if count == 0 {
        return Err(HttpResponse::NotFound().json(serde_json::json!({"error": "Reactor not found"})));
    }

    let params_json = serde_json::to_value(params).unwrap();
    let new_run = NewSimulationRun {
        reactor_id,
        status: "running".into(),
        params: Some(params_json),
    };

    diesel::insert_into(simulation_runs::table)
        .values(&new_run)
        .get_result(conn)
        .map_err(|e| HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})))
}

fn mark_completed(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    run_id: Uuid,
) {
    let _ = diesel::update(simulation_runs::table.find(run_id))
        .set((
            simulation_runs::status.eq("completed"),
            simulation_runs::completed_at.eq(diesel::dsl::now),
        ))
        .execute(conn);
}

fn mark_failed(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    run_id: Uuid,
    error: &str,
) {
    let _ = diesel::update(simulation_runs::table.find(run_id))
        .set((
            simulation_runs::status.eq("failed"),
            simulation_runs::error_message.eq(error),
        ))
        .execute(conn);
}

// POST /api/simulations — launch and return results synchronously.
pub async fn launch_simulation(
    pool: web::Data<DbPool>,
    body: web::Json<LaunchRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let run = match create_run(&mut conn, req.reactor_id, &req.params) {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    match engine::run_simulation(&req.params) {
        Ok(output) => {
            for step in &output.steps {
                if let Err(e) = persist_step(&mut conn, run.id, step) {
                    mark_failed(&mut conn, run.id, &e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({"error": e}));
                }
            }
            mark_completed(&mut conn, run.id);

            HttpResponse::Created().json(serde_json::json!({
                "id": run.id,
                "status": "completed",
                "total_steps": output.total_steps,
                "average_capacity_factor": output.average_capacity_factor,
            }))
        }
        Err(e) => {
            mark_failed(&mut conn, run.id, &e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

// SSE event types sent during streaming.
#[derive(serde::Serialize)]
struct SseStepEvent {
    time_step: usize,
    total_steps: usize,
    time_years: f64,
    fuel_burnup_gwd_t: f64,
    fuel_remaining_pct: f64,
    coolant_temp_inlet_c: f64,
    coolant_temp_outlet_c: f64,
    coolant_flow_rate_kg_s: f64,
    thermal_power_mw: f64,
    electric_power_mw: f64,
    capacity_factor: f64,
    waste_actinides_kg: f64,
    waste_fission_products_kg: f64,
    waste_total_activity_bq: f64,
}

impl SseStepEvent {
    fn from_step(step: &StepResult, total_steps: usize) -> Self {
        Self {
            time_step: step.time_step,
            total_steps,
            time_years: step.time_years,
            fuel_burnup_gwd_t: step.fuel.burnup_gwd_t,
            fuel_remaining_pct: step.fuel.fuel_remaining_pct,
            coolant_temp_inlet_c: step.fluid.inlet_temp_c,
            coolant_temp_outlet_c: step.fluid.outlet_temp_c,
            coolant_flow_rate_kg_s: step.fluid.flow_rate_kg_s,
            thermal_power_mw: step.fuel.effective_thermal_power_mw,
            electric_power_mw: step.power.electric_power_mw,
            capacity_factor: step.power.capacity_factor,
            waste_actinides_kg: step.waste.total_actinides_kg,
            waste_fission_products_kg: step.waste.total_fission_products_kg,
            waste_total_activity_bq: step.waste.total_activity_bq,
        }
    }
}

// POST /api/simulations/stream — launch simulation and stream results via SSE.
pub async fn stream_simulation(
    pool: web::Data<DbPool>,
    body: web::Json<LaunchRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let run = match create_run(&mut conn, req.reactor_id, &req.params) {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let run_id = run.id;
    let params = req.params.clone();
    let pool_clone = pool.clone();

    // Channel for sending SSE events from the blocking simulation thread
    let (tx, mut rx) = mpsc::channel::<String>(64);
    let tx_done = tx.clone(); // Clone for use after web::block completes

    // Spawn the simulation on a blocking thread
    actix_web::rt::spawn(async move {
        let result = web::block(move || {
            let mut conn = pool_clone.get().map_err(|e| e.to_string())?;

            engine::run_simulation_streaming(&params, |step, total_steps| {
                // Persist each step to DB
                if let Err(e) = persist_step(&mut conn, run_id, step) {
                    log::error!("Failed to persist step {}: {}", step.time_step, e);
                }

                // Send SSE event (non-blocking — drop if channel full)
                let event = SseStepEvent::from_step(step, total_steps);
                let json = serde_json::to_string(&event).unwrap();
                let sse_msg = format!("data: {}\n\n", json);
                let _ = tx.try_send(sse_msg);
            })
        })
        .await;

        // Get a connection for marking completion
        if let Ok(mut conn) = pool.get() {
            match result {
                Ok(Ok(output)) => {
                    mark_completed(&mut conn, run_id);
                    let done_msg = format!(
                        "event: done\ndata: {}\n\n",
                        serde_json::json!({
                            "id": run_id,
                            "status": "completed",
                            "total_steps": output.total_steps,
                            "average_capacity_factor": output.average_capacity_factor,
                        })
                    );
                    let _ = tx_done.try_send(done_msg);
                }
                Ok(Err(e)) => {
                    let err_str = e.to_string();
                    mark_failed(&mut conn, run_id, &err_str);
                    let err_msg = format!(
                        "event: error\ndata: {}\n\n",
                        serde_json::json!({"error": err_str})
                    );
                    let _ = tx_done.try_send(err_msg);
                }
                Err(e) => {
                    let err_str = e.to_string();
                    mark_failed(&mut conn, run_id, &err_str);
                    let err_msg = format!(
                        "event: error\ndata: {}\n\n",
                        serde_json::json!({"error": err_str})
                    );
                    let _ = tx_done.try_send(err_msg);
                }
            }
        }
    });

    // Return SSE response that reads from the channel
    let stream = async_stream::stream! {
        // Send initial event with run ID
        yield Ok::<_, actix_web::Error>(
            web::Bytes::from(format!(
                "event: started\ndata: {}\n\n",
                serde_json::json!({"id": run_id})
            ))
        );

        while let Some(msg) = rx.recv().await {
            yield Ok(web::Bytes::from(msg));
        }
    };

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream)
}

// GET /api/simulations/{id} — run status & metadata.
pub async fn get_simulation(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let run_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    match simulation_runs::table.find(run_id).first::<SimulationRun>(&mut conn) {
        Ok(run) => HttpResponse::Ok().json(run),
        Err(diesel::result::Error::NotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Simulation not found"}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

// GET /api/simulations/{id}/results — time-series results.
pub async fn get_simulation_results(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let run_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    match simulation_results::table
        .filter(simulation_results::run_id.eq(run_id))
        .order(simulation_results::time_step.asc())
        .load::<SimulationResult>(&mut conn)
    {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

// GET /api/simulations/{id}/waste — isotope inventory for the final time step.
pub async fn get_simulation_waste(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let run_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let last_result = simulation_results::table
        .filter(simulation_results::run_id.eq(run_id))
        .order(simulation_results::time_step.desc())
        .first::<SimulationResult>(&mut conn);

    match last_result {
        Ok(result) => {
            match waste_isotope_inventory::table
                .filter(waste_isotope_inventory::result_id.eq(result.id))
                .order(waste_isotope_inventory::isotope.asc())
                .load::<WasteIsotopeInventory>(&mut conn)
            {
                Ok(isotopes) => HttpResponse::Ok().json(isotopes),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
            }
        }
        Err(diesel::result::Error::NotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "No results found for this simulation"}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
