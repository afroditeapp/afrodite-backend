use diesel::prelude::*;
use error_stack::Result;
use model::NextQueueNumberType;
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonQueueNumber);

impl CurrentReadCommonQueueNumber<'_> {
    pub fn next_queue_number(
        &mut self,
        queue: NextQueueNumberType,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::next_queue_number::dsl::*;

        let number = next_queue_number
            .filter(queue_type_number.eq(queue))
            .select(next_number)
            .first(self.conn())
            .optional()
            .into_db_error(queue)?
            .unwrap_or(0);

        Ok(number)
    }

    /// Smallest active queue number.
    pub fn smallest_queue_number(
        &mut self,
        queue: NextQueueNumberType,
    ) -> Result<Option<i64>, DieselDatabaseError> {
        use crate::schema::queue_entry::dsl::*;

        let number = queue_entry
            .filter(queue_type_number.eq(queue))
            .select(queue_number)
            .order(queue_number.asc())
            .first(self.conn())
            .optional()
            .into_db_error(queue)?;

        Ok(number)
    }
}
