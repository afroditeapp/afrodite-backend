use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{AccountIdInternal, EventToClientInternal, ProfileReportContent, ProfileTextModerationState, UpdateReportResult};
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::DbTransaction,
    DataError,
};

use crate::write::{profile_admin::profile_text::ModerateProfileTextMode, GetWriteCommandsProfile};

define_cmd_wrapper_write!(WriteCommandsProfileReport);

impl WriteCommandsProfileReport<'_> {
    pub async fn report_profile_text(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_text: String,
    ) -> Result<UpdateReportResult, DataError> {
        let mut current_report = self
            .db_read(move |mut cmds| cmds.profile().report().get_report(creator, target))
            .await?;

        current_report.content.profile_text = Some(profile_text);

        self.update_report(creator, target, current_report.content).await
    }

    pub async fn update_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        reported_content: ProfileReportContent,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.profile().data().my_profile(target, None))
            .await?;

        if let Some(reported_text) = reported_content.profile_text.as_deref() {
            if reported_text != target_data.p.ptext {
                return Ok(UpdateReportResult::outdated_report_content());
            }

            if target_data.text_moderation_info.state == ProfileTextModerationState::AcceptedByBot {
                self.handle().profile_admin().profile_text().moderate_profile_text(
                    ModerateProfileTextMode::MoveToHumanModeration,
                    target,
                    reported_text.to_string(),
                ).await?;

                self.handle()
                    .events()
                    .send_connected_event(target, EventToClientInternal::ProfileChanged)
                    .await?;
            }
        }

        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .report()
                .upsert_report(creator, target, reported_content)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }
}
