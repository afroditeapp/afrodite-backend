use model::{AccountIdInternal, HandleModerationRequest, Moderation};

use super::db_transaction;
use crate::{data::DataError, result::Result};

define_write_commands!(WriteCommandsMediaAdmin);

impl WriteCommandsMediaAdmin<'_> {
    pub async fn moderation_get_list_and_create_new_if_necessary(
        self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media_admin()
                .moderation()
                .moderation_get_list_and_create_new_if_necessary(account_id)
        })
    }

    pub async fn update_moderation(
        self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media_admin().moderation().update_moderation(
                moderator_id,
                moderation_request_owner,
                result,
            )
        })
    }
}
