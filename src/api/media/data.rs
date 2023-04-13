use serde::{Deserialize, Serialize};
use tower::util::Optional;
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
    image1: String,
    /// Include slot 2 image in moderation request.
    image2: Option<String>,
    /// Include slot 3 image in moderation request.
    image3: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequest {
    moderation_request_id: i64,
    account_id: AccountIdLight,
    state_number: ModerationRequestState,
    request: NewModerationRequest,
}

impl ModerationRequest {
    pub fn new(moderation_request_id: i64, account_id: AccountIdLight, state: ModerationRequestState, request: NewModerationRequest) -> Self { Self { moderation_request_id, account_id, state_number: state, request } }
}

#[derive(thiserror::Error, Debug)]
pub enum ModerationRequestStateParsingError {
    #[error("ParsingFailed, value: {0}")]
    ParsingError(i64),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[repr(i64)]
pub enum ModerationRequestState {
    Waiting = 0,
    InProgress = 1,
    Accepted = 2,
    Denied = 3,
}

impl ModerationRequestState {
    pub fn completed(&self) -> bool {
        match self {
            Self::Accepted | Self::Denied => true,
            _ => false,
        }
    }
}

impl TryFrom<i64> for ModerationRequestState {
    type Error = ModerationRequestStateParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::Waiting,
            1 => Self::InProgress,
            2 => Self::Accepted,
            3 => Self::Denied,
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

#[derive(Debug, Copy, Clone)]
pub struct ModerationId {
    pub moderation_row_id: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct ModerationRequestId {
    pub request_row_id: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct ModerationRequestQueueNumber {
    pub number: i64,
}
