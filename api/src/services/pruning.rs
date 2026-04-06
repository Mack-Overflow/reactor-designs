use chrono::Utc;
use diesel::prelude::*;
use crate::DbPool;
use crate::schema::simulation_runs;

/// Deletes simulation runs whose `started_at` timestamp is older than
/// `retention_days` ago. Cascades automatically to `simulation_results`
/// and `waste_isotope_inventory` via DB foreign-key constraints.
///
/// Returns the number of simulation runs deleted.
pub fn prune_old_simulations(
    pool: &DbPool,
    retention_days: i64,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get()?;
    let cutoff = Utc::now() - chrono::Duration::days(retention_days);

    let deleted = diesel::delete(
        simulation_runs::table.filter(simulation_runs::started_at.lt(cutoff)),
    )
    .execute(&mut conn)?;

    Ok(deleted)
}

/// Returns the `std::time::Duration` until the next UTC midnight.
pub fn duration_until_next_midnight() -> std::time::Duration {
    let now = Utc::now();
    let tomorrow = now
        .date_naive()
        .succ_opt()
        .expect("date overflow is not a concern here");
    let midnight = tomorrow
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always valid")
        .and_utc();
    (midnight - now)
        .to_std()
        .unwrap_or(std::time::Duration::from_secs(86_400))
}
