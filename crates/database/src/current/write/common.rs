use diesel::{insert_into, prelude::*, update, delete};
use error_stack::Result;
use model::{AccountIdInternal, AccountState, Capabilities, SharedState, SharedStateInternal, NextQueueNumberType, QueueEntryRaw};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteAccount, CurrentSyncWriteCommon);

impl<C: ConnectionProvider> CurrentSyncWriteCommon<C> {
    pub fn insert_shared_state(
        &mut self,
        id: AccountIdInternal,
        data: SharedStateInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        insert_into(shared_state)
            .values((
                account_id.eq(id.as_db_id()),
                is_profile_public.eq(&data.is_profile_public),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn shared_state(
        &mut self,
        id: AccountIdInternal,
        data: SharedState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set((
                account_state_number.eq(data.account_state as i64),
                is_profile_public.eq(&data.is_profile_public),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn insert_default_account_capabilities(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_capabilities::dsl::*;

        insert_into(account_capabilities)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn account_capabilities(
        &mut self,
        id: AccountIdInternal,
        data: Capabilities,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_capabilities::dsl::*;

        update(account_capabilities.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn account_state(
        mut self,
        id: AccountIdInternal,
        state: AccountState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(account_state_number.eq(state as i64))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    fn next_queue_number_and_update_number_table(
        &mut self,
        queue: NextQueueNumberType,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::next_queue_number::dsl::*;

        let next = self.read().common().next_queue_number(queue)?;
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
