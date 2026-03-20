use diesel::{Selectable, expression::AsExpression, prelude::Queryable, sql_types::SmallInt};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;
use utoipa::ToSchema;

use crate::AdminNotificationBitflags;

/// App notification types
///
/// # Notification specific data
///
/// ## Admin notification
///
/// Integer payload contains the following bitflags:
///
/// * MODERATE_INITIAL_MEDIA_CONTENT_BOT = 1 << 0
/// * MODERATE_INITIAL_MEDIA_CONTENT_HUMAN = 1 << 1
/// * MODERATE_MEDIA_CONTENT_BOT = 1 << 2
/// * MODERATE_MEDIA_CONTENT_HUMAN = 1 << 3
/// * MODERATE_PROFILE_TEXTS_BOT = 1 << 4
/// * MODERATE_PROFILE_TEXTS_HUMAN = 1 << 5
/// * MODERATE_PROFILE_NAMES_BOT = 1 << 6
/// * MODERATE_PROFILE_NAMES_HUMAN = 1 << 7
/// * PROCESS_REPORTS = 1 << 8
///
/// ## News changed
///
/// Integer payload contains current unread news count.
///
/// ## Received likes changed
///
/// Integer payload contains current received likes count.
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
    AdminNotification = 0,
    // 20..39: account
    NewsChanged = 20,
    // 40..59: media
    MediaContentModerationAccepted = 40,
    MediaContentModerationRejected = 41,
    MediaContentModerationDeleted = 42,
    // 60..79: profile
    ProfileNameModerationAccepted = 60,
    ProfileNameModerationRejected = 61,
    ProfileTextModerationAccepted = 62,
    ProfileTextModerationRejected = 63,
    AutomaticProfileSearchCompleted = 64,
    // 80..99: chat
    ReceivedLikesChanged = 80,
}

#[derive(Debug, Clone, Copy)]
pub enum PendingAppNotificationInternal {
    AdminNotification { bitflags: AdminNotificationBitflags },
    NewsChanged { unread_news_count: i64 },
    ReceivedLikesChanged { new_received_likes_count: i64 },
    MediaContentModerationAccepted,
    MediaContentModerationRejected,
    MediaContentModerationDeleted,
    ProfileNameModerationAccepted,
    ProfileNameModerationRejected,
    ProfileTextModerationAccepted,
    ProfileTextModerationRejected,
    AutomaticProfileSearchCompleted { profile_count: i64 },
}

impl PendingAppNotificationInternal {
    pub fn into_db_values(self) -> (PendingAppNotificationType, Option<i64>) {
        match self {
            Self::AdminNotification { bitflags } => (
                PendingAppNotificationType::AdminNotification,
                Some(bitflags.bits()),
            ),
            Self::NewsChanged { unread_news_count } => (
                PendingAppNotificationType::NewsChanged,
                Some(unread_news_count),
            ),
            Self::ReceivedLikesChanged {
                new_received_likes_count,
            } => (
                PendingAppNotificationType::ReceivedLikesChanged,
                Some(new_received_likes_count),
            ),
            Self::MediaContentModerationAccepted => (
                PendingAppNotificationType::MediaContentModerationAccepted,
                None,
            ),
            Self::MediaContentModerationRejected => (
                PendingAppNotificationType::MediaContentModerationRejected,
                None,
            ),
            Self::MediaContentModerationDeleted => (
                PendingAppNotificationType::MediaContentModerationDeleted,
                None,
            ),
            Self::ProfileNameModerationAccepted => (
                PendingAppNotificationType::ProfileNameModerationAccepted,
                None,
            ),
            Self::ProfileNameModerationRejected => (
                PendingAppNotificationType::ProfileNameModerationRejected,
                None,
            ),
            Self::ProfileTextModerationAccepted => (
                PendingAppNotificationType::ProfileTextModerationAccepted,
                None,
            ),
            Self::ProfileTextModerationRejected => (
                PendingAppNotificationType::ProfileTextModerationRejected,
                None,
            ),
            Self::AutomaticProfileSearchCompleted { profile_count } => (
                PendingAppNotificationType::AutomaticProfileSearchCompleted,
                Some(profile_count),
            ),
        }
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_integer: Option<i64>,
}
