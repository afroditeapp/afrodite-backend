use diesel::prelude::Queryable;
use model_server_data::ProfileStringModerationContentType;
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, UnixTime};
use utoipa::{IntoParams, ToSchema};

use crate::AccountId;

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct GetProfileStringIterationPageParams {
    pub content_type: ProfileStringModerationContentType,
    /// Zero-based page number.
    pub page: i64,
    /// Iteration origin point. Max edited time value which can be returned.
    /// Descending iteration starts from this time. Values are always
    /// returned in descending edited time order.
    pub origin_unix_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ProfileStringIterationValue {
    pub id: AccountId,
    pub value: NonEmptyString,
    pub edited_time: UnixTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileStringIterationPage {
    pub values: Vec<ProfileStringIterationValue>,
}
