use diesel::{delete, insert_into, prelude::*, update};
use error_stack::{Result, ResultExt, report};
use model::{
    AccountIdInternal, ContentId, ContentIdDb, ContentState, ContentSlot, ModerationQueueNumber,
    ModerationRequestContent, ProfileContent, NextQueueNumbersRaw, ModerationRequestState, NextQueueNumberType, NewContentParams, SetProfileContent, MediaContentInternal, MediaContentType, PendingProfileContent, SetProfileContentInternal,
};
use simple_backend_database::diesel_db::{DieselConnection, DieselDatabaseError};
use simple_backend_utils::ContextExt;

use super::ConnectionProvider;
use crate::{IntoDatabaseError, TransactionError};

mod media_content;
mod moderation_request;

define_write_commands!(CurrentWriteMedia, CurrentSyncWriteMedia);

pub struct DeletedSomething;

impl<C: ConnectionProvider> CurrentSyncWriteMedia<C> {
    pub fn media_content(self) -> media_content::CurrentSyncWriteMediaContent<C> {
        media_content::CurrentSyncWriteMediaContent::new(self.cmds)
    }

    pub fn moderation_request(self) -> moderation_request::CurrentSyncWriteMediaModerationRequest<C> {
        moderation_request::CurrentSyncWriteMediaModerationRequest::new(self.cmds)
    }
}
