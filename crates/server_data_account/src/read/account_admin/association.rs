use database_account::current::read::GetDbReadCommandsAccount;
use model::AccountIdInternal;
use model_account::{
    AssociationMember, AssociationMembersPage, GetAssociationMembersPage,
    ManualAssociationMembershipRegistry,
};
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

    pub async fn get_manual_registry(
        &self,
    ) -> Result<ManualAssociationMembershipRegistry, DataError> {
        self.db_read(move |mut cmds| cmds.account_admin().association().get_manual_registry())
            .await
            .into_error()
            .map(|v| ManualAssociationMembershipRegistry { registry: v })
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

    pub async fn get_entry(
        &self,
        member: AccountIdInternal,
    ) -> Result<Option<AssociationMember>, DataError> {
        self.db_read(move |mut cmds| cmds.account_admin().association().get_entry(member))
            .await
            .into_error()
    }
}
