use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
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
