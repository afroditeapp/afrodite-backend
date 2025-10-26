use diesel::{Connection, RunQueryDsl, prelude::*};
use error_stack::{Result, ResultExt};

use crate::ComponentError;

impl ComponentError for DieselDatabaseError {
    const COMPONENT_NAME: &'static str = "Diesel";
}

#[derive(thiserror::Error, Debug)]
pub enum DieselDatabaseError {
    #[error("Connecting to SQLite database failed")]
    Connect,
    #[error("SQLite connection setup failed")]
    Setup,
    #[error("Executing SQL query failed")]
    Execute,
    #[error("Running diesel database migrations failed")]
    Migrate,

    #[error("Running an action failed")]
    RunAction,
    #[error("Add connection to pool failed")]
    AddConnection,
    #[error("Connection get failed from connection pool")]
    GetConnection,
    #[error("Interaction with database connection failed")]
    InteractionError,

    #[error("SQLite version query failed")]
    SqliteVersionQuery,

    #[error("Creating in RAM database failed")]
    CreateInRam,

    #[error("Deserializing failed")]
    SerdeDeserialize,
    #[error("Serializing failed")]
    SerdeSerialize,

    #[error("Content slot not empty")]
    ContentSlotNotEmpty,
    #[error("Content slot empty")]
    ContentSlotEmpty,
    #[error("Moderation request content invalid")]
    ModerationRequestContentInvalid,
    #[error("Moderation request is missing")]
    MissingModerationRequest,

    #[error("Not found")]
    NotFound,
    #[error("Operation is not allowed")]
    NotAllowed,
    #[error("Action is already done")]
    AlreadyDone,
    #[error("No available IDs")]
    NoAvailableIds,

    #[error("Data format conversion failed")]
    DataFormatConversion,

    #[error("Transaction failed")]
    FromDieselErrorToTransactionError,

    #[error("File operation failed")]
    File,

    #[error("Zip file related error")]
    Zip,

    #[error("Diesel error")]
    DieselError,

    #[error("Transaction error")]
    FromStdErrorToTransactionError,

    #[error("Message encryption error")]
    MessageEncryptionError,
}

mod sqlite_version {
    use diesel::define_sql_function;
    define_sql_function! { fn sqlite_version() -> Text }
}

#[derive(diesel::MultiConnection)]
pub enum MyDbConnection {
    Sqlite(SqliteConnection),
    Pg(PgConnection),
}

impl MyDbConnection {
    pub fn sqlite_version(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        match self {
            MyDbConnection::Sqlite(conn) => {
                let sqlite_version: Vec<String> = diesel::select(sqlite_version::sqlite_version())
                    .load(conn)
                    .change_context(DieselDatabaseError::Execute)?;
                Ok(sqlite_version.first().cloned())
            }
            MyDbConnection::Pg(_) => Ok(None),
        }
    }
}

/// [MyDbConnection] specific version of [diesel::query_dsl::RunQueryDsl]
/// to workaround missing ON CONFLICT support in [MyDbConnection].
pub trait MyRunQueryDsl: RunQueryDsl<MyDbConnection> {
    fn execute_my_conn(self, conn: &mut MyDbConnection) -> QueryResult<usize>
    where
        Self: diesel::query_dsl::methods::ExecuteDsl<SqliteConnection>
            + diesel::query_dsl::methods::ExecuteDsl<PgConnection>,
    {
        match conn {
            MyDbConnection::Sqlite(conn) => {
                diesel::query_dsl::methods::ExecuteDsl::execute(self, conn)
            }
            MyDbConnection::Pg(conn) => diesel::query_dsl::methods::ExecuteDsl::execute(self, conn),
        }
    }

    fn get_result_my_conn<'query, U>(self, conn: &mut MyDbConnection) -> QueryResult<U>
    where
        Self: diesel::query_dsl::methods::LoadQuery<'query, SqliteConnection, U>
            + diesel::query_dsl::methods::LoadQuery<'query, PgConnection, U>,
    {
        match conn {
            MyDbConnection::Sqlite(conn) => match self.internal_load(conn)?.next() {
                Some(v) => v,
                None => Err(diesel::result::Error::NotFound),
            },
            MyDbConnection::Pg(conn) => match self.internal_load(conn)?.next() {
                Some(v) => v,
                None => Err(diesel::result::Error::NotFound),
            },
        }
    }
}

impl<T: RunQueryDsl<MyDbConnection>> MyRunQueryDsl for T {}
