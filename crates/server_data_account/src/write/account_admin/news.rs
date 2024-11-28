use model_account::{AccountIdInternal, NewsId, NewsLocale, UpdateNewsTranslation};
use server_data::{
    define_cmd_wrapper_write, result::Result, DataError
};

use crate::write::DbTransactionAccount;

define_cmd_wrapper_write!(WriteCommandsAccountNewsAdmin);

impl WriteCommandsAccountNewsAdmin<'_> {

    pub async fn create_news_item(
        &self,
        id: AccountIdInternal,
    ) -> Result<NewsId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().create_new_news_item(id)
        })
    }

    pub async fn delete_news_item(
        &self,
        id: NewsId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().delete_news_item(id)
        })
    }

    pub async fn upsert_news_translation(
        &self,
        id: AccountIdInternal,
        nid: NewsId,
        locale: NewsLocale,
        content: UpdateNewsTranslation,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().upsert_news_translation(id, nid, locale, content)
        })
    }

    pub async fn delete_news_translation(
        &self,
        nid: NewsId,
        locale: NewsLocale,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().delete_news_translation(nid, locale)
        })
    }

    pub async fn set_news_publicity(
        &self,
        nid: NewsId,
        is_public: bool,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().set_news_publicity(nid, is_public)
        })
    }
}
