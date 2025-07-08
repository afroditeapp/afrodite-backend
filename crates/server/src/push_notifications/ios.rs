use fcm::message::{ApnsConfig, Message, Target};
use model::{
    AccountIdInternal, FcmDeviceToken, NewMessageNotificationList, NewReceivedLikesCountResult,
    PendingNotificationFlags, PendingNotificationWithData,
};
use serde_json::json;
use server_api::{
    DataError,
    app::{AdminNotificationProvider, ReadData},
};
use server_common::push_notifications::{PushNotification, SuccessfulSendingAction};
use server_data::read::GetReadCommandsCommon;
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::read::GetReadChatCommands;
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;
use server_state::S;

const FIRST_CONVERSATION_NOTIFICATION_ID: i64 = 1000;

/// iOS push notification related code.
/// Android push notification related code is at
/// [server_data_all::push_notification::get_push_notification_data].
pub async fn ios_notifications(
    state: &S,
    id: AccountIdInternal,
    token: FcmDeviceToken,
    flags: PendingNotificationFlags,
) -> Result<Vec<PushNotification>, DataError> {
    let mut builder = MessageBuilder {
        messages: vec![],
        token,
    };

    builder
        .check(flags, PendingNotificationFlags::NEW_MESSAGE, async || {
            let (notifications, messages_pending_push_notification) = state
                .read()
                .chat()
                .new_message_notification_list(id)
                .await
                .ok()?;

            let mut list = vec![];

            for (n, message) in notifications
                .v
                .into_iter()
                .zip(messages_pending_push_notification)
            {
                let data = PendingNotificationWithData {
                    new_message: Some(NewMessageNotificationList { v: vec![n.clone()] }),
                    ..Default::default()
                };

                let name = state
                    .read()
                    .common()
                    .get_profile_age_and_name_if_profile_component_is_enabled(n.a)
                    .await
                    .ok()
                    .flatten()
                    .map(|v| v.name)
                    .unwrap_or_default();

                list.push(NotificationInfo::title_and_successful_sending_action(
                    FIRST_CONVERSATION_NOTIFICATION_ID.saturating_add(n.c.id),
                    data,
                    if n.m > 1 {
                        format!("{name} sent messages")
                    } else {
                        format!("{name} sent a message")
                    },
                    SuccessfulSendingAction::MarkMessageNotificationSent { message },
                ));
            }

            Some(list)
        })
        .await;

    builder
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

                let data = PendingNotificationWithData {
                    received_likes_changed: Some(v.clone()),
                    ..Default::default()
                };

                if v.c.c > 0 {
                    Some(vec![NotificationInfo::title(
                        0,
                        data,
                        "Like received".to_string(),
                    )])
                } else {
                    None
                }
            },
        )
        .await;

    builder
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

                let data = PendingNotificationWithData {
                    media_content_moderation_completed: Some(v),
                    ..Default::default()
                };

                let mut notifications = vec![];

                if v.accepted != v.accepted_viewed {
                    notifications.push(NotificationInfo::title(
                        1,
                        data.clone(),
                        "Profile image accepted".to_string(),
                    ));
                }

                if v.rejected != v.rejected_viewed {
                    notifications.push(NotificationInfo::title(
                        2,
                        data,
                        "Profile image rejected".to_string(),
                    ));
                }

                Some(notifications)
            },
        )
        .await;

    builder
        .check(flags, PendingNotificationFlags::NEWS_CHANGED, async || {
            state
                .read()
                .account()
                .news()
                .unread_news_count(id)
                .await
                .ok()
                .map(|v| {
                    NotificationInfo::title(
                        3,
                        PendingNotificationWithData {
                            news_changed: Some(v),
                            ..Default::default()
                        },
                        "News available".to_string(),
                    )
                })
                .map(|v| vec![v])
        })
        .await;

    builder
        .check(
            flags,
            PendingNotificationFlags::PROFILE_TEXT_MODERATION_COMPLETED,
            async || {
                let v = state
                    .read()
                    .profile()
                    .notification()
                    .profile_text_moderation_completed(id)
                    .await
                    .ok()?;

                let data = PendingNotificationWithData {
                    profile_text_moderation_completed: Some(v),
                    ..Default::default()
                };

                let mut notifications = vec![];

                if v.accepted != v.accepted_viewed {
                    notifications.push(NotificationInfo::title(
                        4,
                        data.clone(),
                        "Profile text accepted".to_string(),
                    ));
                }

                if v.rejected != v.rejected_viewed {
                    notifications.push(NotificationInfo::title(
                        5,
                        data,
                        "Profile text rejected".to_string(),
                    ));
                }

                Some(notifications)
            },
        )
        .await;

    builder
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
                    .map(|v| {
                        NotificationInfo::title(
                            6,
                            PendingNotificationWithData {
                                automatic_profile_search_completed: Some(v),
                                ..Default::default()
                            },
                            "Automatic profile search completed".to_string(),
                        )
                    })
                    .map(|v| vec![v])
            },
        )
        .await;

    builder
        .check(
            flags,
            PendingNotificationFlags::ADMIN_NOTIFICATION,
            async || {
                state
                    .admin_notification()
                    .get_notification_state(id)
                    .await
                    .map(|v| {
                        let body = serde_json::to_string_pretty(&v).unwrap_or_default();
                        NotificationInfo::title_and_body(
                            7,
                            PendingNotificationWithData {
                                admin_notification: Some(v),
                                ..Default::default()
                            },
                            "Admin notification".to_string(),
                            body,
                        )
                    })
                    .map(|v| vec![v])
            },
        )
        .await;

    Ok(builder.build())
}

