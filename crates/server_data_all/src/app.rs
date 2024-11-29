use std::ops::Deref;

use axum::extract::ws::WebSocket;
use config::Config;
use futures::{future::BoxFuture, FutureExt};
use model::{AccountId, AccountIdInternal, EmailMessages, SyncDataVersionFromClient};
use model_account::{EmailAddress, SignInWithInfo};
use server_common::websocket::WebSocketError;
use server_data::{app::DataAllUtils, db_manager::RouterDatabaseReadHandle, write_commands::WriteCommandRunnerHandle, DataError};
use server_data_account::write::GetWriteCommandsAccount;

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
            write_command_runner.write(move |cmds| async move {
                UnlimitedLikesUpdate::new(cmds.deref())
                    .update_unlimited_likes_value(id, unlimited_likes)
                    .await
            })
            .await
        }.boxed()
    }

    fn register_impl<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> BoxFuture<'a, server_common::result::Result<AccountIdInternal, DataError>> {
        async move {
            // New unique UUID is generated every time so no special handling needed
            // to avoid database collisions.
            let id = AccountId::new_random();

            let id = write_command_runner.write(move |cmds| async move {
                let id = RegisterAccount::new(cmds.deref())
                    .register(id, sign_in_with, email.clone())
                    .await?;

                if email.is_some() {
                    cmds.account().email().send_email_if_not_already_sent(
                        id,
                        EmailMessages::AccountRegistered
                    ).await?;
                }

                Ok(id)
            })
            .await?;

            Ok(id)
        }.boxed()
    }

    fn handle_new_websocket_connection<'a>(
        &self,
        config: &'a Config,
        read_handle: &'a RouterDatabaseReadHandle,
        write_handle: &'a WriteCommandRunnerHandle,
        socket: &'a mut WebSocket,
        id: AccountIdInternal,
        sync_versions: Vec<SyncDataVersionFromClient>,
    ) -> BoxFuture<'a, server_common::result::Result<(), WebSocketError>> {
        async move {
            crate::websocket::reset_pending_notification(
                config,
                write_handle,
                id
            ).await?;
            crate::websocket::sync_data_with_client_if_needed(
                config,
                read_handle,
                write_handle,
                socket,
                id,
                sync_versions,
            )
                .await?;
            crate::websocket::send_new_messages_event_if_needed(
                config,
                read_handle,
                socket,
                id,
            ).await?;
            Ok(())
        }.boxed()
    }
}
