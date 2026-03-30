use config::file_notification_content::{
    NotificationStringGetter, NotificationTitle, NotificationTitleAndBody,
};
use model::{
    AccountIdInternal, NewMessagePushNotification, PendingAppNotification,
    PendingAppNotificationType, PushNotification, PushNotificationFlags, PushNotificationId,
};
use server_api::{
    DataError,
    app::{GetConfig, ReadData},
};
use server_data::read::GetReadCommandsCommon;
use server_data_chat::read::GetReadChatCommands;
use server_state::{S, result::Result};

pub struct NotificationsForSending {
    pub notifications: Vec<PushNotification>,
    pub pending_app_notifications_to_mark_as_sent: Vec<PendingAppNotification>,
    pub new_message_notifications_to_mark_as_sent: Vec<NewMessagePushNotification>,
}

pub async fn notifications_for_sending(
    state: &S,
    id: AccountIdInternal,
    flags: PushNotificationFlags,
) -> Result<NotificationsForSending, DataError> {
    let client_language = state
        .read()
        .common()
        .client_config()
        .client_language(id)
        .await?;

    let mut checker = NotificationChecker {
        id,
        state,
        notification_strings: state
            .config()
            .notification_content()
            .get(client_language.as_ref()),
        notifications: vec![],
        pending_app_notifications_to_mark_as_sent: vec![],
        new_message_notifications_to_mark_as_sent: vec![],
    };

    if flags.contains(PushNotificationFlags::PENDING_CHAT_NOTIFICATION) {
        checker.handle_new_message().await?;
    }

    if flags.contains(PushNotificationFlags::PENDING_APP_NOTIFICATION) {
        checker.handle_pending_notifications().await?;
    }

    Ok(NotificationsForSending {
        notifications: checker.notifications,
        pending_app_notifications_to_mark_as_sent: checker
            .pending_app_notifications_to_mark_as_sent,
        new_message_notifications_to_mark_as_sent: checker
            .new_message_notifications_to_mark_as_sent,
    })
}

struct NotificationChecker<'a> {
    state: &'a S,
    id: AccountIdInternal,
    notification_strings: NotificationStringGetter<'a>,
    notifications: Vec<PushNotification>,
    pending_app_notifications_to_mark_as_sent: Vec<PendingAppNotification>,
    new_message_notifications_to_mark_as_sent: Vec<NewMessagePushNotification>,
}

impl<'a> NotificationChecker<'a> {
    fn add_notification(&mut self, notification: PushNotificationId, title: NotificationTitle) {
        let notification = PushNotification::new(notification, title.title);
        self.notifications.push(notification);
    }

    fn add_notification_with_body(
        &mut self,
        notification: PushNotificationId,
        content: NotificationTitleAndBody,
    ) {
        let notification =
            PushNotification::new_with_body(notification, content.title, content.body);
        self.notifications.push(notification);
    }

    async fn handle_new_message(&mut self) -> Result<(), DataError> {
        let notifications = self
            .state
            .read()
            .chat()
            .notification()
            .new_message_notification_list(self.id)
            .await?;

        let notifications = notifications.values;

        if notifications.is_empty() {
            return Ok(());
        }

        for n in &notifications {
            let name = self
                .state
                .read()
                .common()
                .user_visible_profile_name_if_data_available(n.message_sender.as_id())
                .await?
                .map(|v| v.into_string())
                .unwrap_or_default();
            let title = if n.message_count == 1 {
                self.notification_strings.message_received_single(&name)
            } else {
                self.notification_strings.message_received_multiple(&name)
            };
            let notification = PushNotification::new_message(n.conversation_id, title.title);
            self.notifications.push(notification);
        }

        self.new_message_notifications_to_mark_as_sent = notifications;

        Ok(())
    }

    async fn handle_pending_notifications(&mut self) -> Result<(), DataError> {
        let pending_notifications = self
            .state
            .read()
            .common()
            .notification()
            .pending_app_notifications_without_sent_push(self.id)
            .await?;

        for notification in &pending_notifications {
            match notification.notification_type {
                PendingAppNotificationType::ReceivedLikesChanged => {
                    let received_likes_count = notification.data_integer.unwrap_or_default();
                    // Notification is sent only when like is added so
                    // received_likes_count == 0 doesn't happen.
                    self.add_notification(
                        PushNotificationId::LikeReceived,
                        if received_likes_count == 1 {
                            self.notification_strings.like_received_single()
                        } else {
                            self.notification_strings.like_received_multiple()
                        },
                    )
                }
                PendingAppNotificationType::NewsChanged => {
                    let unread_news_count = notification.data_integer.unwrap_or_default();
                    if unread_news_count == 0 {
                        self.notifications
                            .push(PushNotification::remove_notification(
                                PushNotificationId::NewsItemAvailable,
                            ));
                    } else {
                        self.add_notification(
                            PushNotificationId::NewsItemAvailable,
                            self.notification_strings.news_item_available(),
                        );
                    }
                }
                PendingAppNotificationType::MediaContentModerationAccepted => {
                    self.add_notification(
                        PushNotificationId::MediaContentModerationAccepted,
                        self.notification_strings.media_content_accepted(),
                    );
                }
                PendingAppNotificationType::MediaContentModerationRejected => {
                    self.add_notification(
                        PushNotificationId::MediaContentModerationRejected,
                        self.notification_strings.media_content_rejected(),
                    );
                }
                PendingAppNotificationType::MediaContentModerationDeleted => {
                    self.add_notification_with_body(
                        PushNotificationId::MediaContentModerationDeleted,
                        self.notification_strings.media_content_deleted(),
                    );
                }
                PendingAppNotificationType::ProfileNameModerationAccepted => {
                    self.add_notification(
                        PushNotificationId::ProfileNameModerationAccepted,
                        self.notification_strings.profile_name_accepted(),
                    );
                }
                PendingAppNotificationType::ProfileNameModerationRejected => {
                    self.add_notification(
                        PushNotificationId::ProfileNameModerationRejected,
                        self.notification_strings.profile_name_rejected(),
                    );
                }
                PendingAppNotificationType::ProfileTextModerationAccepted => {
                    self.add_notification(
                        PushNotificationId::ProfileTextModerationAccepted,
                        self.notification_strings.profile_text_accepted(),
                    );
                }
                PendingAppNotificationType::ProfileTextModerationRejected => {
                    self.add_notification(
                        PushNotificationId::ProfileTextModerationRejected,
                        self.notification_strings.profile_text_rejected(),
                    );
                }
                PendingAppNotificationType::AutomaticProfileSearchCompleted => {
                    let data_integer = notification.data_integer.unwrap_or_default();
                    self.add_notification(
                        PushNotificationId::AutomaticProfileSearchCompleted,
                        if data_integer == 1 {
                            self.notification_strings
                                .automatic_profile_search_found_profiles_single()
                        } else {
                            self.notification_strings
                                .automatic_profile_search_found_profiles_multiple(
                                    &data_integer.to_string(),
                                )
                        },
                    );
                }
                PendingAppNotificationType::AdminNotification => {
                    self.notifications.push(PushNotification::new(
                        PushNotificationId::AdminNotification,
                        "Admin notification".to_string(),
                    ));
                }
            }
        }

        self.pending_app_notifications_to_mark_as_sent = pending_notifications;

        Ok(())
    }
}
