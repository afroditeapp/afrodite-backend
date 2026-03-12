use diesel::{Selectable, expression::AsExpression, prelude::Queryable, sql_types::SmallInt};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;
use utoipa::ToSchema;

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    ToSchema,
    TryFromPrimitive,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum PendingAppNotificationType {
    // 0..19: common
    // 20..39: account
    // 40..59: media
    MediaContentModerationAccepted = 40,
    MediaContentModerationRejected = 41,
    MediaContentModerationDeleted = 42,
    // 60..79: profile
    ProfileNameModerationAccepted = 60,
    ProfileNameModerationRejected = 61,
    ProfileTextModerationAccepted = 62,
    ProfileTextModerationRejected = 63,
    // 80..99: chat
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct PendingAppNotificationList {
    pub notifications: Vec<PendingAppNotification>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Queryable, Selectable)]
#[diesel(table_name = crate::schema::pending_app_notifications)]
#[diesel(check_for_backend(crate::Db))]
pub struct PendingAppNotification {
    #[diesel(column_name = notification_type_number)]
    pub notification_type: PendingAppNotificationType,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub push_notification_sent: bool,
}
