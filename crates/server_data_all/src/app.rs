use axum::extract::ws::WebSocket;
use config::Config;
use futures::{FutureExt, future::BoxFuture};
use model::{
    Account, AccountIdInternal, AccountVerificationErrorFlags, AccountVerificationErrorFlagsValue,
    ClientMessageForDataAllCrate, EditVerificationValues, EmailMessages, UnixTime,
    VerificationMethod,
};
use model_account::{EmailAddress, SignInWithInfo};
use server_common::websocket::WebSocketError;
use server_data::{
    DataError, app::DataAllUtils, data_export::DataExportCmd, data_reset::BACKEND_DATA_RESET_STATE,
    db_manager::RouterDatabaseReadHandle, result::WrappedContextExt,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_account::write::GetWriteCommandsAccount;
use server_data_chat::read::GetReadChatCommands;
use simple_backend::manager_client::ManagerApiClient;

use crate::{register::RegisterAccount, unlimited_likes::UnlimitedLikesUpdate};

pub struct DataAllUtilsImpl;

impl DataAllUtils for DataAllUtilsImpl {
    fn update_unlimited_likes<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        id: AccountIdInternal,
        unlimited_likes: bool,
    ) -> BoxFuture<'a, server_common::result::Result<(), DataError>> {
        async move {
            write_command_runner
                .write(move |cmds| async move {
                    UnlimitedLikesUpdate::new(cmds.write_handle())
                        .update_unlimited_likes_value(id, unlimited_likes)
                        .await
                })
                .await
        }
        .boxed()
    }

    fn register_impl<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        sign_in_with: SignInWithInfo,
        sign_in_with_email: Option<EmailAddress>,
    ) -> BoxFuture<'a, server_common::result::Result<AccountIdInternal, DataError>> {
        async move {
            let id = write_command_runner
                .write(move |cmds| async move {
                    if BACKEND_DATA_RESET_STATE.is_ongoing() {
                        return Err(DataError::NotAllowed.report());
                    }

                    let id = cmds.account().get_next_unique_account_id().await?;
                    RegisterAccount::new(cmds.write_handle())
                        .register(id, sign_in_with, sign_in_with_email.clone())
                        .await?;

                    if sign_in_with_email.is_some() {
                        cmds.account()
                            .email()
                            .send_email_if_not_already_sent(id, EmailMessages::EmailVerification)
                            .await?;
                    }

                    Ok(id)
                })
                .await?;

            Ok(id)
        }
        .boxed()
    }

    fn handle_websocket_binary_message_from_client<'a>(
        &self,
        read_handle: &'a RouterDatabaseReadHandle,
        write_handle: &'a WriteCommandRunnerHandle,
        manager_api: &'a ManagerApiClient,
        socket: &'a mut WebSocket,
        id: AccountIdInternal,
        client_message: ClientMessageForDataAllCrate<'a>,
    ) -> BoxFuture<'a, server_common::result::Result<(), WebSocketError>> {
        crate::websocket::handle_websocket_binary_message_from_client(
            read_handle,
            write_handle,
            manager_api,
            socket,
            id,
            client_message,
        )
        .boxed()
    }

    fn complete_initial_setup<'a>(
        &self,
        config: &'a Config,
        read_handle: &'a RouterDatabaseReadHandle,
        write_handle: &'a WriteCommandRunnerHandle,
        id: AccountIdInternal,
    ) -> BoxFuture<'a, server_common::result::Result<Account, DataError>> {
        async move {
            crate::initial_setup::complete_initial_setup(config, read_handle, write_handle, id)
                .await
        }
        .boxed()
    }

    fn is_match<'a>(
        &self,
        read_handle: &'a RouterDatabaseReadHandle,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> BoxFuture<'a, server_common::result::Result<bool, DataError>> {
        async move {
            let interaction = read_handle
                .chat()
                .account_interaction(account0, account1)
                .await?;
            if let Some(interaction) = interaction {
                Ok(interaction.is_match() && !interaction.is_blocked())
            } else {
                Ok(false)
            }
        }
        .boxed()
    }

    fn delete_all_accounts<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
    ) -> BoxFuture<'a, server_common::result::Result<(), DataError>> {
        async move {
            write_command_runner
                .write(
                    move |cmds| async move { cmds.account().delete().delete_all_accounts().await },
                )
                .await
        }
        .boxed()
    }

    fn data_export<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        zip_main_directory_name: String,
        cmd: DataExportCmd,
    ) -> BoxFuture<'a, server_common::result::Result<(), DataError>> {
        async move {
            crate::data_export::data_export(write_command_runner, zip_main_directory_name, cmd)
                .await
        }
        .boxed()
    }

    fn edit_verification_values<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        moderator_id: AccountIdInternal,
        profile_owner_id: AccountIdInternal,
        values: EditVerificationValues,
    ) -> BoxFuture<'a, server_common::result::Result<(), DataError>> {
        crate::edit_verification_values::edit_verification_values(
            write_command_runner,
            moderator_id,
            profile_owner_id,
            values,
        )
        .boxed()
    }

    fn process_removed_account_verification_queue_item<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        moderator_id: AccountIdInternal,
        profile_owner_id: AccountIdInternal,
        verification_method: VerificationMethod,
        verification_unix_time: UnixTime,
        verification_error_flags: AccountVerificationErrorFlagsValue,
        edit: Option<EditVerificationValues>,
    ) -> BoxFuture<'a, server_common::result::Result<(), DataError>> {
        async move {
            write_command_runner
                .write(move |cmds| async move {
                    // TODO(quality): Single DB transaction for all value updates

                    let mut merged_flags: AccountVerificationErrorFlags =
                        verification_error_flags.into();

                    if let Some(edit_values) = edit {
                        merged_flags |=
                            crate::edit_verification_values::edit_verification_values_in_write_call(
                                &cmds,
                                moderator_id,
                                profile_owner_id,
                                edit_values,
                            )
                            .await?;
                    }

                    cmds.account()
                        .set_account_verification_data(
                            profile_owner_id,
                            verification_method,
                            verification_unix_time,
                            merged_flags.into(),
                        )
                        .await?;

                    Ok(())
                })
                .await
        }
        .boxed()
    }
}
