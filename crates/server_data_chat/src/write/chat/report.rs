use database_chat::current::write::GetDbWriteCommandsChat;
use model::AccountIdInternal;
use model_chat::{ChatReportContent, UpdateChatReportResult};
use server_data::{
    define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError
};

use crate::read::GetReadChatCommands;

define_cmd_wrapper_write!(WriteCommandsChatReport);

impl WriteCommandsChatReport<'_> {
    /// The [ChatReportContent::is_against_video_calling] can be true only
    /// when users are a match.
    pub async fn update_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        reported_content: ChatReportContent,
    ) -> Result<UpdateChatReportResult, DataError> {
        if reported_content.is_against_video_calling {
            let interaction = self.handle().read().chat().account_interaction(creator, target).await?;
            let is_match = interaction.map(|v| v.is_match()).unwrap_or_default();
            if !is_match {
                return Ok(UpdateChatReportResult::not_match())
            }
        }

        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .report()
                .upsert_report(creator, target, reported_content)?;
            Ok(())
        })?;

        Ok(UpdateChatReportResult::success())
    }
}
