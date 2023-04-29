use async_trait::async_trait;
use error_stack::Result;

use crate::api::account::data::AccountSetup;
use crate::server::database::current::CurrentDataWriteCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteUpdateJson};

use crate::api::model::*;

use crate::utils::IntoReportExt;

use crate::insert_or_update_json;

#[async_trait]
impl SqliteUpdateJson for Account {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &CurrentDataWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE Account
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            self,
            id
        )
    }
}

#[async_trait]
impl SqliteUpdateJson for AccountSetup {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &CurrentDataWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE AccountSetup
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            self,
            id
        )
    }
}
