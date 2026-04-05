use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use futures_util::StreamExt;
use std::str::FromStr;

use crate::models::{NewReactorDesign, ReactorDesign};
use crate::schema::reactor_designs;
use crate::DbPool;

// Maximum upload size: 10 MB
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

// A single reactor record from an uploaded file (JSON or CSV).
#[derive(Debug, serde::Deserialize)]
struct IngestRecord {
    name: String,
    design_type: String,
    vendor: Option<String>,
    thermal_power_mw: Option<f64>,
    electric_power_mw: Option<f64>,
    coolant_type: Option<String>,
    moderator: Option<String>,
    fuel_type: Option<String>,
    enrichment_pct: Option<f64>,
    design_metadata: Option<serde_json::Value>,
    source_url: Option<String>,
}

impl IngestRecord {
    fn into_new_reactor(self) -> NewReactorDesign {
        NewReactorDesign {
            name: self.name,
            design_type: self.design_type,
            vendor: self.vendor,
            thermal_power_mw: self.thermal_power_mw.map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_default()),
            electric_power_mw: self.electric_power_mw.map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_default()),
            coolant_type: self.coolant_type,
            moderator: self.moderator,
            fuel_type: self.fuel_type,
            enrichment_pct: self.enrichment_pct.map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_default()),
            design_metadata: self.design_metadata,
            source_url: self.source_url,
        }
    }
}

#[derive(serde::Serialize)]
struct IngestResponse {
    imported: usize,
    failed: usize,
    errors: Vec<String>,
    reactors: Vec<ReactorDesign>,
}

// POST /api/ingest  — upload a JSON or CSV file of reactor designs.
//
// Expects a multipart form with a single file field named `file`.
// The file extension or content-type determines parsing:
//   - `.json` / `application/json`: expects a JSON array of records
//   - `.csv`  / `text/csv`: expects a CSV with header row
pub async fn ingest_file(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> HttpResponse {
    // Read the first file field from the multipart body.
    let (filename, data) = match read_file_field(&mut payload).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    };

    // Determine format from filename extension.
    let records: Vec<IngestRecord> = if filename.ends_with(".csv") {
        match parse_csv(&data) {
            Ok(r) => r,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": format!("CSV parse error: {e}")})),
        }
    } else if filename.ends_with(".json") {
        match serde_json::from_slice::<Vec<IngestRecord>>(&data) {
            Ok(r) => r,
            Err(_) => {
                // Try as a single object
                match serde_json::from_slice::<IngestRecord>(&data) {
                    Ok(r) => vec![r],
                    Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": format!("JSON parse error: {e}")})),
                }
            }
        }
    } else {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Unsupported file type. Upload a .json or .csv file."}));
    };

    if records.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "File contains no reactor records."}));
    }

    // Insert each record, collecting successes and errors.
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut imported = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (i, record) in records.into_iter().enumerate() {
        let row_label = format!("row {}", i + 1);
        let name = record.name.clone();
        let new_reactor = record.into_new_reactor();

        match diesel::insert_into(reactor_designs::table)
            .values(&new_reactor)
            .get_result::<ReactorDesign>(&mut conn)
        {
            Ok(reactor) => imported.push(reactor),
            Err(e) => errors.push(format!("{row_label} ({name}): {e}")),
        }
    }

    let response = IngestResponse {
        imported: imported.len(),
        failed: errors.len(),
        errors,
        reactors: imported,
    };

    HttpResponse::Ok().json(response)
}

// Read the first file field from a multipart stream.
async fn read_file_field(payload: &mut Multipart) -> Result<(String, Vec<u8>), String> {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| format!("Multipart error: {e}"))?;

        let disposition = match field.content_disposition() {
            Some(d) => d.clone(),
            None => continue,
        };
        let field_name = disposition.get_name().unwrap_or("");
        if field_name != "file" {
            continue;
        }

        let filename = disposition
            .get_filename()
            .unwrap_or("upload")
            .to_lowercase();

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| format!("Read error: {e}"))?;
            if data.len() + chunk.len() > MAX_FILE_SIZE {
                return Err(format!("File exceeds maximum size of {} MB", MAX_FILE_SIZE / 1024 / 1024));
            }
            data.extend_from_slice(&chunk);
        }

        return Ok((filename, data));
    }
    Err("No file field found in upload.".to_string())
}

// Parse CSV bytes into IngestRecords.
fn parse_csv(data: &[u8]) -> Result<Vec<IngestRecord>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(data);

    let mut records = Vec::new();
    for (i, result) in reader.deserialize().enumerate() {
        let record: IngestRecord = result.map_err(|e| format!("row {}: {e}", i + 1))?;
        records.push(record);
    }
    Ok(records)
}
