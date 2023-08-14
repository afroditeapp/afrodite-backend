use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use diesel::prelude::*;

use crate::{AccountIdInternal, AccountIdLight, ModerationRequestContent, AccountIdDb, ModerationRequestIdDb};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ModerationList {
    pub list: Vec<Moderation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct HandleModerationRequest {
    pub accept: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ModerationId {
    pub request_id: ModerationRequestId,
    /// Moderator AccountId
    pub account_id: AccountIdInternal,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_moderation)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaModerationRaw {
    pub account_id: AccountIdDb,
    pub moderation_request_id: ModerationRequestIdDb,
    pub state_number: i64,
    pub json_text: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Moderation {
    pub request_creator_id: AccountIdLight,
    pub request_id: ModerationRequestId,
    pub moderator_id: AccountIdLight,
    pub content: ModerationRequestContent,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ModerationRequestId {
    pub request_row_id: ModerationRequestIdDb,
}
