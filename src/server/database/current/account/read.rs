
use async_trait::async_trait;
use error_stack::Result;



use crate::server::database::current::SqliteReadCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteSelectJson};
use crate::api::account::data::AccountSetup;

use crate::api::model::{
    *
};

use crate::utils::{IntoReportExt};

use crate::read_json;


#[async_trait]
impl SqliteSelectJson for Account {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM Account
            WHERE account_row_id = ?
            "#,
            json_text
        )
    }
}



#[async_trait]
impl SqliteSelectJson for AccountSetup {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM AccountSetup
            WHERE account_row_id = ?
            "#,
            json_text
        )
    }
}
