use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountAssociationAdmin);

impl WriteCommandsAccountAssociationAdmin<'_> {
    pub async fn upsert_manual_registry(&self, registry: String) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .association()
                .upsert_manual_registry(&registry)
        })
    }

    pub async fn delete_entry(&self, member: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().association().delete_entry(member)
        })
    }
}
