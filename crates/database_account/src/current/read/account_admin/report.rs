use database::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, ReportProcessingState};
use model_account::{AccountReportContent, AccountReportDetailed, GetAccountReportList};

define_current_read_commands!(CurrentReadAccountReport);

impl CurrentReadAccountReport<'_> {
    pub fn get_report_list(
        &mut self,
    ) -> Result<GetAccountReportList, DieselDatabaseError> {
        use crate::schema::{account_id, account_report::dsl::*};

        let (creator_aid, target_aid) =
            alias!(account_id as creator_aid, account_id as target_aid);

        let values: Vec<(AccountId, AccountId, ReportProcessingState, AccountReportContent)> = account_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(creator_aid.field(account_id::id))))
            .filter(
                processing_state.eq(ReportProcessingState::Waiting)
            )
            .select((
                creator_aid.field(account_id::uuid),
                target_aid.field(account_id::uuid),
                processing_state,
                AccountReportContent::as_select(),
            ))
            .order((
                content_edit_unix_time.asc(),
                creator_account_id.asc(),
            ))
            .load(self.conn())
            .into_db_error(())?;

        let values = values.into_iter().map(|(creator, target, state, content)| {
            AccountReportDetailed {
                creator,
                target,
                processing_state: state,
                content,
            }
        }).collect();

        Ok(GetAccountReportList { values })
    }
}
