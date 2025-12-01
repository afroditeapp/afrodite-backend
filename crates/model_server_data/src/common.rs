use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
};
use model::NotificationEvent;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::account_app_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountAppNotificationSettings {
    pub news: bool,
}

impl Default for AccountAppNotificationSettings {
    fn default() -> Self {
        Self { news: true }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::profile_app_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileAppNotificationSettings {
    pub profile_string_moderation: bool,
    pub automatic_profile_search: bool,
}

impl Default for ProfileAppNotificationSettings {
    fn default() -> Self {
        Self {
            profile_string_moderation: true,
            automatic_profile_search: true,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::media_app_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaAppNotificationSettings {
    pub media_content_moderation: bool,
}

impl Default for MediaAppNotificationSettings {
    fn default() -> Self {
        Self {
            media_content_moderation: true,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::chat_app_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct ChatAppNotificationSettings {
    pub likes: bool,
    pub messages: bool,
}

impl Default for ChatAppNotificationSettings {
    fn default() -> Self {
        Self {
            likes: true,
            messages: true,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::chat_email_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct ChatEmailNotificationSettings {
    pub likes: bool,
    pub messages: bool,
}

impl Default for ChatEmailNotificationSettings {
    fn default() -> Self {
        Self {
            likes: true,
            messages: true,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    Deserialize,
    Serialize,
    ToSchema,
)]
#[diesel(table_name = crate::schema::profile_privacy_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfilePrivacySettings {
    pub online_status: bool,
    pub last_seen_time: bool,
}

impl Default for ProfilePrivacySettings {
    fn default() -> Self {
        Self {
            online_status: true,
            last_seen_time: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct AppNotificationSettingsInternal {
    pub account: AccountAppNotificationSettings,
    pub profile: ProfileAppNotificationSettings,
    pub media: MediaAppNotificationSettings,
    pub chat: ChatAppNotificationSettings,
}

impl AppNotificationSettingsInternal {
    pub fn get_setting(&self, event: NotificationEvent) -> bool {
        match event {
            NotificationEvent::NewsChanged => self.account.news,
            NotificationEvent::ReceivedLikesChanged => self.chat.likes,
            NotificationEvent::NewMessageReceived => self.chat.messages,
            NotificationEvent::MediaContentModerationCompleted => {
                self.media.media_content_moderation
            }
            NotificationEvent::ProfileStringModerationCompleted => {
                self.profile.profile_string_moderation
            }
            NotificationEvent::AutomaticProfileSearchCompleted => {
                self.profile.automatic_profile_search
            }
            NotificationEvent::AdminNotification => true,
        }
    }
}
