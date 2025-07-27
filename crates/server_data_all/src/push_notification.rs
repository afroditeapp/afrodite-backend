use model::{
    AccountIdInternal, NewReceivedLikesCountResult, PendingNotificationFlags,
    PendingNotificationToken, PendingNotificationWithData,
};
use server_data::{
    db_manager::RouterDatabaseReadHandle, write::GetWriteCommandsCommon,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;

/// Android push notification related code.
/// iOS push notification related code is at
/// [server::push_notifications::ios::ios_notifications].
pub async fn get_push_notification_data(
    read_handle: &RouterDatabaseReadHandle,
    write_handle: &WriteCommandRunnerHandle,
    token: PendingNotificationToken,
) -> (Option<AccountIdInternal>, PendingNotificationWithData) {
    let result = write_handle
        .write(move |cmds| async move {
            let (id, notification_value) = cmds
                .common()
                .push_notification()
                .get_and_reset_pending_notification_with_notification_token(token)
                .await?;

            let flags = PendingNotificationFlags::from(notification_value);
            let new_message = if flags.contains(PendingNotificationFlags::NEW_MESSAGE) {
                let (notifications, messages_pending_push_notification) = cmds
                    .read()
                    .chat()
                    .notification()
                    .new_message_notification_list(id)
                    .await?;

                cmds.chat()
                    .notification()
                    .mark_receiver_push_notification_sent(messages_pending_push_notification)
                    .await?;

                Some(notifications)
            } else {
                None
            };

            Ok((id, notification_value, flags, new_message))
        })
        .await;

    let (id, notification_value, flags, new_message) = match result {
        Err(_) => return (None, PendingNotificationWithData::default()),
        Ok(v) => v,
    };

    let received_likes_info = if flags.contains(PendingNotificationFlags::RECEIVED_LIKES_CHANGED) {
        read_handle
            .chat()
            .chat_state(id)
            .await
            .ok()
            .map(|chat_state| NewReceivedLikesCountResult {
                v: chat_state.received_likes_sync_version,
                c: chat_state.new_received_likes_count,
            })
    } else {
        None
    };

    let media_content_moderation_completed =
        if flags.contains(PendingNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED) {
            read_handle
                .media()
                .notification()
                .media_content_moderation_completed(id)
                .await
                .ok()
        } else {
            None
        };

    let unread_news_count = if flags.contains(PendingNotificationFlags::NEWS_CHANGED) {
        read_handle
            .account()
            .news()
            .unread_news_count(id)
            .await
            .ok()
    } else {
        None
    };

    let profile_string_moderation_completed =
        if flags.contains(PendingNotificationFlags::PROFILE_STRING_MODERATION_COMPLETED) {
            read_handle
                .profile()
                .notification()
                .profile_string_moderation_completed(id)
                .await
                .ok()
        } else {
            None
        };

    let automatic_profile_search_completed =
        if flags.contains(PendingNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED) {
            read_handle
                .profile()
                .notification()
                .automatic_profile_search_completed(id)
                .await
                .ok()
        } else {
            None
        };

    let notification = PendingNotificationWithData {
        value: notification_value,
        new_message,
        received_likes_changed: received_likes_info,
        media_content_moderation_completed,
        news_changed: unread_news_count,
        profile_string_moderation_completed,
        automatic_profile_search_completed,
        // State for this is added in API route handler
        admin_notification: None,
    };

    (Some(id), notification)
}
