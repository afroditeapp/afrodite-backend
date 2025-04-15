
use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use model::NotificationEvent;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Queryable, Selectable, AsChangeset, Insertable, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::account_app_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountAppNotificationSettings {
    pub news: bool,
}

impl Default for AccountAppNotificationSettings {
    fn default() -> Self {
        Self {
            news: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Queryable, Selectable, AsChangeset, Insertable, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::profile_app_notification_settings)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileAppNotificationSettings {
    pub profile_text_moderation: bool,
}

impl Default for ProfileAppNotificationSettings {
    fn default() -> Self {
        Self {
            profile_text_moderation: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Queryable, Selectable, AsChangeset, Insertable, Deserialize, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Copy, Queryable, Selectable, AsChangeset, Insertable, Deserialize, Serialize, ToSchema)]
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
            NotificationEvent::MediaContentModerationCompleted => self.media.media_content_moderation,
        }
    }
}
