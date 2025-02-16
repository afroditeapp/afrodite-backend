use database::current::read::GetDbReadCommandsCommon;
use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{AccountIdInternal, EventToClientInternal, ProfileNameModerationState, ProfileTextModerationState, ReportTypeNumber, UpdateReportResult};
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::DbTransaction,
    DataError,
};
use tracing::warn;

use crate::write::{profile_admin::profile_text::ModerateProfileTextMode, GetWriteCommandsProfile};

define_cmd_wrapper_write!(WriteCommandsProfileReport);

impl WriteCommandsProfileReport<'_> {
    pub async fn report_profile_name(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_name: String,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.profile().data().my_profile(target, None))
            .await?;

        if profile_name != target_data.p.name {
            return Ok(UpdateReportResult::outdated_report_content());
        }

        if target_data.name_moderation_state == ProfileNameModerationState::AcceptedByBot {
            // TODO(future): Profile name bot moderation
            warn!("Profile name bot moderations are unsupported currently");
        }

        let reports = self
            .db_read(move |mut cmds| cmds.common().report().get_all_detailed_reports(creator, target, ReportTypeNumber::ProfileName))
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let current_report = reports.iter().find(|v| v.report.content.profile_name.as_deref() == Some(&profile_name));
        if current_report.is_some() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.profile().report().insert_profile_name_report(creator, target, profile_name)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }

    pub async fn report_profile_text(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_text: String,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.profile().data().my_profile(target, None))
            .await?;

        if profile_text != target_data.p.ptext {
            return Ok(UpdateReportResult::outdated_report_content());
        }

        if target_data.text_moderation_info.state == ProfileTextModerationState::AcceptedByBot {
            self.handle().profile_admin().profile_text().moderate_profile_text(
                ModerateProfileTextMode::MoveToHumanModeration,
                target,
                profile_text.to_string(),
            ).await?;

            self.handle()
                .events()
                .send_connected_event(target, EventToClientInternal::ProfileChanged)
                .await?;
        }

        let reports = self
            .db_read(move |mut cmds| cmds.common().report().get_all_detailed_reports(creator, target, ReportTypeNumber::ProfileText))
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let current_report = reports.iter().find(|v| v.report.content.profile_text.as_deref() == Some(&profile_text));
        if current_report.is_some() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .report()
                .insert_profile_text_report(creator, target, profile_text)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }
}
