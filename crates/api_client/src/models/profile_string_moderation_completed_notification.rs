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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileStringModerationCompletedNotification {
    #[serde(rename = "name_accepted")]
    pub name_accepted: Box<models::NotificationStatus>,
    #[serde(rename = "name_rejected")]
    pub name_rejected: Box<models::NotificationStatus>,
    #[serde(rename = "text_accepted")]
    pub text_accepted: Box<models::NotificationStatus>,
    #[serde(rename = "text_rejected")]
    pub text_rejected: Box<models::NotificationStatus>,
}

impl ProfileStringModerationCompletedNotification {
    pub fn new(name_accepted: models::NotificationStatus, name_rejected: models::NotificationStatus, text_accepted: models::NotificationStatus, text_rejected: models::NotificationStatus) -> ProfileStringModerationCompletedNotification {
        ProfileStringModerationCompletedNotification {
            name_accepted: Box::new(name_accepted),
            name_rejected: Box::new(name_rejected),
            text_accepted: Box::new(text_accepted),
            text_rejected: Box::new(text_rejected),
        }
    }
}

