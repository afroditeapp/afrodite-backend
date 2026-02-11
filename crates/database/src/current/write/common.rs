use diesel::{ExpressionMethods, RunQueryDsl, insert_into};
use error_stack::Result;
use model::AccountIdInternal;
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_write_commands};

mod bot_config;
mod client_config;
pub mod profile_attributes;
mod push_notification;
mod report;
mod state;
mod token;

define_current_write_commands!(CurrentWriteCommon);

impl<'a> CurrentWriteCommon<'a> {
    pub fn profile_attributes(self) -> profile_attributes::CurrentWriteCommonProfileAttributes<'a> {
        profile_attributes::CurrentWriteCommonProfileAttributes::new(self.cmds)
    }

    pub fn state(self) -> state::CurrentWriteCommonState<'a> {
        state::CurrentWriteCommonState::new(self.cmds)
    }

    pub fn token(self) -> token::CurrentWriteAccountToken<'a> {
        token::CurrentWriteAccountToken::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentWriteCommonReport<'a> {
        report::CurrentWriteCommonReport::new(self.cmds)
    }

    pub fn bot_config(self) -> bot_config::CurrentWriteCommonBotConfig<'a> {
        bot_config::CurrentWriteCommonBotConfig::new(self.cmds)
    }

    pub fn client_config(self) -> client_config::CurrentWriteCommonClientConfig<'a> {
        client_config::CurrentWriteCommonClientConfig::new(self.cmds)
    }

    pub fn push_notification(self) -> push_notification::CurrentWriteCommonPushNotification<'a> {
        push_notification::CurrentWriteCommonPushNotification::new(self.cmds)
    }
}

impl CurrentWriteCommon<'_> {
    pub fn insert_account_id(
        mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        insert_into(account_id)
            .values((
                id.eq(account_id_value.as_db_id()),
                uuid.eq(account_id_value.uuid),
            ))
            .execute(self.conn())
            .into_db_error(account_id_value)?;

        Ok(())
    }

    pub fn insert_common_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        insert_into(common_state)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_push_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        insert_into(push_notification)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
