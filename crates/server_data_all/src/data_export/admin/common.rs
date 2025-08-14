use config::Config;
use database::{DbReadMode, DieselDatabaseError, current::read::GetDbReadCommandsCommon};
use model::{AccountIdInternal, ReportDetailed, ReportIteratorQueryInternal, UnixTime};
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct AdminDataExportJsonCommon {
    sent_reports: Vec<ReportDetailed>,
}

impl AdminDataExportJsonCommon {
    pub fn query(
        config: &Config,
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let data = Self {
            sent_reports: DataExportReport::query(config, current, id)?,
        };
        Ok(data)
    }
}

#[derive(Serialize)]
struct DataExportReport;

impl DataExportReport {
    fn query(
        config: &Config,
        current: &mut DbReadMode,
        id: AccountIdInternal,
    ) -> error_stack::Result<Vec<ReportDetailed>, DieselDatabaseError> {
        let mut data_export_reports = vec![];
        let mut query = ReportIteratorQueryInternal {
            start_position: UnixTime::default(),
            page: 0,
            aid: id,
            mode: model::ReportIteratorMode::Sent,
        };
        loop {
            let reports = current
                .common_admin()
                .report()
                .get_report_iterator_page(query.clone(), config.components())?;
            if reports.values.is_empty() {
                break;
            }
            for r in reports.values {
                data_export_reports.push(r);
            }
            query.page += 1;
        }
        Ok(data_export_reports)
    }
}
