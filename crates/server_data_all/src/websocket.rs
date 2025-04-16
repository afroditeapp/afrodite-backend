use axum::extract::ws::{Message, WebSocket};
use config::Config;
use model::ScheduledMaintenanceStatus;
use model_chat::{
    AccountIdInternal, ChatStateRaw, EventToClient, EventToClientInternal,
    SyncCheckDataType, SyncCheckResult, SyncDataVersionFromClient, SyncVersionFromClient,
    SyncVersionUtils,
};
use server_common::websocket::WebSocketError;
use server_data::{
    db_manager::RouterDatabaseReadHandle, read::GetReadCommandsCommon, result::{Result, WrappedResultExt}, write::GetWriteCommandsCommon, write_commands::WriteCommandRunnerHandle
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::manager_client::ManagerApiClient;

pub async fn reset_pending_notification(
    config: &Config,
    write_handle: &WriteCommandRunnerHandle,
    id: AccountIdInternal,
) -> Result<(), WebSocketError> {
    if config.components().chat {
        write_handle
            .write(move |cmds| async move {
                cmds.common()
                    .push_notification()
                    .reset_pending_notification(id)
                    .await
            })
            .await
            .change_context(WebSocketError::DatabasePendingNotificationReset)?;
    }

    Ok(())
}

pub async fn send_events_if_needed(
    config: &Config,
    read_handle: &RouterDatabaseReadHandle,
    manager_api_client: &ManagerApiClient,
    socket: &mut WebSocket,
    id: AccountIdInternal,
) -> Result<(), WebSocketError> {
    if config.components().profile {
        let notification = read_handle
            .profile()
            .notification()
            .profile_text_moderation_completed(id)
            .await
            .change_context(WebSocketError::DatabaseProfileTextModerationCompletedNotificationQuery)?;

        if !notification.notifications_viewed() {
            send_event(socket, EventToClientInternal::ProfileTextModerationCompleted).await?;
        }
    }

    if config.components().media {
        let notification = read_handle
            .media()
            .notification()
            .media_content_moderation_completed(id)
            .await
            .change_context(WebSocketError::DatabaseMediaContentModerationCompletedNotificationQuery)?;

        if !notification.notifications_viewed() {
            send_event(socket, EventToClientInternal::MediaContentModerationCompleted).await?;
        }
    }

    if config.components().chat {
        let pending_messages = read_handle
            .chat()
            .all_pending_messages(id)
            .await
            .change_context(WebSocketError::DatabasePendingMessagesQuery)?;

        if !pending_messages.is_empty() {
            send_event(socket, EventToClientInternal::NewMessageReceived).await?;
        }
    }

    if let Some(time) = manager_api_client.latest_scheduled_reboot() {
        send_event(
            socket,
            EventToClientInternal::ScheduledMaintenanceStatus(
                ScheduledMaintenanceStatus {
                    scheduled_maintenance: Some(time),
                }
            ),
        ).await?;
    }

    Ok(())
}

pub async fn sync_data_with_client_if_needed(
    config: &Config,
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    manager_api_client: &ManagerApiClient,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_versions: Vec<SyncDataVersionFromClient>,
) -> Result<(), WebSocketError> {
    let chat_state = read_handle
        .chat()
        .chat_state(id)
        .await
        .change_context(WebSocketError::DatabaseChatStateQuery)?;

    for version in sync_versions {
        match version.data_type {
            SyncCheckDataType::Account => {
                if config.components().account {
                    handle_account_data_sync(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ReveivedBlocks => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.received_blocks_sync_version,
                        EventToClientInternal::ReceivedBlocksChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ReveivedLikes => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.received_likes_sync_version,
                        EventToClientInternal::ReceivedLikesChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::SentBlocks => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.sent_blocks_sync_version,
                        EventToClientInternal::SentBlocksChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::SentLikes => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.sent_likes_sync_version,
                        EventToClientInternal::SentLikesChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::Matches => {
                if config.components().chat {
                    handle_chat_state_version_check(
                        write_handle,
                        socket,
                        id,
                        version.version,
                        chat_state.clone(),
                        |s| &mut s.matches_sync_version,
                        EventToClientInternal::MatchesChanged,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ClientConfig => {
                handle_client_config_sync_version_check(
                    read_handle,
                    write_handle,
                    socket,
                    id,
                    version.version,
                )
                .await?;
            }
            SyncCheckDataType::Profile => {
                if config.components().profile {
                    handle_profile_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::News => {
                if config.components().account {
                    handle_news_count_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::MediaContent => {
                if config.components().account {
                    handle_media_content_sync_version_check(
                        read_handle,
                        write_handle,
                        socket,
                        id,
                        version.version,
                    )
                    .await?;
                }
            }
            SyncCheckDataType::ServerMaintenanceIsScheduled => {
                handle_maintenance_info_removing_if_needed(
                    manager_api_client,
                    socket,
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn handle_account_data_sync(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let account = read_handle
        .common()
        .account(id)
        .await
        .change_context(WebSocketError::DatabaseAccountStateQuery)?;

    match account.sync_version().check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => {
            write_handle
                .write(move |cmds| async move {
                    cmds.account().reset_syncable_account_data_version(id).await
                })
                .await
                .change_context(WebSocketError::AccountDataVersionResetFailed)?;
        }
        SyncCheckResult::Sync => (),
    };

    send_event(
        socket,
        EventToClientInternal::AccountStateChanged,
    )
    .await?;

    Ok(())
}

async fn handle_chat_state_version_check<T: SyncVersionUtils>(
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
    mut chat_state: ChatStateRaw,
    getter: impl Fn(&mut ChatStateRaw) -> &mut T + Send + 'static,
    event: EventToClientInternal,
) -> Result<(), WebSocketError> {
    let check_this_version = getter(&mut chat_state);
    match check_this_version.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.chat()
                    .modify_chat_state(id, move |s| {
                        let version_to_be_reseted = getter(s);
                        *version_to_be_reseted = Default::default();
                    })
                    .await
            })
            .await
            .change_context(WebSocketError::ChatDataVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, event).await?;

    Ok(())
}

async fn handle_client_config_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .common()
        .client_config_sync_version(id)
        .await
        .change_context(WebSocketError::DatabaseProfileStateQuery)?;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.common()
                    .reset_client_config_sync_version(id)
                    .await
            })
            .await
            .change_context(WebSocketError::ProfileAttributesSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(
        socket,
        EventToClientInternal::ClientConfigChanged,
    )
    .await?;

    Ok(())
}

async fn handle_profile_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .profile()
        .profile_state(id)
        .await
        .change_context(WebSocketError::DatabaseProfileStateQuery)?
        .profile_sync_version;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move { cmds.profile().reset_profile_sync_version(id).await })
            .await
            .change_context(WebSocketError::ProfileSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::ProfileChanged).await?;

    Ok(())
}

async fn handle_news_count_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .account()
        .news()
        .unread_news_count(id)
        .await
        .change_context(WebSocketError::DatabaseNewsCountQuery)?
        .v;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.account()
                    .news()
                    .reset_news_count_sync_version(id)
                    .await
            })
            .await
            .change_context(WebSocketError::NewsCountSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::NewsChanged).await?;

    Ok(())
}


async fn handle_media_content_sync_version_check(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: SyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current = read_handle
        .media()
        .media_content_sync_version(id)
        .await
        .change_context(WebSocketError::DatabaseMediaContentSyncVersionQuery)?;
    match current.check_is_sync_required(sync_version) {
        SyncCheckResult::DoNothing => return Ok(()),
        SyncCheckResult::ResetVersionAndSync => write_handle
            .write(move |cmds| async move {
                cmds.media()
                    .reset_media_content_sync_version(id)
                    .await
            })
            .await
            .change_context(WebSocketError::MediaContentSyncVersionResetFailed)?,
        SyncCheckResult::Sync => (),
    };

    send_event(socket, EventToClientInternal::MediaContentChanged).await?;

    Ok(())
}

async fn handle_maintenance_info_removing_if_needed(
    manager_api_client: &ManagerApiClient,
    socket: &mut WebSocket,
) -> Result<(), WebSocketError> {
    if manager_api_client.latest_scheduled_reboot().is_none() {
        send_event(
            socket,
            EventToClientInternal::ScheduledMaintenanceStatus(
                ScheduledMaintenanceStatus::default()
            ),
        ).await?;
    }

    Ok(())
}

async fn send_event(
    socket: &mut WebSocket,
    event: impl Into<EventToClient>,
) -> Result<(), WebSocketError> {
    let event: EventToClient = event.into();
    let event = serde_json::to_string(&event).change_context(WebSocketError::Serialize)?;
    socket
        .send(Message::Text(event.into()))
        .await
        .change_context(WebSocketError::Send)?;

    Ok(())
}
