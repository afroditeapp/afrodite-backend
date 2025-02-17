
use database::current::read::GetDbReadCommandsCommon;
use model::{GetReportList, ReportIteratorQueryInternal};

use crate::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminReport);

impl ReadCommandsCommonAdminReport<'_> {
    pub async fn get_waiting_report_list(
        &self,
    ) -> Result<GetReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .report()
                .get_reports_page()
        })
        .await
        .into_error()
    }

    pub async fn get_report_iterator_page(
        &self,
        query: ReportIteratorQueryInternal,
    ) -> Result<GetReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .report()
                .get_report_iterator_page(query)
        })
        .await
        .into_error()
    }
}
