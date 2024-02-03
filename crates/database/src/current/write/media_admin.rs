use diesel::{delete, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ContentId, ContentIdDb, ContentState, HandleModerationRequest,
    Moderation, ModerationId, ModerationQueueNumber, ModerationRequestId,
    ModerationRequestState, ProfileContent, NextQueueNumberType, schema::media_moderation_request::content_id_1,
};
use simple_backend_database::diesel_db::{DieselConnection, DieselDatabaseError};

use super::{media::CurrentSyncWriteMedia, ConnectionProvider};
use crate::{IntoDatabaseError, TransactionError, current::write::CurrentSyncWriteCommands};

mod moderation;
mod media_content;

define_write_commands!(CurrentWriteMediaAdmin, CurrentSyncWriteMediaAdmin);

impl<C: ConnectionProvider> CurrentSyncWriteMediaAdmin<C> {
    pub fn moderation(self) -> moderation::CurrentSyncWriteMediaAdminModeration<C> {
        moderation::CurrentSyncWriteMediaAdminModeration::new(self.cmds)
    }

    pub fn media_content(self) -> media_content::CurrentSyncWriteMediaAdminMediaContent<C> {
        media_content::CurrentSyncWriteMediaAdminMediaContent::new(self.cmds)
    }
}
