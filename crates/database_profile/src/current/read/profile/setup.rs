use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountIdInternal, ProfileSetup
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadProfileFavorite, CurrentSyncReadProfileSetup);

impl<C: ConnectionProvider> CurrentSyncReadProfileSetup<C> {
    pub fn profile_setup(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ProfileSetup, DieselDatabaseError> {
        use crate::schema::profile_setup::dsl::*;

        profile_setup
            .filter(account_id.eq(id.as_db_id()))
            .select(ProfileSetup::as_select())
            .first(self.conn())
            .into_db_error(id)
    }
}
