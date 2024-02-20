
use base64::Engine;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, sync_version_wrappers, Account, AccountState, Capabilities, ContentProcessingId, ContentProcessingState, MessageNumber, ModerationQueueNumber, ModerationQueueType, Profile, ProfileVisibility, SyncVersion, SyncVersionUtils
};


sync_version_wrappers!(
    ReceivedBlocksSyncVersion,
    ReceivedLikesSyncVersion,
    SentBlocksSyncVersion,
    SentLikesSyncVersion,
    MatchesSyncVersion,
);
