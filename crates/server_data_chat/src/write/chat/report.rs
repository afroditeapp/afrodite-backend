use database::current::read::GetDbReadCommandsCommon;
use database_chat::current::write::GetDbWriteCommandsChat;
use model::{AccountIdInternal, ReportTypeNumber, ReportTypeNumberInternal, UpdateReportResult};
use model_chat::NewChatMessageReportInternal;
use server_data::{
    DataError,
    app::GetConfig,
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

use crate::read::GetReadChatCommands;

define_cmd_wrapper_write!(WriteCommandsChatReport);

impl WriteCommandsChatReport<'_> {
    /// The users must be a match.
    pub async fn report_chat_message(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        message: NewChatMessageReportInternal,
    ) -> Result<UpdateReportResult, DataError> {
        let interaction = self
            .handle()
            .read()
            .chat()
            .account_interaction(creator, target)
            .await?;
        let is_match = interaction.map(|v| v.is_match()).unwrap_or_default();
        if !is_match {
            return Err(DataError::NotAllowed.report());
        }

        let components = self.config().components();
        let reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    ReportTypeNumberInternal::ChatMessage,
                    components,
                )
            })
            .await?;

        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let current_report = reports.iter().find(|v| {
            v.report.content.chat_message.as_ref() == Some(&message.to_chat_message_report())
        });
        if current_report.is_some() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .report()
                .insert_chat_message_report(creator, target, message)?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }
}
