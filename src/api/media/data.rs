use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{api::model::AccountIdLight, server::database::file::file::ImageSlot};

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
    /// Use slot 1 image as camera image.
    camera_image: bool,
    /// Include slot 1 image in moderation request.
    image1: bool,
    /// Include slot 2 image in moderation request.
    image2: bool,
    /// Include slot 3 image in moderation request.
    image3: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequest {
    moderation_request_id: i64,
    account_id: AccountIdLight,
    state_number: ModerationRequestState,
    request: NewModerationRequest,
}

impl ModerationRequest {
    pub fn new(moderation_request_id: i64, account_id: AccountIdLight, queue_number: ModerationRequestState, request: NewModerationRequest) -> Self { Self { moderation_request_id, account_id, state_number: queue_number, request } }
}

#[derive(thiserror::Error, Debug)]
pub enum ModerationRequestStateParsingError {
    #[error("ParsingFailed, value: {0}")]
    ParsingError(i64),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[repr(i64)]
pub enum ModerationRequestState {
    InQueue0 = 0,
    InQueue1 = 1,
    InQueue2 = 2,
    InQueue3 = 3,
    HandledAccepted = 10,
    HandledDenied = 11,
}

impl TryFrom<i64> for ModerationRequestState {
    type Error = ModerationRequestStateParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::InQueue0,
            1 => Self::InQueue1,
            2 => Self::InQueue2,
            3 => Self::InQueue3,
            10 => Self::HandledAccepted,
            11 => Self::HandledDenied,
            _ => return Err(ModerationRequestStateParsingError::ParsingError(value)),
        };

        Ok(value)
    }
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
