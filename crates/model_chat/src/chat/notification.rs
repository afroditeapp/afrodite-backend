use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
