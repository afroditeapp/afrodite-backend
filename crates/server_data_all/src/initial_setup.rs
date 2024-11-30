use config::Config;
use model::{Account, AccountIdInternal, AccountState, EmailMessages, EventToClientInternal, Permissions};
use server_data::{db_manager::RouterDatabaseReadHandle, read::GetReadCommandsCommon, result::WrappedContextExt, write_commands::WriteCommandRunnerHandle, DataError};
use server_data_account::{read::GetReadCommandsAccount, write::{account::IncrementAdminAccessGrantedCount, GetWriteCommandsAccount}};
use server_data_profile::write::GetWriteCommandsProfile;
use tracing::warn;

pub async fn complete_initial_setup(
    config: &Config,
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    id: AccountIdInternal,
) -> server_common::result::Result<Account, server_common::data::DataError> {
    let account_data = read_handle.account().account_data(id).await?;
    let sign_in_with_info = read_handle.account().account_sign_in_with_info(id).await?;
    let (matches_with_grant_admin_access_config, grant_admin_access_more_than_once) =
        if let Some(grant_admin_access_config) = config.grant_admin_access_config() {
            let matches = match (
                grant_admin_access_config.email.as_ref(),
                grant_admin_access_config.google_account_id.as_ref(),
            ) {
                (wanted_email @ Some(_), Some(wanted_google_account_id)) => {
                    wanted_email == account_data.email.as_ref()
                        && sign_in_with_info
                            .google_account_id_matches_with(wanted_google_account_id)
                }
                (wanted_email @ Some(_), None) => wanted_email == account_data.email.as_ref(),
                (None, Some(wanted_google_account_id)) => {
                    sign_in_with_info.google_account_id_matches_with(wanted_google_account_id)
                }
                (None, None) => false,
            };

            (
                matches,
                grant_admin_access_config.for_every_matching_new_account,
            )
        } else {
            (false, false)
        };

    let is_bot_account = read_handle.account().is_bot_account(id).await?;

    let new_account = write_handle.write(move |cmds| async move {
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

        // TODO(microservice): API for setting initial profile age
        cmds.profile().set_initial_profile_age_from_current_profile(id).await?;

        let global_state = cmds.read().account().global_state().await?;
        let enable_all_permissions = if matches_with_grant_admin_access_config
            && (global_state.admin_access_granted_count == 0 || grant_admin_access_more_than_once)
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
                move |state, permissions, _| {
                    if *state == AccountState::InitialSetup {
                        *state = AccountState::Normal;
                        if enable_all_permissions.is_some() {
                            warn!("Account detected as admin account. Enabling all permissions");
                            *permissions = Permissions::all_enabled();
                        }
                    }
                    Ok(())
                },
            )
            .await?;

        if !is_bot_account && !sign_in_with_info.some_sign_in_with_method_is_set() {
            // Account registered email is not yet sent if email address
            // was provided manually and not from some sign in with method.
            cmds.account().email().send_email_if_not_already_sent(id, EmailMessages::AccountRegistered).await?;
        }

        cmds.events()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountStateChanged(new_account.state()),
            )
            .await?;

        cmds.events()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountPermissionsChanged(new_account.permissions()),
            )
            .await?;

        Ok(new_account)
    }).await?;

    Ok(new_account)
}
