use diesel::{insert_into, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{AccessToken, AccountIdInternal, RefreshToken};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteAccountToken);

impl CurrentWriteAccountToken<'_> {
    pub fn insert_access_token(
        mut self,
        id: AccountIdInternal,
        token_value: Option<AccessToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::access_token::dsl::*;

        let token_value = token_value.as_ref().map(|k| k.as_str());

        insert_into(access_token)
            .values((account_id.eq(id.as_db_id()), token.eq(token_value)))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn access_token(
        mut self,
        id: AccountIdInternal,
        token_value: Option<AccessToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::access_token::dsl::*;

        let token_value = token_value.as_ref().map(|k| k.as_str());

        update(access_token.find(id.as_db_id()))
            .set(token.eq(token_value))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_refresh_token(
        mut self,
        id: AccountIdInternal,
        token_value: Option<RefreshToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::refresh_token::dsl::*;

        let token_value = if let Some(t) = token_value {
            Some(
                t.bytes()
                    .change_context(DieselDatabaseError::DataFormatConversion)?,
            )
        } else {
            None
        };

        insert_into(refresh_token)
            .values((account_id.eq(id.as_db_id()), token.eq(token_value)))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn refresh_token(
        &mut self,
        id: AccountIdInternal,
        token_value: Option<RefreshToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::refresh_token::dsl::*;

        let token_value = if let Some(t) = token_value {
            Some(
                t.bytes()
                    .change_context(DieselDatabaseError::DataFormatConversion)?,
            )
        } else {
            None
        };

        update(refresh_token.find(id.as_db_id()))
            .set(token.eq(token_value))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
