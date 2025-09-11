use model::{AccountIdInternal, NewReceivedLikesCountResult, PendingNotificationFlags};
use server_api::{
    DataError,
    app::{AdminNotificationProvider, ReadData},
};
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::read::GetReadChatCommands;
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;
use server_state::S;

pub async fn is_notification_visible(
    state: &S,
    id: AccountIdInternal,
    flags: PendingNotificationFlags,
) -> Result<bool, DataError> {
    let mut checker = VisibilityChecker { is_visible: false };

    checker
        .check(flags, PendingNotificationFlags::NEW_MESSAGE, async || {
            let (notifications, _) = state
                .read()
                .chat()
                .notification()
                .new_message_notification_list(id)
                .await
                .ok()?;

            Some(!notifications.v.is_empty())
        })
        .await;

    checker
        .check(
            flags,
            PendingNotificationFlags::RECEIVED_LIKES_CHANGED,
            async || {
                let v = state
                    .read()
                    .chat()
                    .chat_state(id)
                    .await
                    .ok()
                    .map(|chat_state| NewReceivedLikesCountResult {
                        v: chat_state.received_likes_sync_version,
                        c: chat_state.new_received_likes_count,
                    })?;

                Some(v.c.c > 0)
            },
        )
        .await;

    checker
        .check(
            flags,
            PendingNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED,
            async || {
                let v = state
                    .read()
                    .media()
                    .notification()
                    .media_content_moderation_completed(id)
                    .await
                    .ok()?;

                Some(!v.accepted.notification_viewed() || !v.rejected.notification_viewed())
            },
        )
        .await;

    checker
        .check(flags, PendingNotificationFlags::NEWS_CHANGED, async || {
            state
                .read()
                .account()
                .news()
                .unread_news_count(id)
                .await
                .ok()
                .map(|_| true)
        })
        .await;

    checker
        .check(
            flags,
            PendingNotificationFlags::PROFILE_STRING_MODERATION_COMPLETED,
            async || {
                let v = state
                    .read()
                    .profile()
                    .notification()
                    .profile_string_moderation_completed(id)
                    .await
                    .ok()?;

                Some(
                    !v.name_accepted.notification_viewed()
                        || !v.name_rejected.notification_viewed()
                        || !v.text_accepted.notification_viewed()
                        || !v.text_rejected.notification_viewed(),
                )
            },
        )
        .await;

    checker
        .check(
            flags,
            PendingNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED,
            async || {
                state
                    .read()
                    .profile()
                    .notification()
                    .automatic_profile_search_completed(id)
                    .await
                    .ok()
                    .map(|_| true)
            },
        )
        .await;

    checker
        .check(
            flags,
            PendingNotificationFlags::ADMIN_NOTIFICATION,
            async || {
                state
                    .admin_notification()
                    .get_unreceived_notification(id)
                    .await
                    .map(|_| true)
            },
        )
        .await;

    Ok(checker.is_visible())
}

struct VisibilityChecker {
    is_visible: bool,
}

impl VisibilityChecker {
    async fn check(
        &mut self,
        flags: PendingNotificationFlags,
        wanted: PendingNotificationFlags,
        action: impl AsyncFnOnce() -> Option<bool>,
    ) {
        if flags.contains(wanted) && action().await.unwrap_or_default() {
            self.is_visible = true;
        }
    }

    fn is_visible(&self) -> bool {
        self.is_visible
    }
}
