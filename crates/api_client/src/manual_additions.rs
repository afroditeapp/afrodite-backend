use std::fmt;

use crate::{
    apis::{
        Error, ResponseContent, chat_api::{GetPendingMessagesError, GetPublicKeyError, PostAddPublicKeyError, PostSendMessageError}, configuration, media_api::{GetContentError, PutContentToContentSlotError}
    },
    models::{AccountId, ContentId, Location, MediaContentType, SendMessageResult, UnixTime},
};

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
