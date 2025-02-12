use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model::{ContentIdInternal, UpdateReportResult};
use model_media::{AccountIdInternal, ContentModerationState, EventToClientInternal, MediaReportContent};
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::DbTransaction,
    DataError,
};

use crate::write::{media_admin::content::ContentModerationMode, GetWriteCommandsMedia};

define_cmd_wrapper_write!(WriteCommandsMediaReport);

impl WriteCommandsMediaReport<'_> {
    pub async fn update_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: MediaReportContent,
    ) -> Result<UpdateReportResult, DataError> {
        let target_data = self
            .db_read(move |mut cmds| cmds.media().media_content().current_account_media(target))
            .await?;

        let mut send_event = false;
        if !content.profile_content.is_empty() {
            for c in &content.profile_content {
                let profile_content = target_data.iter_current_profile_content().find(|v| v.uuid == *c);
                if let Some(profile_content) = profile_content {
                    if profile_content.state() == ContentModerationState::AcceptedByBot {
                        let content_id_internal = ContentIdInternal::new(target, profile_content.uuid, profile_content.id);
                        self.handle().media_admin().content().moderate_profile_content(
                            ContentModerationMode::MoveToHumanModeration,
                            content_id_internal,
                        ).await?;
                        send_event = true;
                    }
                } else {
                    return Ok(UpdateReportResult::outdated_report_content())
                }
            }
        }

        if send_event {
            self.handle()
                .events()
                .send_connected_event(
                    target,
                    EventToClientInternal::MediaContentChanged,
                )
                .await?;
        }

        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .report()
                .upsert_report(creator, target, content)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }
}
