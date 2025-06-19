use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Admin notification values or subscription info
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Deserialize,
    Serialize,
    Queryable,
    Selectable,
    AsChangeset,
    Insertable,
    ToSchema,
)]
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
            AdminNotificationTypes::ModerateInitialMediaContentBot => {
                self.moderate_initial_media_content_bot = true
            }
            AdminNotificationTypes::ModerateInitialMediaContentHuman => {
                self.moderate_initial_media_content_human = true
            }
            AdminNotificationTypes::ModerateMediaContentBot => {
                self.moderate_media_content_bot = true
            }
            AdminNotificationTypes::ModerateMediaContentHuman => {
                self.moderate_media_content_human = true
            }
            AdminNotificationTypes::ModerateProfileTextsBot => {
                self.moderate_profile_texts_bot = true
            }
            AdminNotificationTypes::ModerateProfileTextsHuman => {
                self.moderate_profile_texts_human = true
            }
            AdminNotificationTypes::ModerateProfileNamesBot => {
                self.moderate_profile_names_bot = true
            }
            AdminNotificationTypes::ModerateProfileNamesHuman => {
                self.moderate_profile_names_human = true
            }
            AdminNotificationTypes::ProcessReports => self.process_reports = true,
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            moderate_initial_media_content_bot: self.moderate_initial_media_content_bot
                || other.moderate_initial_media_content_bot,
            moderate_initial_media_content_human: self.moderate_initial_media_content_human
                || other.moderate_initial_media_content_human,
            moderate_media_content_bot: self.moderate_media_content_bot
                || other.moderate_media_content_bot,
            moderate_media_content_human: self.moderate_media_content_human
                || other.moderate_media_content_human,
            moderate_profile_texts_bot: self.moderate_profile_texts_bot
                || other.moderate_profile_texts_bot,
            moderate_profile_texts_human: self.moderate_profile_texts_human
                || other.moderate_profile_texts_human,
            moderate_profile_names_bot: self.moderate_profile_names_bot
                || other.moderate_profile_names_bot,
            moderate_profile_names_human: self.moderate_profile_names_human
                || other.moderate_profile_names_human,
            process_reports: self.process_reports || other.process_reports,
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
