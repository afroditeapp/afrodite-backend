use database::current::write::chat::CurrentSyncWriteChat;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, Location, ProfileLink, ProfileUpdateInternal, AccountInteractionInternal};

use crate::data::{cache::CacheError, DataError, IntoDataError, index::location::LocationIndexIteratorState};


define_write_commands!(WriteCommandsChat);

impl WriteCommandsChat<'_> {
    pub async fn like_profile(
        &mut self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
    ) -> Result<(), DataError> {
        let interaction = self.db_write(move |cmds| cmds.into_chat().get_or_create_account_interaction(id_like_sender, id_like_receiver))
            .await?;
        if interaction.is_like() {
            return Ok(());
        }
        let updated = interaction
            .try_into_like(id_like_sender, id_like_receiver)
            .change_context(DataError::NotAllowed)?;
        self.db_write(move |cmds|
            cmds.into_chat().update_account_interaction(updated)
        )
            .await?;

        Ok(())
    }
}
