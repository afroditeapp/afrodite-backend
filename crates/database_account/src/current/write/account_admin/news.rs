
use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{delete, insert_into, prelude::*};
use error_stack::Result;
use model::{
    AccountIdInternal, NewsId
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountNewsAdmin, CurrentSyncWriteAccountNewsAdmin);

impl<C: ConnectionProvider> CurrentSyncWriteAccountNewsAdmin<C> {
    pub fn create_new_news_item(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<NewsId, DieselDatabaseError> {
        use model::schema::news::dsl::*;

        let news_id_value: NewsId = insert_into(news)
            .values((
                account_id_creator.eq(id_value.as_db_id()),
            ))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(news_id_value)
    }

    pub fn delete_news_item(
        &mut self,
        id_value: NewsId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::news::dsl::*;

        delete(news)
            .filter(id.eq(id_value))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
