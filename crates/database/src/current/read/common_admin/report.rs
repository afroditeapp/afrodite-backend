use crate::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, ReportIdDb, ReportProcessingState, ReportTypeNumber, WaitingReport};

define_current_read_commands!(CurrentReadCommonAdminReport);

impl CurrentReadCommonAdminReport<'_> {
    pub fn get_waiting_reports_page(
        &mut self,
        report_type: ReportTypeNumber,
    ) -> Result<Vec<WaitingReport>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) =
            alias!(account_id as creator_aid, account_id as target_aid);

        const PAGE_SIZE: i64 = 25;

        let values: Vec<(AccountId, AccountId, ReportIdDb)> = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))))
            .filter(
                processing_state.eq(ReportProcessingState::Waiting)
            )
            .filter(
                report_type_number.eq(report_type)
            )
            .select((
                creator_aid.field(account_id::uuid),
                target_aid.field(account_id::uuid),
                id,
            ))
            .order((
                content_edit_unix_time.asc(),
                creator_account_id.asc(),
            ))
            .limit(PAGE_SIZE)
            .load(self.conn())
            .into_db_error(())?;

        let values = values.into_iter().map(|(creator, target, report_id)| {
            WaitingReport {
                creator,
                target,
                id: report_id,
            }
        }).collect();

        Ok(values)
    }
}
