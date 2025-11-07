use config::Config;
use model::{
    Account, AccountIdInternal, AccountState, ContentIdInternal, EmailMessages,
    EventToClientInternal, Permissions,
};
use server_data::{
    DataError, cache::profile::UpdateLocationCacheState, db_manager::RouterDatabaseReadHandle,
    read::GetReadCommandsCommon, result::WrappedContextExt, write::GetWriteCommandsCommon,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_account::{
    read::GetReadCommandsAccount,
    write::{GetWriteCommandsAccount, account::IncrementAdminAccessGrantedCount},
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use server_data_profile::write::GetWriteCommandsProfile;
use tracing::warn;

pub async fn complete_initial_setup(
    config: &Config,
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    id: AccountIdInternal,
) -> server_common::result::Result<Account, server_common::data::DataError> {
    let email_address_state = read_handle.account().email_address_state(id).await?;
    let sign_in_with_info = read_handle.account().account_sign_in_with_info(id).await?;
    let (matches_with_grant_admin_access_config, grant_admin_access_more_than_once) =
        if let (Some(grant_admin_access_config), Some(email)) = (
            config.grant_admin_access_config(),
            email_address_state.email.as_ref(),
        ) {
            let matches = if grant_admin_access_config.debug_match_only_email_domain {
                let mut wanted_email_iter = grant_admin_access_config.email.0.split('@');
                let mut email_iter = email.0.split('@');
                wanted_email_iter.next();
                email_iter.next();
                let wanted_domain = wanted_email_iter.next();
                let email_domain = email_iter.next();
                if wanted_email_iter.next().is_some() || email_iter.next().is_some() {
                    // Multiple '@' characters
                    false
                } else if wanted_domain.is_none() || email_domain.is_none() {
                    // Missing '@' character
                    false
                } else {
                    wanted_domain == email_domain
                }
            } else {
                grant_admin_access_config.email == *email
            };

            (
                matches,
                grant_admin_access_config.debug_for_every_matching_new_account,
            )
        } else {
            (false, false)
        };

    let is_bot_account = read_handle.account().is_bot_account(id).await?;

    let new_account = write_handle
        .write(move |cmds| async move {
            // Second account state check as db_write quarantees synchronous
            // access.
            let account_state = cmds.read().common().account(id).await?.state();
            if account_state != AccountState::InitialSetup {
                return Err(DataError::NotAllowed.report());
            }

            let account_setup = cmds.read().account().account_setup(id).await?;
            if !account_setup.is_valid() {
                return Err(DataError::NotAllowed.report());
            }

            cmds.profile()
                .set_initial_profile_age_from_current_profile(id)
                .await?;

            cmds.common()
                .update_initial_setup_completed_unix_time(id)
                .await?;

            let global_state = cmds.read().account().global_state().await?;
            let enable_all_permissions = if matches_with_grant_admin_access_config
                && (global_state.admin_access_granted_count == 0
                    || grant_admin_access_more_than_once)
            {
                Some(IncrementAdminAccessGrantedCount)
            } else {
                None
            };

            let new_account = cmds
                .account()
                .update_syncable_account_data(
                    id,
                    enable_all_permissions,
                    move |state, permissions, _, _| {
                        if state.account_state() == AccountState::InitialSetup {
                            state.complete_initial_setup();
                            if enable_all_permissions.is_some() {
                                warn!(
                                    "Account detected as admin account. Enabling all permissions"
                                );
                                *permissions = Permissions::all_enabled();
                            }
                        }
                        Ok(())
                    },
                )
                .await?;

            // Update initial setup completed time to profile index
            (&cmds.profile()).update_location_cache_profile(id).await?;

            if !is_bot_account && !sign_in_with_info.some_sign_in_with_method_is_set() {
                // Email verification email is not yet sent if email address
                // was provided manually and not from some sign in with method.
                cmds.account()
                    .email()
                    .send_email_if_not_already_sent(id, EmailMessages::EmailVerification)
                    .await?;
            }

            cmds.events()
                .send_connected_event(id.uuid, EventToClientInternal::AccountStateChanged)
                .await?;

            let current_content = cmds.read().media().all_account_media_content(id).await?;
            for c in current_content {
                if c.moderation_state.is_in_slot() {
                    let content_id = ContentIdInternal::new(id, c.content_id(), c.content_row_id());
                    cmds.media().delete_content(content_id).await?;
                }
            }

            Ok(new_account)
        })
        .await?;

    Ok(new_account)
}
