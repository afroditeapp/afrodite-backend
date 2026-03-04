use std::fmt;

use crate::{
    apis::{
        Error, ResponseContent, chat_api::{GetPendingMessagesError, GetPublicKeyError, PostAddPublicKeyError, PostSendMessageError}, configuration, media_api::{GetContentError, PutContentToContentSlotError}
    },
    models::{AccountId, ContentId, Location, MediaContentType, SendMessageResult, UnixTime},
};

pub const RESPONSE_ERROR_STATUS_CODE_401: &str = "status code 401";

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.aid)
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cid)
    }
}

impl fmt::Display for UnixTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ut)
    }
}

impl Copy for Location {}

#[cfg(test)]
mod tests {
    use super::{Error, ResponseContent, RESPONSE_ERROR_STATUS_CODE_401};

    /// Admin and user bots require that this test passes
    #[test]
    fn response_error_display_includes_401_status_code() {
        let error: Error<()> = Error::ResponseError(ResponseContent {
            status: reqwest::StatusCode::UNAUTHORIZED,
            content: String::new(),
            entity: None,
        });
        let text = error.to_string().to_ascii_lowercase();
        assert!(
            text.contains(RESPONSE_ERROR_STATUS_CODE_401),
            "Expected '{}' in '{}'",
            RESPONSE_ERROR_STATUS_CODE_401,
            text
        );
    }
}
