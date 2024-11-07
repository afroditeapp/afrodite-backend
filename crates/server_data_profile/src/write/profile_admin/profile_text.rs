use server_data::{define_server_data_write_commands, result::WrappedContextExt};

use model::{
    AccountIdInternal, ProfileTextModerationRejectedReasonCategory, ProfileTextModerationRejectedReasonDetails, ProfileVersion
};
use server_data::{
    result::Result,
    write::WriteCommandsProvider,
    DataError, IntoDataError,
};

define_server_data_write_commands!(WriteCommandsProfileAdminProfileText);
define_db_read_command_for_write!(WriteCommandsProfileAdminProfileText);
define_db_transaction_command!(WriteCommandsProfileAdminProfileText);

impl<C: WriteCommandsProvider> WriteCommandsProfileAdminProfileText<C> {
    pub async fn moderate_profile_text(
        self,
        moderator_id: AccountIdInternal,
        name_owner_id: AccountIdInternal,
        text: String,
        accept: bool,
        rejected_category: Option<ProfileTextModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileTextModerationRejectedReasonDetails>,
    ) -> Result<(), DataError> {
        let current_profile = self.db_read(move |mut cmds| cmds.profile().data().profile(name_owner_id)).await?;
        let current_profile_state = self.db_read(move |mut cmds| cmds.profile().data().profile_state(name_owner_id)).await?;
        if current_profile.ptext != text {
            return Err(DataError::NotAllowed.report());
        }
        if current_profile_state.profile_text_moderation_state.is_moderated() {
            return Err(DataError::NotAllowed.report());
        }
        if accept {
            // Profile text accepted value is part of Profile, so update it's version
            let new_profile_version = ProfileVersion::new_random();
            let (account, new_state) = db_transaction!(self, move |mut cmds| {
                cmds.profile().data().only_profile_version(name_owner_id, new_profile_version)?;
                cmds.profile().data().increment_profile_sync_version(name_owner_id)?;
                let new_state = cmds.profile_admin().profile_text().moderate_profile_text(
                    moderator_id,
                    name_owner_id,
                    accept,
                    rejected_category,
                    rejected_details,
                )?;
                let account = cmds.read().common().account(name_owner_id)?;
                Ok((account, new_state))
            })?;

            let location_and_profile_data = self
                .cache()
                .write_cache(name_owner_id.as_id(), |e| {
                    if let Some(p) = e.profile.as_mut() {
                        p.state.profile_text_moderation_state = new_state;
                        p.data.version_uuid = new_profile_version;
                        Ok(Some((p.location.current_position, e.location_index_profile_data()?)))
                    } else {
                        Ok(None)
                    }
                })
                .await
                .into_data_error(name_owner_id)?;

            if account.profile_visibility().is_currently_public() {
                if let Some((location, profile_data)) = location_and_profile_data {
                    self.location()
                        .update_profile_data(name_owner_id.as_id(), profile_data, location)
                        .await?;
                }
            }
        } else {
            db_transaction!(self, move |mut cmds| {
                cmds.profile_admin().profile_text().moderate_profile_text(
                    moderator_id,
                    name_owner_id,
                    accept,
                    rejected_category,
                    rejected_details,
                )
            })?;
        }

        Ok(())
    }
}
