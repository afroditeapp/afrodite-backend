use std::path::PathBuf;

use diesel::{Connection, prelude::*};
use error_stack::{Result, ResultExt};
use simple_backend_config::{SimpleBackendConfig, SqliteDatabase};
use simple_backend_utils::{ContextExt, db::MyDbConnection};

use super::{DieselDatabaseError, sqlite_setup_connection};

pub async fn create_connection(
    config: &SimpleBackendConfig,
    database_info: &SqliteDatabase,
    db_path: PathBuf,
) -> Result<MyDbConnection, DieselDatabaseError> {
    let db_str = if config.sqlite_in_ram() {
        for c in database_info.name.chars() {
            if !c.is_ascii_alphanumeric() {
                return Err(DieselDatabaseError::Connect.report())
                    .attach_printable("Database name is not ASCII alphanumeric");
            }
        }
        format!("file:{}?mode=memory&cache=shared", database_info.name)
    } else {
        db_path.to_string_lossy().to_string()
    };

    let mut conn =
        SqliteConnection::establish(&db_str).change_context(DieselDatabaseError::Connect)?;
    sqlite_setup_connection(&mut conn)?;

    Ok(MyDbConnection::Sqlite(conn))
}
