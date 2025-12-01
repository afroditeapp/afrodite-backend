use database::current::read::GetDbReadCommandsCommon;
use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{
    AccountIdInternal, EventToClientInternal, ProfileStringModerationContentType,
    ProfileStringModerationState, ReportTypeNumber, ReportTypeNumberInternal, UpdateReportResult,
};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, read::DbRead, result::Result,
    write::DbTransaction,
};
use simple_backend_model::NonEmptyString;

use crate::write::{GetWriteCommandsProfile, profile_admin::moderation::ModerateProfileValueMode};

define_cmd_wrapper_write!(WriteCommandsProfileReport);

impl WriteCommandsProfileReport<'_> {
    pub async fn report_profile_name(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_name: NonEmptyString,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.profile().data().my_profile(target))
            .await?;

        if Some(&profile_name) != target_data.p.name.as_ref() {
            return Ok(UpdateReportResult::outdated_report_content());
        }

        if target_data.name_moderation_info.as_ref().map(|v| v.state)
            == Some(ProfileStringModerationState::AcceptedByBot)
        {
            self.handle()
                .profile_admin()
                .moderation()
                .moderate_profile_string(
                    ProfileStringModerationContentType::ProfileName,
                    ModerateProfileValueMode::MoveToHumanModeration,
                    target,
                    profile_name.clone(),
                )
                .await?;

            self.handle()
                .events()
                .send_connected_event(target, EventToClientInternal::ProfileChanged)
                .await?;
        }

        let reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    ReportTypeNumberInternal::ProfileName,
                )
            })
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let current_report = reports
            .iter()
            .find(|v| v.report.content.profile_name.as_ref() == Some(&profile_name));
        if current_report.is_some() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .report()
                .insert_profile_name_report(creator, target, profile_name)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }

    pub async fn report_profile_text(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        profile_text: NonEmptyString,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.profile().data().my_profile(target))
            .await?;

        if Some(&profile_text) != target_data.p.ptext.as_ref() {
            return Ok(UpdateReportResult::outdated_report_content());
        }

        if target_data.text_moderation_info.as_ref().map(|v| v.state)
            == Some(ProfileStringModerationState::AcceptedByBot)
        {
            self.handle()
                .profile_admin()
                .moderation()
                .moderate_profile_string(
                    ProfileStringModerationContentType::ProfileText,
                    ModerateProfileValueMode::MoveToHumanModeration,
                    target,
                    profile_text.clone(),
                )
                .await?;

            self.handle()
                .events()
                .send_connected_event(target, EventToClientInternal::ProfileChanged)
                .await?;
        }

        let reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    ReportTypeNumberInternal::ProfileText,
                )
            })
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let current_report = reports
            .iter()
            .find(|v| v.report.content.profile_text.as_ref() == Some(&profile_text));
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
