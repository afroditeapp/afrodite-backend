use database::current::read::GetDbReadCommandsCommon;
use model::{
    GetChatMessageReportsInternal, GetReportList, ReportIteratorQueryInternal, ReportQueueType,
    ReportType, ReportTypeInternal,
};
use simple_backend_utils::IntoReportFromString;

use crate::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonAdminReport);

impl ReadCommandsCommonAdminReport<'_> {
    /// Empty report type list disables report type filtering.
    pub async fn get_reports_page(
        &self,
        wanted_report_types: &[ReportType],
        queue_type: ReportQueueType,
    ) -> Result<GetReportList, DataError> {
        let mut wanted_report_types_internal = Vec::with_capacity(wanted_report_types.len());
        for report_type in wanted_report_types {
            let report_type_internal =
                TryInto::<ReportTypeInternal>::try_into(Into::<i16>::into(report_type.n))
                    .into_error_string(DataError::NotAllowed)?;
            wanted_report_types_internal.push(report_type_internal);
        }

        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .report()
                .get_reports_page(wanted_report_types_internal, queue_type)
        })
        .await
        .into_error()
    }

    pub async fn get_report_iterator_page(
        &self,
        query: ReportIteratorQueryInternal,
    ) -> Result<GetReportList, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().report().get_report_iterator_page(query))
            .await
            .into_error()
    }

    pub async fn get_chat_message_reports(
        &self,
        query: GetChatMessageReportsInternal,
    ) -> Result<GetReportList, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().report().get_chat_message_reports(query))
            .await
            .into_error()
    }
}
