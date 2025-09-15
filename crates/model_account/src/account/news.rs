use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use model::{NewsSyncVersion, UnreadNewsCount};
use model_server_data::{NewsIteratorState, PublicationId};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb};

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::news)]
#[diesel(check_for_backend(crate::Db))]
pub struct NewsItemInternal {
    pub id: NewsId,
    pub account_id_creator: Option<AccountIdDb>,
    pub first_publication_unix_time: Option<UnixTime>,
    pub latest_publication_unix_time: Option<UnixTime>,
    pub publication_id: Option<PublicationId>,
}

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::news_translations)]
#[diesel(check_for_backend(crate::Db))]
pub struct NewsTranslationInternal {
    pub locale: String,
    pub news_id: NewsId,
    pub title: String,
    pub body: String,
    pub creation_unix_time: UnixTime,
    pub version_number: NewsTranslationVersion,
    pub account_id_creator: Option<AccountIdDb>,
    pub account_id_editor: Option<AccountIdDb>,
    pub edit_unix_time: Option<UnixTime>,
}

/// News ID
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct NewsId {
    pub nid: i64,
}

impl NewsId {
    /// The value is the same as [crate::MatchId::next_id_to_latest_used_id]
    /// returns if there is no items.
    pub const NO_NEWS_ID: NewsId = NewsId { nid: -1 };

    pub fn new(id: i64) -> Self {
        Self { nid: id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.nid
    }
}

diesel_i64_wrapper!(NewsId);

impl From<NewsId> for i64 {
    fn from(value: NewsId) -> Self {
        value.nid
    }
}

/// News translation version which prevents editing
/// newer version than user has seen.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct NewsTranslationVersion {
    pub version: i64,
}

impl NewsTranslationVersion {
    pub fn new(version: i64) -> Self {
        Self { version }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.version
    }
}

diesel_i64_wrapper!(NewsTranslationVersion);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetNewsIteratorResult {
    pub s: NewsIteratorState,
    pub v: NewsSyncVersion,
    pub c: UnreadNewsCount,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsPage {
    pub news: Vec<NewsItemSimple>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsItem {
    pub title: String,
    pub body: String,
    pub locale: String,
    /// Latest publication time
    pub time: Option<UnixTime>,
    /// Option<i64> is a workaround for Dart OpenApi generator version 7.9.0
    pub edit_unix_time: Option<i64>,
    /// Only visible for accounts which have some news permissions
    pub aid_creator: Option<AccountId>,
    /// Only visible for accounts which have some news permissions
    pub aid_editor: Option<AccountId>,
    /// Only visible for accounts which have some news permissions
    pub version: Option<NewsTranslationVersion>,
}

impl NewsItem {
    pub fn clear_admin_info(&mut self) {
        self.aid_creator = None;
        self.aid_editor = None;
        self.version = None;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetNewsItemResult {
    pub item: Option<NewsItem>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub private: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsItemSimple {
    pub id: NewsId,
    pub title: Option<String>,
    /// Latest publication time
    pub time: Option<UnixTime>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub private: bool,
}

/// Value "default" or language code.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct NewsLocale {
    pub locale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct RequireNewsLocale {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[param(default = false)]
    pub require_locale: bool,
}

impl NewsLocale {
    pub const DEFAULT: &'static str = "default";
}

impl Default for NewsLocale {
    fn default() -> Self {
        Self {
            locale: Self::DEFAULT.to_string(),
        }
    }
}
