use database::current::read::GetDbReadCommandsCommon;
use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model::{
    ContentId, ContentIdInternal, ReportTypeNumber, ReportTypeNumberInternal, UpdateReportResult,
};
use model_media::{AccountIdInternal, ContentModerationState, EventToClientInternal};
use server_data::{
    DataError, app::GetConfig, db_transaction, define_cmd_wrapper_write, read::DbRead,
    result::Result, write::DbTransaction,
};

use crate::write::{GetWriteCommandsMedia, media_admin::content::ContentModerationMode};

define_cmd_wrapper_write!(WriteCommandsMediaReport);

impl WriteCommandsMediaReport<'_> {
    pub async fn update_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: ContentId,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.media().media_content().current_account_media(target))
            .await?;

        let mut send_event = false;

        let profile_content = target_data
            .iter_current_profile_content()
            .find(|v| v.uuid == content);
        if let Some(profile_content) = profile_content {
            if profile_content.state() == ContentModerationState::AcceptedByBot {
                let content_id_internal =
                    ContentIdInternal::new(target, profile_content.uuid, profile_content.id);
                self.handle()
                    .media_admin()
                    .content()
                    .moderate_media_content(
                        ContentModerationMode::MoveToHumanModeration,
                        content_id_internal,
                    )
                    .await?;
                send_event = true;
            }
        } else {
            return Ok(UpdateReportResult::outdated_report_content());
        }

        if send_event {
            self.handle()
                .events()
                .send_connected_event(target, EventToClientInternal::MediaContentChanged)
                .await?;
        }

        let components = self.config().components();
        let reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    ReportTypeNumberInternal::ProfileContent,
                    components,
                )
            })
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let current_report = reports
            .iter()
            .find(|v| v.report.content.profile_content == Some(content));
        if current_report.is_some() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .report()
                .insert_profile_content_report(creator, target, content)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }
}
