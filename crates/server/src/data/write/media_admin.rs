use database::current::write::media_admin::InitialModerationRequestIsNowAccepted;
use model::{AccountIdInternal, HandleModerationRequest, Moderation, ModerationQueueType, ProfileVisibility};

use super::db_transaction;
use crate::{data::DataError, result::Result};

define_write_commands!(WriteCommandsMediaAdmin);

// TODO(prod): Move event sending to WriteCommands instead of route handlers
//             to avoid disappearing events in case client disconnects before
//             event is sent.

pub struct UpdateModerationInfo {
    pub new_visibility: Option<ProfileVisibility>,
    pub initial_request_accepted: Option<InitialModerationRequestIsNowAccepted>,
    location_cache_should_be_updated: Option<ProfileVisibility>,
}

impl WriteCommandsMediaAdmin<'_> {
    pub async fn moderation_get_list_and_create_new_if_necessary(
        self,
        account_id: AccountIdInternal,
        queue: ModerationQueueType,
    ) -> Result<Vec<Moderation>, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media_admin()
                .moderation()
                .moderation_get_list_and_create_new_if_necessary(account_id, queue)
        })
    }

    /// Updates moderation request and if needed, updates profile visibility to
    /// this server instance.
    pub async fn update_moderation(
        self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<UpdateModerationInfo, DataError> {
        let account_component_enabled = self.config().components().account;
        let info = db_transaction!(self, move |mut cmds| {
            let initial_request_accepted_status = cmds.media_admin().moderation().update_moderation(
                moderator_id,
                moderation_request_owner,
                result,
            )?;

            // If needed, do profile visibility update here to avoid broken
            // state if server crashes right after current transaction.

            if initial_request_accepted_status.is_some() && account_component_enabled {
                let account = cmds.read().common().account(moderation_request_owner)?;
                let visibility = account.profile_visibility();
                let new_visibility = match visibility {
                    ProfileVisibility::Public => ProfileVisibility::Public,
                    ProfileVisibility::Private => ProfileVisibility::Private,
                    ProfileVisibility::PendingPublic => ProfileVisibility::Public,
                    ProfileVisibility::PendingPrivate => ProfileVisibility::Private,
                };
                cmds.common().state().update_syncable_account_data(moderation_request_owner, account, move |_, _, visibility| {
                    *visibility = new_visibility;
                    Ok(())
                })?;

                let location_cache_should_be_updated = if visibility.is_currently_public() != new_visibility.is_currently_public() {
                    Some(new_visibility)
                } else {
                    None
                };

                Ok(UpdateModerationInfo {
                    new_visibility: Some(new_visibility),
                    initial_request_accepted: initial_request_accepted_status,
                    location_cache_should_be_updated,
                })
            } else {
                Ok(UpdateModerationInfo {
                    new_visibility: None,
                    initial_request_accepted: initial_request_accepted_status,
                    location_cache_should_be_updated: None,
                })
            }
        })?;

        if let Some(visibility) = info.location_cache_should_be_updated {
            self.cmds.account().profile_update_visibility(moderation_request_owner, visibility.is_currently_public()).await?;
        }

        Ok(info)
    }
}
