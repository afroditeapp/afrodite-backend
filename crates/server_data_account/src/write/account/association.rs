use database_account::current::write::GetDbWriteCommandsAccount;
use model::{AccountIdInternal, UnixTime};
use model_account::{AssociationMembershipEntryInternal, UpdateAssociationMembership};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountAssociation);

impl WriteCommandsAccountAssociation<'_> {
    pub async fn upsert_own_entry(
        &self,
        id: AccountIdInternal,
        input: UpdateAssociationMembership,
    ) -> Result<(), DataError> {
        let now = UnixTime::current_time();

        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .association()
                .upsert_own_entry(AssociationMembershipEntryInternal {
                    account_id_member: *id.as_db_id(),
                    account_id_creator: Some(*id.as_db_id()),
                    account_id_editor: Some(*id.as_db_id()),
                    creation_unix_time: now,
                    edit_unix_time: now,
                    full_name: input.full_name,
                    domicile: input.domicile,
                    membership_type: input.membership_type,
                })
        })
    }

    pub async fn remove_own_entry(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().association().remove_own_entry(id)
        })
    }
}
