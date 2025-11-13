use config::file_notification_content::{
    NotificationStringGetter, NotificationTitle, NotificationTitleAndBody,
};
use model::{AccountIdInternal, PushNotification, PushNotificationFlags, PushNotificationId};
use server_api::{
    DataError,
    app::{AdminNotificationProvider, GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_data::read::GetReadCommandsCommon;
use server_data_account::read::GetReadCommandsAccount;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;
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

    if flags.contains(PushNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED) {
        checker.handle_media_content_moderation().await?;
    }

    if flags.contains(PushNotificationFlags::NEWS_CHANGED) {
        checker.handle_news().await?;
    }

    if flags.contains(PushNotificationFlags::PROFILE_STRING_MODERATION_COMPLETED) {
        checker.handle_profile_string_moderation().await?;
    }

    if flags.contains(PushNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED) {
        checker.handle_automatic_profile_search_completed().await?;
    }

    if flags.contains(PushNotificationFlags::ADMIN_NOTIFICATION) {
        checker.handle_admin_notification().await?;
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
                .mark_receiver_push_notification_sent(messages)
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

    async fn handle_media_content_moderation(&mut self) -> Result<(), DataError> {
        let v = self
            .state
            .read()
            .media()
            .notification()
            .media_content_moderation_completed(self.id)
            .await?;

        if !v.accepted.notification_viewed() {
            self.add_notification(
                PushNotificationId::MediaContentModerationAccepted,
                self.notification_strings.media_content_accepted(),
            );
        }

        if !v.rejected.notification_viewed() {
            self.add_notification(
                PushNotificationId::MediaContentModerationRejected,
                self.notification_strings.media_content_rejected(),
            );
        }

        if !v.deleted.notification_viewed() {
            self.add_notification_with_body(
                PushNotificationId::MediaContentModerationDeleted,
                self.notification_strings.media_content_deleted(),
            );
        }

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

    async fn handle_profile_string_moderation(&mut self) -> Result<(), DataError> {
        let v = self
            .state
            .read()
            .profile()
            .notification()
            .profile_string_moderation_completed(self.id)
            .await?;

        if !v.name_accepted.notification_viewed() {
            self.add_notification(
                PushNotificationId::ProfileNameModerationAccepted,
                self.notification_strings.profile_name_accepted(),
            );
        }

        if !v.name_rejected.notification_viewed() {
            self.add_notification(
                PushNotificationId::ProfileNameModerationRejected,
                self.notification_strings.profile_name_rejected(),
            );
        }

        if !v.text_accepted.notification_viewed() {
            self.add_notification(
                PushNotificationId::ProfileTextModerationAccepted,
                self.notification_strings.profile_text_accepted(),
            );
        }

        if !v.text_rejected.notification_viewed() {
            self.add_notification(
                PushNotificationId::ProfileTextModerationRejected,
                self.notification_strings.profile_text_rejected(),
            );
        }

        Ok(())
    }

    async fn handle_automatic_profile_search_completed(&mut self) -> Result<(), DataError> {
        let search = self
            .state
            .read()
            .profile()
            .notification()
            .automatic_profile_search_completed(self.id)
            .await?;

        if !search.notifications_viewed() {
            self.add_notification(
                PushNotificationId::AutomaticProfileSearchCompleted,
                if search.profile_count == 1 {
                    self.notification_strings
                        .automatic_profile_search_found_profiles_single()
                } else {
                    self.notification_strings
                        .automatic_profile_search_found_profiles_multiple(
                            &search.profile_count.to_string(),
                        )
                },
            );
        }

        Ok(())
    }

    async fn handle_admin_notification(&mut self) -> Result<(), DataError> {
        let admin = self
            .state
            .admin_notification()
            .get_unreceived_notification(self.id)
            .await;

        if let Some(admin) = admin {
            let notification = PushNotification::new_with_body(
                PushNotificationId::AdminNotification,
                "Admin notification".to_string(),
                admin.field_names_of_true_values(),
            );
            self.notifications.push(notification);
        }

        Ok(())
    }
}
