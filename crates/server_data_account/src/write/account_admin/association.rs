use database_account::current::write::GetDbWriteCommandsAccount;
use model::{AccountIdInternal, UnixTime};
use model_account::{
    AssociationMemberIdManual, NewAssociationMemberManualEntry, UpdateAssociationMemberManualEntry,
};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};
use simple_backend_utils::string::NonEmptyString;

define_cmd_wrapper_write!(WriteCommandsAccountAssociationAdmin);

impl WriteCommandsAccountAssociationAdmin<'_> {
    pub async fn create_entry_manual(
        &self,
        creator: AccountIdInternal,
        full_name: Option<NonEmptyString>,
        domicile: Option<NonEmptyString>,
        email: Option<NonEmptyString>,
        membership_type: i16,
    ) -> Result<AssociationMemberIdManual, DataError> {
        let now = UnixTime::current_time();

        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().association().insert_entry_manual(
                NewAssociationMemberManualEntry {
                    account_id_creator: Some(*creator.as_db_id()),
                    account_id_editor: Some(*creator.as_db_id()),
                    creation_unix_time: now,
                    edit_unix_time: now,
                    full_name,
                    domicile,
                    email,
                    membership_type,
                },
            )
        })
    }

    pub async fn edit_entry_manual(
        &self,
        editor: AccountIdInternal,
        entry_id: AssociationMemberIdManual,
        full_name: Option<NonEmptyString>,
        domicile: Option<NonEmptyString>,
        email: Option<NonEmptyString>,
        membership_type: i16,
    ) -> Result<(), DataError> {
        let now = UnixTime::current_time();

        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().association().update_entry_manual(
                entry_id,
                UpdateAssociationMemberManualEntry {
                    account_id_editor: *editor.as_db_id(),
                    edit_unix_time: now,
                    full_name,
                    domicile,
                    email,
                    membership_type,
                },
            )
        })
    }

    pub async fn delete_entry_manual(
        &self,
        entry_id: AssociationMemberIdManual,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .association()
                .delete_entry_manual(entry_id)
        })
    }

    pub async fn delete_entry(&self, member: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().association().delete_entry(member)
        })
    }
}
