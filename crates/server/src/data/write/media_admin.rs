use error_stack::Result;

use model::{AccountIdInternal, HandleModerationRequest, Moderation};

use crate::{data::DatabaseError, utils::ConvertCommandErrorExt};

define_write_commands!(WriteCommandsMediaAdmin);

impl WriteCommandsMediaAdmin<'_> {
    pub async fn moderation_get_list_and_create_new_if_necessary(
        self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DatabaseError> {
        self.current()
            .media_admin()
            .moderation_get_list_and_create_new_if_necessary(account_id)
            .await
            .convert(account_id)
    }

    pub async fn update_moderation(
        self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.current()
            .media_admin()
            .update_moderation(moderator_id, moderation_request_owner, result)
            .await
            .convert(moderator_id)
    }
}
