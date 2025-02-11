use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{AccountIdInternal, EventToClientInternal, ProfileTextModerationState, UpdateProfileReportResult};
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
    pub async fn report_profile(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        reported_profile_text: Option<String>,
    ) -> Result<UpdateProfileReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.profile().data().my_profile(target, None))
            .await?;

        if let Some(reported_text) = reported_profile_text.as_deref() {
            if reported_text != target_data.p.ptext {
                return Ok(UpdateProfileReportResult::outdated_profile_text());
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
                .upsert_profile_report(creator, target, reported_profile_text)?;
            Ok(())
        })?;

        Ok(UpdateProfileReportResult::success())
    }
}
