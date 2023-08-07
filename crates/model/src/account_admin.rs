use base64::Engine;
use diesel::{deserialize::FromSql, sql_types::Binary, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct CurrentVersions {
    pub versions: String,
}
