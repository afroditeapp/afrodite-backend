use database_account::current::read::GetDbReadCommandsAccount;
use model::AccountIdInternal;
use model_account::{AssociationMemberManual, AssociationMembersPage, GetAssociationMembersPage};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountAssociationAdmin);

impl ReadCommandsAccountAssociationAdmin<'_> {
    pub async fn all_account_ids_with_membership(
        &self,
    ) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account_admin()
                .association()
                .all_account_ids_with_membership()
        })
        .await
        .into_error()
    }

    pub async fn get_all_manual(&self) -> Result<Vec<AssociationMemberManual>, DataError> {
        self.db_read(move |mut cmds| {
            let entries = cmds.account_admin().association().get_all_manual()?;
            Ok(entries)
        })
        .await
        .into_error()
    }

    pub async fn get_page(
        &self,
        query: GetAssociationMembersPage,
    ) -> Result<AssociationMembersPage, DataError> {
        self.db_read(move |mut cmds| {
            let entries = cmds.account_admin().association().get_page(&query)?;
            Ok(entries)
        })
        .await
        .into_error()
    }
}
