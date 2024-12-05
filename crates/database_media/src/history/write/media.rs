use database::{define_history_write_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use model::{AccountIdInternal, ContentId};
use error_stack::Result;

define_history_write_commands!(HistoryWriteMedia);

impl HistoryWriteMedia<'_> {
    pub fn get_next_unique_content_id(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ContentId, DieselDatabaseError> {
        use model::schema::history_used_content_ids::dsl::*;

        let random_cid = ContentId::new_random();

        insert_into(history_used_content_ids)
            .values((
                account_id.eq(id.as_db_id()),
                uuid.eq(random_cid),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(random_cid)
    }
}
