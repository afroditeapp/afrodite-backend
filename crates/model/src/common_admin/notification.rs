use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Clone, Default, Deserialize, Serialize, Queryable, Selectable, AsChangeset, Insertable, ToSchema)]
#[diesel(table_name = crate::schema::admin_notification_subscriptions)]
#[diesel(check_for_backend(crate::Db))]
pub struct AdminNotificationSubscriptions {
    pub moderate_media_content_bot: bool,
    pub moderate_media_content_human: bool,
    pub moderate_profile_texts_bot: bool,
    pub moderate_profile_texts_human: bool,
    pub moderate_profile_names_bot: bool,
    pub moderate_profile_names_human: bool,
    pub process_reports: bool,
}
