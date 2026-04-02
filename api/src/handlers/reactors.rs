use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{NewReactorDesign, ReactorDesign};
use crate::schema::reactor_designs;
use crate::DbPool;

#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    pub design_type: Option<String>,
    pub coolant_type: Option<String>,
    pub fuel_type: Option<String>,
}

pub async fn list_reactors(
    pool: web::Data<DbPool>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut q = reactor_designs::table.into_boxed();

    if let Some(ref dt) = query.design_type {
        q = q.filter(reactor_designs::design_type.eq(dt));
    }
    if let Some(ref ct) = query.coolant_type {
        q = q.filter(reactor_designs::coolant_type.eq(ct));
    }
    if let Some(ref ft) = query.fuel_type {
        q = q.filter(reactor_designs::fuel_type.eq(ft));
    }

    match q.order(reactor_designs::name.asc()).load::<ReactorDesign>(&mut conn) {
        Ok(reactors) => HttpResponse::Ok().json(reactors),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_reactor(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let reactor_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    match reactor_designs::table
        .find(reactor_id)
        .first::<ReactorDesign>(&mut conn)
    {
        Ok(reactor) => HttpResponse::Ok().json(reactor),
        Err(diesel::result::Error::NotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Reactor not found"}))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn create_reactor(
    pool: web::Data<DbPool>,
    body: web::Json<NewReactorDesign>,
) -> HttpResponse {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    match diesel::insert_into(reactor_designs::table)
        .values(&body.into_inner())
        .get_result::<ReactorDesign>(&mut conn)
    {
        Ok(reactor) => HttpResponse::Created().json(reactor),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn delete_reactor(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let reactor_id = path.into_inner();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    match diesel::delete(reactor_designs::table.find(reactor_id))
        .execute(&mut conn)
    {
        Ok(0) => HttpResponse::NotFound().json(serde_json::json!({"error": "Reactor not found"})),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
