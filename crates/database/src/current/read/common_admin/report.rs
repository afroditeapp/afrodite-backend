use config::file::Components;
use diesel::{alias, prelude::*, sql_types::Bool};
use error_stack::Result;
use model::{
    AccountId, AccountIdDb, GetChatMessageReportsInternal, GetReportList,
    ReportDetailedInfoInternal, ReportIdDb, ReportInternal, ReportIteratorMode,
    ReportIteratorQueryInternal, ReportProcessingState, ReportTypeNumberInternal,
};

use crate::{
    DieselDatabaseError, IntoDatabaseError, current::read::GetDbReadCommandsCommon,
    define_current_read_commands,
};

define_current_read_commands!(CurrentReadCommonAdminReport);

impl CurrentReadCommonAdminReport<'_> {
    pub fn get_reports_page(
        &mut self,
        components: Components,
    ) -> Result<GetReportList, DieselDatabaseError> {
        let reports = self.get_waiting_reports_page()?;

        let mut page = vec![];

        for r in reports {
            let detailed = self
                .read()
                .common()
                .report()
                .convert_to_detailed_report(r, components)?;
            page.push(detailed);
        }

        Ok(GetReportList {
            values: page.into_iter().map(|v| v.report).collect(),
        })
    }

    fn get_waiting_reports_page(&mut self) -> Result<Vec<ReportInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) = alias!(account_id as creator_aid, account_id as target_aid);

        const PAGE_SIZE: i64 = 25;

        let values: Vec<(
            AccountId,
            AccountIdDb,
            AccountId,
            AccountIdDb,
            ReportIdDb,
            ReportTypeNumberInternal,
        )> = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))))
            .filter(processing_state.eq(ReportProcessingState::Waiting))
            .select((
                creator_aid.field(account_id::uuid),
                creator_account_id,
                target_aid.field(account_id::uuid),
                target_account_id,
                id,
                report_type_number,
            ))
            .order((creation_unix_time.asc(), creator_account_id.asc()))
            .limit(PAGE_SIZE)
            .load(self.conn())
            .into_db_error(())?;

        let values = values
            .into_iter()
            .map(
                |(creator, creator_db_id, target, target_db_id, report_id, report_type)| {
                    ReportInternal {
                        info: ReportDetailedInfoInternal {
                            creator,
                            target,
                            processing_state: ReportProcessingState::Waiting,
                            report_type,
                        },
                        id: report_id,
                        creator_db_id,
                        target_db_id,
                    }
                },
            )
            .collect();

        Ok(values)
    }

    pub fn get_report_iterator_page(
        &mut self,
        query: ReportIteratorQueryInternal,
        components: Components,
    ) -> Result<GetReportList, DieselDatabaseError> {
        let reports = self.get_report_iterator_page_internal(query)?;

        let mut page = vec![];

        for r in reports {
            let detailed = self
                .read()
                .common()
                .report()
                .convert_to_detailed_report(r, components)?;
            page.push(detailed);
        }

        Ok(GetReportList {
            values: page.into_iter().map(|v| v.report).collect(),
        })
    }

    fn get_report_iterator_page_internal(
        &mut self,
        query: ReportIteratorQueryInternal,
    ) -> Result<Vec<ReportInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) = alias!(account_id as creator_aid, account_id as target_aid);

        const PAGE_SIZE: i64 = 25;

        let db_query = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))));

        let values: Vec<(
            AccountId,
            AccountIdDb,
            AccountId,
            AccountIdDb,
            ReportIdDb,
            ReportProcessingState,
            ReportTypeNumberInternal,
        )> = match query.mode {
            ReportIteratorMode::Received => db_query
                .filter(target_account_id.eq(query.aid.as_db_id()))
                .filter(creation_unix_time.le(query.start_position))
                .select((
                    creator_aid.field(account_id::uuid),
                    creator_account_id,
                    target_aid.field(account_id::uuid),
                    target_account_id,
                    id,
                    processing_state,
                    report_type_number,
                ))
                .order((creation_unix_time.desc(), creator_account_id.desc()))
                .limit(PAGE_SIZE)
                .offset(PAGE_SIZE.saturating_mul(query.page))
                .load(self.conn())
                .into_db_error(())?,
            ReportIteratorMode::Sent => db_query
                .filter(creator_account_id.eq(query.aid.as_db_id()))
                .filter(creation_unix_time.le(query.start_position))
                .select((
                    creator_aid.field(account_id::uuid),
                    creator_account_id,
                    target_aid.field(account_id::uuid),
                    target_account_id,
                    id,
                    processing_state,
                    report_type_number,
                ))
                .order((creation_unix_time.desc(), creator_account_id.desc()))
                .limit(PAGE_SIZE)
                .offset(PAGE_SIZE.saturating_mul(query.page))
                .load(self.conn())
                .into_db_error(())?,
        };

        let values = values
            .into_iter()
            .map(
                |(
                    creator,
                    creator_db_id,
                    target,
                    target_db_id,
                    report_id,
                    report_state,
                    report_type,
                )| {
                    ReportInternal {
                        info: ReportDetailedInfoInternal {
                            creator,
                            target,
                            processing_state: report_state,
                            report_type,
                        },
                        id: report_id,
                        creator_db_id,
                        target_db_id,
                    }
                },
            )
            .collect();

        Ok(values)
    }

    pub fn get_chat_message_reports(
        &mut self,
        query: GetChatMessageReportsInternal,
        components: Components,
    ) -> Result<GetReportList, DieselDatabaseError> {
        let reports = self.get_chat_message_reports_internal(query)?;

        let mut page = vec![];

        for r in reports {
            let detailed = self
                .read()
                .common()
                .report()
                .convert_to_detailed_report(r, components)?;
            page.push(detailed);
        }

        page.sort_by_key(|v| {
            match &v.report.content.chat_message {
                Some(v) => v.message_id.id,
                None => -1, // Should not happen
            }
        });

        Ok(GetReportList {
            values: page.into_iter().map(|v| v.report).collect(),
        })
    }

    fn get_chat_message_reports_internal(
        &mut self,
        query: GetChatMessageReportsInternal,
    ) -> Result<Vec<ReportInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) = alias!(account_id as creator_aid, account_id as target_aid);

        let db_query = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))));

        let values: Vec<(
            AccountId,
            AccountIdDb,
            AccountId,
            AccountIdDb,
            ReportIdDb,
            ReportProcessingState,
            ReportTypeNumberInternal,
        )> = {
            db_query
                .filter(creator_account_id.eq(query.creator.as_db_id()))
                .filter(target_account_id.eq(query.target.as_db_id()))
                .filter(report_type_number.eq(ReportTypeNumberInternal::ChatMessage))
                .filter(
                    (!query.only_not_processed)
                        .as_sql::<Bool>()
                        .or(processing_state
                            .eq(ReportProcessingState::Waiting)
                            .and(query.only_not_processed.as_sql::<Bool>())),
                )
                .select((
                    creator_aid.field(account_id::uuid),
                    creator_account_id,
                    target_aid.field(account_id::uuid),
                    target_account_id,
                    id,
                    processing_state,
                    report_type_number,
                ))
                .order((creation_unix_time.desc(), creator_account_id.desc()))
                .load(self.conn())
                .into_db_error(())?
        };

        let values = values
            .into_iter()
            .map(
                |(
                    creator,
                    creator_db_id,
                    target,
                    target_db_id,
                    report_id,
                    report_state,
                    report_type,
                )| {
                    ReportInternal {
                        info: ReportDetailedInfoInternal {
                            creator,
                            target,
                            processing_state: report_state,
                            report_type,
                        },
                        id: report_id,
                        creator_db_id,
                        target_db_id,
                    }
                },
            )
            .collect();

        Ok(values)
    }
}
