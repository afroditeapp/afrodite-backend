use database::current::write::GetDbWriteCommandsCommon;
use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{AccountIdInternal, ReportTypeNumber};
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsProfileReport);

impl WriteCommandsProfileReport<'_> {
    pub async fn process_profile_name_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_name: String,
    ) -> Result<(), DataError> {
        let current_report = self
            .db_read(move |mut cmds| cmds.profile_admin().report().get_current_profile_name_report(creator, target))
            .await?;
        if current_report != Some(profile_name) {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.common_admin()
                .report()
                .mark_report_done(moderator_id, creator, target, ReportTypeNumber::ProfileName)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn process_profile_text_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_text: String,
    ) -> Result<(), DataError> {
        let current_report = self
            .db_read(move |mut cmds| cmds.profile_admin().report().get_current_profile_text_report(creator, target))
            .await?;
        if current_report != Some(profile_text) {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.common_admin()
                .report()
                .mark_report_done(moderator_id, creator, target, ReportTypeNumber::ProfileText)?;
            Ok(())
        })?;

        Ok(())
    }
}
