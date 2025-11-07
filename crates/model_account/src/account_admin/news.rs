use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::ToSchema;

use crate::{AccountId, NewsId, NewsItem, NewsTranslationVersion};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsTranslations {
    pub id: NewsId,
    pub public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aid_creator: Option<AccountId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_publication_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_publication_time: Option<UnixTime>,
    pub translations: Vec<NewsItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct UpdateNewsTranslation {
    pub title: String,
    pub body: String,
    pub current_version: NewsTranslationVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct UpdateNewsTranslationResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_already_changed: bool,
}

impl UpdateNewsTranslationResult {
    pub fn success() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn error_already_changed() -> Self {
        Self {
            error: true,
            error_already_changed: true,
        }
    }
}
