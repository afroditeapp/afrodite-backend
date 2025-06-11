use model::{
    AccountIdInternal, NewReceivedLikesCountResult, PendingNotification, PendingNotificationFlags, PendingNotificationWithData
};
use server_data::db_manager::RouterDatabaseReadHandle;
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::read::GetReadChatCommands;
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;

pub async fn get_push_notification_data(
    read_handle: &RouterDatabaseReadHandle,
    id: AccountIdInternal,
    notification_value: PendingNotification,
) -> PendingNotificationWithData {
    let flags = PendingNotificationFlags::from(notification_value);
    let sender_info = if flags.contains(PendingNotificationFlags::NEW_MESSAGE) {
        read_handle
            .chat()
            .all_pending_message_sender_account_ids(id)
            .await
            .ok()
    } else {
        None
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

    let media_content_moderation_completed = if flags.contains(PendingNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED) {
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

    let profile_text_moderation_completed = if flags.contains(PendingNotificationFlags::PROFILE_TEXT_MODERATION_COMPLETED) {
        read_handle
            .profile()
            .notification()
            .profile_text_moderation_completed(id)
            .await
            .ok()
    } else {
        None
    };

    let automatic_profile_search_completed = if flags.contains(PendingNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED) {
        read_handle
            .profile()
            .notification()
            .automatic_profile_search_completed(id)
            .await
            .ok()
    } else {
        None
    };

    PendingNotificationWithData {
        value: notification_value,
        new_message_received_from: sender_info,
        received_likes_changed: received_likes_info,
        media_content_moderation_completed,
        news_changed: unread_news_count,
        profile_text_moderation_completed,
        automatic_profile_search_completed,
        // State for this is added in API route handler
        admin_notification: None,
    }
}
