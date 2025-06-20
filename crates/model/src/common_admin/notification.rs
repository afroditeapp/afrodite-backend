use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Admin notification values or subscription info
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, Queryable, Selectable, AsChangeset, Insertable, ToSchema)]
#[diesel(table_name = crate::schema::admin_notification_subscriptions)]
#[diesel(check_for_backend(crate::Db))]
pub struct AdminNotification {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_initial_media_content_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_initial_media_content_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_media_content_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_media_content_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_texts_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_texts_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_names_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub moderate_profile_names_human: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub process_reports: bool,
}

impl AdminNotification {
    pub fn enable(&mut self, event: AdminNotificationTypes) {
        match event {
            AdminNotificationTypes::ModerateInitialMediaContentBot =>
                self.moderate_initial_media_content_bot = true,
            AdminNotificationTypes::ModerateInitialMediaContentHuman =>
                self.moderate_initial_media_content_human = true,
            AdminNotificationTypes::ModerateMediaContentBot =>
                self.moderate_media_content_bot = true,
            AdminNotificationTypes::ModerateMediaContentHuman =>
                self.moderate_media_content_human = true,
            AdminNotificationTypes::ModerateProfileTextsBot =>
                self.moderate_profile_texts_bot = true,
            AdminNotificationTypes::ModerateProfileTextsHuman =>
                self.moderate_profile_texts_human = true,
            AdminNotificationTypes::ModerateProfileNamesBot =>
                self.moderate_profile_names_bot = true,
            AdminNotificationTypes::ModerateProfileNamesHuman =>
                self.moderate_profile_names_human = true,
            AdminNotificationTypes::ProcessReports =>
                self.process_reports = true,
        }
    }
}

pub enum AdminNotificationTypes {
    ModerateInitialMediaContentBot,
    ModerateInitialMediaContentHuman,
    ModerateMediaContentBot,
    ModerateMediaContentHuman,
    ModerateProfileTextsBot,
    ModerateProfileTextsHuman,
    ModerateProfileNamesBot,
    ModerateProfileNamesHuman,
    ProcessReports,
}
