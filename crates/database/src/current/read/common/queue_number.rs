use diesel::prelude::*;
use error_stack::Result;
use model::NextQueueNumberType;
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};
use tokio_stream::StreamExt;

use crate::IntoDatabaseError;

define_read_commands!(
    CurrentReadAccountQueueNumber,
    CurrentSyncReadCommonQueueNumber
);

impl<C: ConnectionProvider> CurrentSyncReadCommonQueueNumber<C> {
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
}
