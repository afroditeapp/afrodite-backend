/*
 * afrodite-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// EventType : Identifier for event.
/// Identifier for event.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "AccountStateChanged")]
    AccountStateChanged,
    #[serde(rename = "NewMessageReceived")]
    NewMessageReceived,
    #[serde(rename = "ReceivedLikesChanged")]
    ReceivedLikesChanged,
    #[serde(rename = "ContentProcessingStateChanged")]
    ContentProcessingStateChanged,
    #[serde(rename = "ClientConfigChanged")]
    ClientConfigChanged,
    #[serde(rename = "ProfileChanged")]
    ProfileChanged,
    #[serde(rename = "NewsCountChanged")]
    NewsCountChanged,
    #[serde(rename = "MediaContentModerationCompleted")]
    MediaContentModerationCompleted,
    #[serde(rename = "MediaContentChanged")]
    MediaContentChanged,
    #[serde(rename = "DailyLikesLeftChanged")]
    DailyLikesLeftChanged,
    #[serde(rename = "ScheduledMaintenanceStatus")]
    ScheduledMaintenanceStatus,
    #[serde(rename = "ProfileStringModerationCompleted")]
    ProfileStringModerationCompleted,
    #[serde(rename = "AutomaticProfileSearchCompleted")]
    AutomaticProfileSearchCompleted,
    #[serde(rename = "AdminNotification")]
    AdminNotification,

}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AccountStateChanged => write!(f, "AccountStateChanged"),
            Self::NewMessageReceived => write!(f, "NewMessageReceived"),
            Self::ReceivedLikesChanged => write!(f, "ReceivedLikesChanged"),
            Self::ContentProcessingStateChanged => write!(f, "ContentProcessingStateChanged"),
            Self::ClientConfigChanged => write!(f, "ClientConfigChanged"),
            Self::ProfileChanged => write!(f, "ProfileChanged"),
            Self::NewsCountChanged => write!(f, "NewsCountChanged"),
            Self::MediaContentModerationCompleted => write!(f, "MediaContentModerationCompleted"),
            Self::MediaContentChanged => write!(f, "MediaContentChanged"),
            Self::DailyLikesLeftChanged => write!(f, "DailyLikesLeftChanged"),
            Self::ScheduledMaintenanceStatus => write!(f, "ScheduledMaintenanceStatus"),
            Self::ProfileStringModerationCompleted => write!(f, "ProfileStringModerationCompleted"),
            Self::AutomaticProfileSearchCompleted => write!(f, "AutomaticProfileSearchCompleted"),
            Self::AdminNotification => write!(f, "AdminNotification"),
        }
    }
}

impl Default for EventType {
    fn default() -> EventType {
        Self::AccountStateChanged
    }
}