struct NotificationInfo {
    data: PendingNotificationWithData,
    title: String,
    body: String,
    collapse_id: i64,
    successful_sending_action: Option<SuccessfulSendingAction>,
}

impl NotificationInfo {
    fn title(collapse_id: i64, data: PendingNotificationWithData, title: String) -> Self {
        Self {
            data,
            title,
            collapse_id,
            body: String::new(),
            successful_sending_action: None,
        }
    }

    fn title_and_body(
        collapse_id: i64,
        data: PendingNotificationWithData,
        title: String,
        body: String,
    ) -> Self {
        Self {
            data,
            title,
            collapse_id,
            body,
            successful_sending_action: None,
        }
    }

    fn title_and_successful_sending_action(
        collapse_id: i64,
        data: PendingNotificationWithData,
        title: String,
        successful_sending_action: SuccessfulSendingAction,
    ) -> Self {
        Self {
            data,
            title,
            collapse_id,
            body: String::new(),
            successful_sending_action: Some(successful_sending_action),
        }
    }
}

struct MessageBuilder {
    messages: Vec<PushNotification>,
    token: FcmDeviceToken,
}

impl MessageBuilder {
    async fn check(
        &mut self,
        flags: PendingNotificationFlags,
        wanted: PendingNotificationFlags,
        action: impl AsyncFnOnce() -> Option<Vec<NotificationInfo>>,
    ) {
        if flags.contains(wanted) {
            let Some(list) = action().await else {
                return;
            };

            if list.is_empty() {
                // Clear pending notification flag from the cache
                self.messages.push(PushNotification {
                    message: None,
                    flags,
                    successful_sending_action: None,
                });
            }

            for info in list {
                self.messages
                    .push(create_message(self.token.clone(), info, wanted));
            }
        }
    }

    fn build(self) -> Vec<PushNotification> {
        self.messages
    }
}

fn create_message(
    token: FcmDeviceToken,
    info: NotificationInfo,
    flags: PendingNotificationFlags,
) -> PushNotification {
    let message = Message {
        target: Target::Token(token.into_string()),
        apns: Some(ApnsConfig {
            headers: Some(json!({
                "apns-collapse-id": info.collapse_id.to_string(),
            })),
            payload: Some(json!({
                "aps": {
                    "mutable-content": 1,
                    "alert": {
                        "title": info.title,
                        "body": info.body,
                    }
                },
                "data": info.data,
            })),
            ..Default::default()
        }),
        notification: None,
        android: None,
        webpush: None,
        fcm_options: None,
        data: None,
    };
    PushNotification {
        message: Some(message),
        flags,
        successful_sending_action: info.successful_sending_action,
    }
}
