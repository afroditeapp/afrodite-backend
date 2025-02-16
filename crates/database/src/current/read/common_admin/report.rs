use crate::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, AccountIdInternal, GetReportList, ReportContent, ReportDetailed, ReportDetailedInfo, ReportIdDb, ReportInternal, ReportProcessingState, ReportTypeNumber};

define_current_read_commands!(CurrentReadCommonAdminReport);

impl CurrentReadCommonAdminReport<'_> {
    fn get_internal_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportTypeNumber,
    ) -> Result<Option<ReportInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) =
            alias!(account_id as creator_aid, account_id as target_aid);

        let value: Option<(AccountId, AccountId, ReportIdDb, ReportProcessingState)> = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))))
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .filter(report_type_number.eq(report_type))
            .select((
                creator_aid.field(account_id::uuid),
                target_aid.field(account_id::uuid),
                id,
                processing_state,
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        let value = value.map(|(creator, target, report_id, state)| {
            ReportInternal {
                info: ReportDetailedInfo {
                    creator,
                    target,
                    processing_state: state,
                    report_type,
                },
                id: report_id,
            }
        });

        Ok(value)
    }

    pub fn get_detailed_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportTypeNumber,
    ) -> Result<Option<ReportDetailed>, DieselDatabaseError> {
        let internal = self.get_internal_report(creator, target, report_type)?;

        if let Some(internal) = internal {
            let detailed = self.convert_to_detailed_report(internal.info, internal.id)?;
            Ok(Some(detailed))
        } else {
            Ok(None)
        }
    }

    pub fn get_reports_page(
        &mut self,
    ) -> Result<GetReportList, DieselDatabaseError> {
        let reports = self.get_waiting_reports_page()?;

        let mut page = vec![];

        for r in reports {
            let detailed = self.convert_to_detailed_report(r.info, r.id)?;
            page.push(detailed);
        }

        Ok(GetReportList {
            values: page,
        })
    }

    fn convert_to_detailed_report(
        &mut self,
        info: ReportDetailedInfo,
        id: ReportIdDb,
    ) -> Result<ReportDetailed, DieselDatabaseError> {
        let detailed = ReportDetailed {
            content: ReportContent {
                profile_name: if info.report_type == ReportTypeNumber::ProfileName {
                    self.profile_name_report(id)?
                } else {
                    None
                },
                profile_text: if info.report_type == ReportTypeNumber::ProfileText {
                    self.profile_text_report(id)?
                } else {
                    None
                },
            },
            info,
        };

        Ok(detailed)
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

    fn profile_name_report(
        &mut self,
        id: ReportIdDb
    ) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_report_profile_name::dsl::*;

        profile_report_profile_name.find(id)
            .select(profile_name)
            .first(self.conn())
            .optional()
            .into_db_error(())
            .map(|v| v.flatten())
    }

    fn profile_text_report(
        &mut self,
        id: ReportIdDb
    ) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_report_profile_text::dsl::*;

        profile_report_profile_text.find(id)
            .select(profile_text)
            .first(self.conn())
            .optional()
            .into_db_error(())
            .map(|v| v.flatten())
    }
}
