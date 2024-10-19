use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, UnixTime};
use utoipa::{IntoParams, ToSchema};

use crate::{
    AccountId, AccountIdDb, NextNumberStorage
};

use crate::{sync_version_wrappers, SyncVersion, SyncVersionUtils};

sync_version_wrappers!(
    NewsSyncVersion,
);


#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::news)]
#[diesel(check_for_backend(crate::Db))]
pub struct NewsItemInternal {
    pub id: NewsId,
}

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::news_translations)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct NewsTranslationInternal {
    pub locale: String,
    pub news_id: NewsId,
    pub title: String,
    pub body: String,
    pub creation_unix_time: UnixTime,
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

/// Session ID type for news iterator so that client can detect
/// server restarts and ask user to refresh news.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewsIteratorSessionIdInternal {
    id: i64,
}

impl NewsIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Session ID type for news iterator so that client can detect
/// server restarts and ask user to refresh news.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct NewsIteratorSessionId {
    id: i64,
}

impl From<NewsIteratorSessionIdInternal> for NewsIteratorSessionId {
    fn from(value: NewsIteratorSessionIdInternal) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl From<NewsIteratorSessionId> for NewsIteratorSessionIdInternal {
    fn from(value: NewsIteratorSessionId) -> Self {
        Self {
            id: value.id,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct NewsCount {
    pub c: i64,
}

impl NewsCount {
    pub fn new(count: i64) -> Self {
        Self { c: count }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.c
    }
}

diesel_i64_wrapper!(NewsCount);


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewsCountResult {
    pub v: NewsSyncVersion,
    pub c: NewsCount,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetNewsIteratorResult {
    pub s: NewsIteratorSessionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsPage {
    pub news: Vec<NewsItemSimple>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_invalid_iterator_session_id: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NewsItem {
    pub title: String,
    pub body: String,
    pub creation_time: UnixTime,
    /// Only visible for accounts which have some news permissions
    pub aid_creator: Option<AccountId>,
    /// Only visible for accounts which have some news permissions
    pub aid_editor: Option<AccountId>,
    pub edit_time: Option<UnixTime>,
}

impl NewsItem {
    pub fn clear_admin_info(&mut self) {
        self.aid_creator = None;
        self.aid_editor = None;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetNewsItemResult {
    pub item: Option<NewsItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::news_translations)]
#[diesel(check_for_backend(crate::Db))]
pub struct NewsItemSimple {
    #[diesel(column_name = news_id)]
    pub id: NewsId,
    pub title: String,
    #[diesel(column_name = creation_unix_time)]
    pub creation_time: UnixTime,
}

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
    pub const ENGLISH: &'static str = "en";
    pub const FINNISH: &'static str = "fi";
}
