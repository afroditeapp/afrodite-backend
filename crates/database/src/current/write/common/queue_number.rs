use diesel::{insert_into, prelude::*, update, delete};
use error_stack::Result;
use model::{AccountIdInternal, AccountState, Capabilities, SharedState, SharedStateInternal, NextQueueNumberType, QueueEntryRaw};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteAccountQueueNumber, CurrentSyncWriteCommonQueueNumber);

impl<C: ConnectionProvider> CurrentSyncWriteCommonQueueNumber<C> {
    fn next_queue_number_and_update_number_table(
        &mut self,
        queue: NextQueueNumberType,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::next_queue_number::dsl::*;

        let next = self.read().common().queue_number().next_queue_number(queue)?;
        let new_next = next + 1;

        insert_into(next_queue_number)
            .values((
                queue_type_number.eq(queue),
                next_number.eq(new_next)
            ))
            .on_conflict(queue_type_number)
            .do_update()
            .set(next_number.eq(new_next))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (queue, next, new_next))?;

        Ok(next)
    }

    pub fn create_new_queue_entry(
        &mut self,
        request_creator: AccountIdInternal,
        queue_type: NextQueueNumberType,
    ) -> Result<i64, DieselDatabaseError> {
        use model::schema::queue_entry::dsl::*;

        let number = self.next_queue_number_and_update_number_table(queue_type)?;

        insert_into(queue_entry)
            .values(
                (
                    account_id.eq(request_creator.as_db_id()),
                    queue_number.eq(number),
                    queue_type_number.eq(queue_type),
                ),
            )
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, request_creator)?;

        Ok(number)
    }

    pub fn delete_queue_entry(
        &mut self,
        queue_number_entry: i64,
        queue: NextQueueNumberType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::queue_entry::dsl::*;

        delete(queue_entry.filter(queue_number.eq(queue_number_entry).and(queue_type_number.eq(queue))))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, queue_number_entry)?;

        Ok(())
    }
}
