use database::current::read::GetDbReadCommandsCommon;
use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{AccountIdInternal, EventToClientInternal, ProfileModificationMetadata};
use server_data::{
    DataError, app::EventManagerProvider, cache::profile::UpdateLocationCacheState, db_transaction,
    define_cmd_wrapper_write, read::DbRead, result::Result, write::DbTransaction,
};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfileAdminVerification);

impl WriteCommandsProfileAdminVerification<'_> {
    pub async fn change_profile_age_range_verified_value(
        &self,
        moderator_id: AccountIdInternal,
        account: AccountIdInternal,
        value: Option<bool>,
    ) -> Result<(), DataError> {
        let (profile_state, moderator_is_bot) = self
            .db_read(move |mut cmds| {
                let profile_state = cmds.profile().data().profile_state(account)?;
                let moderator_is_bot = cmds
                    .common()
                    .state()
                    .other_shared_state(moderator_id)
                    .map(|v| v.is_bot())?;
                Ok((profile_state, moderator_is_bot))
            })
            .await?;

        let value_already_set = if moderator_is_bot {
            profile_state.profile_age_range_verified == value
        } else {
            profile_state.profile_age_range_verified_manual == value
        };

        if value_already_set {
            return Ok(());
        }

        let modification = ProfileModificationMetadata::generate();
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .required_changes_for_profile_update(account, &modification)?;

            if moderator_is_bot {
                cmds.profile_admin()
                    .verification()
                    .change_profile_age_range_verified_value(account, value)?;
            } else {
                cmds.profile_admin()
                    .verification()
                    .change_profile_age_range_verified_manual_value(account, value)?;
            }

            Ok(())
        })?;

        self.write_cache_profile(account.as_id(), |p| {
            if moderator_is_bot {
                p.state.profile_age_range_verified = value;
            } else {
                p.state.profile_age_range_verified_manual = value;
            }
            p.update_profile_version_uuid(modification.version);
            p.state.profile_edited_time = modification.time;
            Ok(())
        })
        .await?;

        self.update_location_cache_profile(account).await?;

        self.event_manager()
            .send_connected_event(account, EventToClientInternal::ProfileChanged)
            .await?;

        Ok(())
    }
}
