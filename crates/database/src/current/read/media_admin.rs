use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, MediaModerationRaw, Moderation, ModerationId, ModerationRequestContent,
    ModerationRequestId, MediaModerationRequestRaw, ModerationRequestState,
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

mod moderation_request;
mod moderation;

define_read_commands!(CurrentReadMediaAdmin, CurrentSyncReadMediaAdmin);

impl<C: ConnectionProvider> CurrentSyncReadMediaAdmin<C> {
    pub fn moderation_request(self) -> moderation_request::CurrentSyncReadMediaAdminModerationRequest<C> {
        moderation_request::CurrentSyncReadMediaAdminModerationRequest::new(self.cmds)
    }

    pub fn moderation(self) -> moderation::CurrentSyncReadMediaAdminModeration<C> {
        moderation::CurrentSyncReadMediaAdminModeration::new(self.cmds)
    }
}
