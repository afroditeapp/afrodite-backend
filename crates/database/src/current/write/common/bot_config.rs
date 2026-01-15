use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::BackendConfig;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentWriteCommonBotConfig);

impl CurrentWriteCommonBotConfig<'_> {
    pub fn upsert_bot_config(&mut self, config: &BackendConfig) -> Result<(), DieselDatabaseError> {
        use model::schema::bot_config::dsl::*;

        // Ensure user_bots fits in i16
        let user_bots_i16 = if config.user_bots > i16::MAX as u32 {
            i16::MAX
        } else {
            config.user_bots as i16
        };

        insert_into(bot_config)
            .values((
                row_type.eq(0),
                user_bots.eq(user_bots_i16),
                admin_bot.eq(config.admin_bot),
                remote_bot_login.eq(config.remote_bot_login),
            ))
            .on_conflict(row_type)
            .do_update()
            .set((
                user_bots.eq(user_bots_i16),
                admin_bot.eq(config.admin_bot),
                remote_bot_login.eq(config.remote_bot_login),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
