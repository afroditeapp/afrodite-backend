use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Default, Queryable, Selectable, AsChangeset, Insertable, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::media_app_notification_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaContentModerationCompleted {
    /// Show accepted notification
    pub media_content_accepted: bool,
    /// Show rejected notification
    pub media_content_rejected: bool,
}
