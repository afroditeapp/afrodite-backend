

use diesel::{
    AsExpression, FromSqlRow, sql_types::BigInt,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::ToSchema;

use crate::{
    sync_version_wrappers, SyncVersion, SyncVersionUtils
};


sync_version_wrappers!(
    ReceivedBlocksSyncVersion,
    ReceivedLikesSyncVersion,
    SentBlocksSyncVersion,
    SentLikesSyncVersion,
    MatchesSyncVersion,
);
