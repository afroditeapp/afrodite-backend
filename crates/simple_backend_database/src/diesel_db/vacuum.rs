use std::path::Path;

use diesel::RunQueryDsl;
use error_stack::{Result, ResultExt};
use nix::sys::statvfs::statvfs;
use simple_backend_config::file::SqliteVacuumConfig;
use simple_backend_utils::db::DieselDatabaseError;
use tracing::{error, info};

use super::MyDbConnection;

struct DatabaseFileInfo {
    age_seconds: u64,
    size_bytes: i64,
}

/// Get database file age and size
fn get_database_file_info(db_path: &Path) -> Result<DatabaseFileInfo, DieselDatabaseError> {
    let metadata = std::fs::metadata(db_path).change_context(DieselDatabaseError::File)?;

    let created = metadata
        .created()
        .change_context(DieselDatabaseError::File)?;

    let age_seconds = created
        .elapsed()
        .change_context(DieselDatabaseError::File)?
        .as_secs();

    Ok(DatabaseFileInfo {
        age_seconds,
        size_bytes: metadata.len() as i64,
    })
}

/// Get free space in database in bytes
fn free_space_in_db(
    sqlite_conn: &mut diesel::SqliteConnection,
) -> Result<i64, DieselDatabaseError> {
    #[derive(diesel::QueryableByName)]
    struct FreelistCount {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        freelist_count: i64,
    }

    #[derive(diesel::QueryableByName)]
    struct PageSize {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        page_size: i64,
    }

    let freelist_count: FreelistCount = diesel::sql_query("PRAGMA freelist_count;")
        .get_result(sqlite_conn)
        .change_context(DieselDatabaseError::Execute)?;

    let page_size: PageSize = diesel::sql_query("PRAGMA page_size;")
        .get_result(sqlite_conn)
        .change_context(DieselDatabaseError::Execute)?;

    Ok(freelist_count.freelist_count * page_size.page_size)
}

/// Returns true if there is sufficient disk space to run VACUUM
fn is_disk_space_available(
    db_path: &Path,
    required_space: i64,
    db_name: &str,
) -> Result<bool, DieselDatabaseError> {
    let stat = statvfs(db_path).change_context(DieselDatabaseError::File)?;

    let available_space = stat.blocks_available() as u64 * stat.block_size();

    if available_space >= required_space as u64 {
        Ok(true)
    } else {
        error!(
            "Insufficient disk space to run VACUUM for '{}' DB: available={}MB, required={}MB",
            db_name,
            available_space / 1024 / 1024,
            required_space / 1024 / 1024
        );
        Ok(false)
    }
}

pub fn run_sqlite_vacuum_if_needed(
    conn: &mut MyDbConnection,
    db_path: &Path,
    config: &SqliteVacuumConfig,
    db_name: &str,
) -> Result<(), DieselDatabaseError> {
    if let MyDbConnection::Sqlite(sqlite_conn) = conn {
        let db_info = get_database_file_info(db_path)?;
        let free_space_bytes = free_space_in_db(sqlite_conn)?;

        let min_wait_passed = db_info.age_seconds >= config.min_wait_time.seconds as u64;
        let max_wait_exceeded = db_info.age_seconds >= config.max_wait_time.seconds as u64;
        let free_space_exceeded = free_space_bytes >= config.max_free_space.bytes();
        let should_run_vacuum = (min_wait_passed && free_space_exceeded) || max_wait_exceeded;

        if !should_run_vacuum {
            return Ok(());
        }

        if is_disk_space_available(db_path, db_info.size_bytes, db_name)? {
            info!(
                "VACUUM conditions met for '{}' DB: age={}d, free_space={}MB, total_size={}MB",
                db_name,
                db_info.age_seconds / (60 * 60 * 24),
                free_space_bytes / 1024 / 1024,
                db_info.size_bytes / 1024 / 1024
            );
            diesel::sql_query("VACUUM;")
                .execute(sqlite_conn)
                .change_context(DieselDatabaseError::Execute)?;
            info!("VACUUM completed successfully for '{db_name}' DB");
        }
    }

    Ok(())
}
