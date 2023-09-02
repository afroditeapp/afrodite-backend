
use base64::Engine;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::macros::{diesel_i64_wrapper, diesel_uuid_wrapper};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BackendConfig {
    pub bots: Option<BotConfig>
}

/// Enable automatic bots when server starts.
/// Editing of this field with edit module is only allowed when
/// this exists in the config file.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct BotConfig {
    /// User bot count
    pub users: u32,
    /// Admin bot count
    pub admins: u32,
}
