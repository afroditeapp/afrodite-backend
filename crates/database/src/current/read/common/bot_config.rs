use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::BotConfig;

use crate::{DieselDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonBotConfig);

impl CurrentReadCommonBotConfig<'_> {
    pub fn bot_config(&mut self) -> Result<Option<BotConfig>, DieselDatabaseError> {
        use crate::schema::bot_config::dsl::*;

        bot_config
            .filter(row_type.eq(0))
            .select((
                user_bots,
                admin_bot,
                remote_bot_login,
                admin_bot_config_json,
            ))
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
            .map(|opt| {
                opt.map(|(u, a, r, c): (i16, bool, bool, Option<String>)| {
                    let users = if u < 0 { 0 } else { u as u32 };
                    let config = c.and_then(|v| serde_json::from_str(&v).ok());
                    BotConfig {
                        user_bots: users,
                        admin_bot: a,
                        remote_bot_login: r,
                        admin_bot_config: config,
                    }
                })
            })
    }
}
