use axum::extract::ws::WebSocket;
use config::Config;
use futures::{future::BoxFuture, FutureExt};
use model::{
    Account, AccountIdInternal, EmailMessages, PendingNotification,
    PendingNotificationWithData, SyncDataVersionFromClient,
};
use model_account::{EmailAddress, SignInWithInfo};
use server_common::websocket::WebSocketError;
use server_data::{
    app::DataAllUtils, db_manager::RouterDatabaseReadHandle,
    write_commands::WriteCommandRunnerHandle, DataError,
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
        email: Option<EmailAddress>,
    ) -> BoxFuture<'a, server_common::result::Result<AccountIdInternal, DataError>> {
        async move {
            let id = write_command_runner
                .write(move |cmds| async move {
                    let id = cmds.account().get_next_unique_account_id().await?;
                    let id = RegisterAccount::new(cmds.write_handle())
                        .register(id, sign_in_with, email.clone())
                        .await?;

                    if email.is_some() {
                        cmds.account()
                            .email()
                            .send_email_if_not_already_sent(id, EmailMessages::AccountRegistered)
                            .await?;
                    }

                    Ok(id)
                })
                .await?;

            Ok(id)
        }
        .boxed()
    }

    fn handle_new_websocket_connection<'a>(
        &self,
        config: &'a Config,
        read_handle: &'a RouterDatabaseReadHandle,
        write_handle: &'a WriteCommandRunnerHandle,
        manager_api: &'a ManagerApiClient,
        socket: &'a mut WebSocket,
        id: AccountIdInternal,
        sync_versions: Vec<SyncDataVersionFromClient>,
    ) -> BoxFuture<'a, server_common::result::Result<(), WebSocketError>> {
        async move {
            // TODO(prod): Could data writing be avoided on every websocket
            // connection or could it happen once? The server_api crate
            // also does writing using write handle.
            crate::websocket::reset_pending_notification(config, write_handle, id).await?;
            crate::websocket::sync_data_with_client_if_needed(
                config,
                read_handle,
                write_handle,
                manager_api,
                socket,
                id,
                sync_versions,
            )
            .await?;
            crate::websocket::send_new_messages_and_server_maintenance_events_if_needed(
                config,
                read_handle,
                manager_api,
                socket,
                id
            )
                .await?;
            Ok(())
        }
        .boxed()
    }

    fn get_push_notification_data<'a>(
        &self,
        read_handle: &'a RouterDatabaseReadHandle,
        id: AccountIdInternal,
        notification_value: PendingNotification,
    ) -> BoxFuture<'a, PendingNotificationWithData> {
        async move {
            crate::push_notification::get_push_notification_data(
                read_handle,
                id,
                notification_value,
            )
            .await
        }
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
}
