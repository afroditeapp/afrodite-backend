use async_trait::async_trait;
use serde::Serialize;

use crate::api::model::{Account, AccountId};

use super::{SqliteDatabaseError, write::SqliteWriteCommands};

use error_stack::Result;


#[async_trait]
pub trait SqliteUpdateJson {
    async fn update_json(
        &self, id: &AccountId, write: &SqliteWriteCommands,
    ) -> Result<(), SqliteDatabaseError>;
}
