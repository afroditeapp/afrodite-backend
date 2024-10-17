
use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, AccountInternal, ClientId, EmailAddress, SetAccountSetup, ACCOUNT_GLOBAL_STATE_ROW_TYPE
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountNewsAdmin, CurrentSyncWriteAccountNewsAdmin);

impl<C: ConnectionProvider> CurrentSyncWriteAccountNewsAdmin<C> {

}
