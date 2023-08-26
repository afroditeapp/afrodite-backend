use error_stack::Result;
use model::{AccountIdInternal, HandleModerationRequest, Moderation};

use crate::data::DataError;

define_write_commands!(WriteCommandsMediaAdmin);

impl WriteCommandsMediaAdmin<'_> {
    pub async fn moderation_get_list_and_create_new_if_necessary(
        self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DataError> {
        self.db_write(move |cmds| {
            cmds.into_media_admin()
                .moderation_get_list_and_create_new_if_necessary(account_id)
        })
        .await
    }

    pub async fn update_moderation(
        self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DataError> {
        self.db_write(move |cmds| {
            cmds.into_media_admin().update_moderation(
                moderator_id,
                moderation_request_owner,
                result,
            )
        })
        .await
    }
}
