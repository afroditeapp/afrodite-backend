use std::path::PathBuf;

use diesel::{Connection, prelude::*};
use error_stack::{Result, ResultExt};
use simple_backend_config::{Database, SimpleBackendConfig};
use simple_backend_utils::{ContextExt, db::MyDbConnection};

use super::DieselDatabaseError;

pub async fn create_connection(
    config: &SimpleBackendConfig,
    database_info: &Database,
    db_path: PathBuf,
) -> Result<MyDbConnection, DieselDatabaseError> {
    if let Some(postgres_config) = &config.database_config().postgres {
        let url = match database_info {
            Database::Current => &postgres_config.current,
            Database::History => &postgres_config.history,
        };

        let conn =
            PgConnection::establish(url.as_str()).change_context(DieselDatabaseError::Connect)?;

        return Ok(MyDbConnection::Pg(conn));
    }

    let db_str = if config.sqlite_in_ram() {
        let name = database_info.sqlite_name();
        for c in name.chars() {
            if !c.is_ascii_alphanumeric() {
                return Err(DieselDatabaseError::Connect.report())
                    .attach_printable("Database name is not ASCII alphanumeric");
            }
        }
        format!("file:{name}?mode=memory&cache=shared")
    } else {
        db_path.to_string_lossy().to_string()
    };

    let mut conn =
        SqliteConnection::establish(&db_str).change_context(DieselDatabaseError::Connect)?;
    sqlite_setup_connection(&mut conn)?;

    Ok(MyDbConnection::Sqlite(conn))
}

fn sqlite_setup_connection(conn: &mut SqliteConnection) -> Result<(), DieselDatabaseError> {
    let pragmas = &[
        "PRAGMA journal_mode=WAL;",
        "PRAGMA synchronous=NORMAL;",
        "PRAGMA foreign_keys=ON;",
        "PRAGMA secure_delete=ON;",
    ];

    for pragma_str in pragmas.iter() {
        diesel::sql_query(*pragma_str)
            .execute(conn)
            .change_context(DieselDatabaseError::Setup)?;
    }

    Ok(())
}
