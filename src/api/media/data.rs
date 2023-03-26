use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::api::model::AccountIdLight;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ImageFileName {
    image_file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ImageFile {
    #[schema(value_type = String, format = Binary)]
    data: Vec<u8>,
}


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct NewModerationRequest {
    camera: Option<bool>,
    image1: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequest {
    moderation_request_id: uuid::Uuid,
    account_id: AccountIdLight,
    queue_position: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequestList {
    list: Vec<ModerationRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct HandleModerationRequest {
    accept: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotId {
    slot_id: String,
}
