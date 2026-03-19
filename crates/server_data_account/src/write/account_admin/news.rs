use database_account::current::write::GetDbWriteCommandsAccount;
use model_account::{
    AccountIdInternal, NewsId, NewsLocale, NotificationEvent, UpdateNewsTranslation,
};
use server_data::{
    DataError, db_manager::InternalWriting, db_transaction, define_cmd_wrapper_write,
    result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountNewsAdmin);

impl WriteCommandsAccountNewsAdmin<'_> {
    pub async fn create_news_item(&self, id: AccountIdInternal) -> Result<NewsId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().news().create_new_news_item(id)
        })
    }

    pub async fn delete_news_item(&self, id: NewsId) -> Result<(), DataError> {
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
            cmds.account_admin()
                .news()
                .upsert_news_translation(id, nid, locale, content)
        })
    }

    pub async fn delete_news_translation(
        &self,
        nid: NewsId,
        locale: NewsLocale,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .news()
                .delete_news_translation(nid, locale)
        })
    }

    pub async fn set_news_publicity(&self, nid: NewsId, is_public: bool) -> Result<(), DataError> {
        let send_notification = db_transaction!(self, move |mut cmds| {
            let send_notification = cmds
                .account_admin()
                .news()
                .set_news_publicity(nid, is_public)?;
            if send_notification {
                cmds.account_admin()
                    .news()
                    .upsert_news_pending_notification_for_every_account()?;
            }
            Ok(send_notification)
        })?;

        if send_notification {
            self.events()
                .send_low_priority_notification_to_logged_in_clients(NotificationEvent::NewsChanged)
                .await;
        }

        Ok(())
    }
}
