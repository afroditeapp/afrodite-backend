use crate::{current::read::GetDbReadCommandsCommon, define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, GetReportList, ReportDetailedInfo, ReportIdDb, ReportInternal, ReportProcessingState, ReportTypeNumber};

define_current_read_commands!(CurrentReadCommonAdminReport);

impl CurrentReadCommonAdminReport<'_> {
    pub fn get_reports_page(
        &mut self,
    ) -> Result<GetReportList, DieselDatabaseError> {
        let reports = self.get_waiting_reports_page()?;

        let mut page = vec![];

        for r in reports {
            let detailed = self.read().common().report().convert_to_detailed_report(r.info, r.id)?;
            page.push(detailed);
        }

        Ok(GetReportList {
            values: page.into_iter().map(|v| v.report).collect(),
        })
    }

    fn get_waiting_reports_page(
        &mut self,
    ) -> Result<Vec<ReportInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) =
            alias!(account_id as creator_aid, account_id as target_aid);

        const PAGE_SIZE: i64 = 25;

        let values: Vec<(AccountId, AccountId, ReportIdDb, ReportTypeNumber)> = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))))
            .filter(
                processing_state.eq(ReportProcessingState::Waiting)
            )
            .select((
                creator_aid.field(account_id::uuid),
                target_aid.field(account_id::uuid),
                id,
                report_type_number,
            ))
            .order((
                content_edit_unix_time.asc(),
                creator_account_id.asc(),
            ))
            .limit(PAGE_SIZE)
            .load(self.conn())
            .into_db_error(())?;

        let values = values.into_iter().map(|(creator, target, report_id, report_type)| {
            ReportInternal {
                info: ReportDetailedInfo {
                    creator,
                    target,
                    processing_state: ReportProcessingState::Waiting,
                    report_type,
                },
                id: report_id,
            }
        }).collect();

        Ok(values)
    }
}
