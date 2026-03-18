use config::file_notification_content::{
    NotificationStringGetter, NotificationTitle, NotificationTitleAndBody,
};
use model::{
    AccountIdInternal, PendingAppNotificationType, PushNotification, PushNotificationFlags,
    PushNotificationId,
};
use server_api::{
    DataError,
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_state::{S, result::Result};

pub async fn notifications_for_sending(
    state: &S,
    id: AccountIdInternal,
    flags: PushNotificationFlags,
) -> Result<Vec<PushNotification>, DataError> {
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
    };

    if flags.contains(PushNotificationFlags::NEW_MESSAGE) {
        checker.handle_new_message().await?;
    }

    if flags.contains(PushNotificationFlags::RECEIVED_LIKES_CHANGED) {
        checker.handle_received_likes().await?;
    }

    if flags.intersects(
        PushNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED
            | PushNotificationFlags::PROFILE_STRING_MODERATION_COMPLETED
            | PushNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED
            | PushNotificationFlags::ADMIN_NOTIFICATION,
    ) {
        checker.handle_pending_notifications().await?;
    }

    if flags.contains(PushNotificationFlags::NEWS_CHANGED) {
        checker.handle_news().await?;
    }

    Ok(checker.notifications)
}

struct NotificationChecker<'a> {
    state: &'a S,
    id: AccountIdInternal,
    notification_strings: NotificationStringGetter<'a>,
    notifications: Vec<PushNotification>,
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
        let (notifications, messages) = self
            .state
            .read()
            .chat()
            .notification()
            .new_message_notification_list(self.id)
            .await?;

        for n in notifications.v {
            let name = self
                .state
                .read()
                .common()
                .user_visible_profile_name_if_data_available(n.a)
                .await?
                .map(|v| v.into_string())
                .unwrap_or_default();
            let title = if n.m == 1 {
                self.notification_strings.message_received_single(&name)
            } else {
                self.notification_strings.message_received_multiple(&name)
            };
            let notification = PushNotification::new_message(n.c, title.title);
            self.notifications.push(notification);
        }

        db_write_raw!(self.state, move |cmds| {
            cmds.chat()
                .notification()
                .mark_recipient_push_notification_sent(messages)
                .await
        })
        .await?;

        Ok(())
    }

    async fn handle_received_likes(&mut self) -> Result<(), DataError> {
        let v = self
            .state
            .read()
            .chat()
            .chat_state(self.id)
            .await?
            .new_received_likes_info();

        self.add_notification(
            PushNotificationId::LikeReceived,
            if v.c.c == 1 {
                self.notification_strings.like_received_single()
            } else {
                self.notification_strings.like_received_multiple()
            },
        );

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

        let account_id = self.id;
        db_write_raw!(self.state, move |cmds| {
            cmds.common()
                .notification()
                .mark_pending_app_notifications_push_sent(account_id, pending_notifications)
                .await
        })
        .await?;

        Ok(())
    }

    async fn handle_news(&mut self) -> Result<(), DataError> {
        let count = self
            .state
            .read()
            .account()
            .news()
            .unread_news_count(self.id)
            .await?;

        if count.c.c == 0 {
            let notification =
                PushNotification::remove_notification(PushNotificationId::NewsItemAvailable);
            self.notifications.push(notification);
        } else {
            self.add_notification(
                PushNotificationId::NewsItemAvailable,
                self.notification_strings.news_item_available(),
            );
        }

        Ok(())
    }
}
