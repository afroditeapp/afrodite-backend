use database::{current::read::GetDbReadCommandsCommon, define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ReportDetailedInfo, ReportTypeNumber};
use model_profile::{GetProfileNameReportList, GetProfileTextReportList, ProfileNameReportDetailed, ProfileTextReportDetailed};

define_current_read_commands!(CurrentReadProfileReport);

impl CurrentReadProfileReport<'_> {
    pub fn get_current_profile_name_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_report_profile_name::dsl::*;

        let Some(id) = self.read()
            .common()
            .report()
            .get_report_id(creator, target, ReportTypeNumber::ProfileName)? else {
                return Ok(None);
            };

        let value = profile_report_profile_name
            .filter(report_id.eq(id))
            .select(profile_name)
            .first::<Option<String>>(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        Ok(value.flatten())
    }

    pub fn get_current_profile_text_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_report_profile_text::dsl::*;

        let Some(id) = self.read()
            .common()
            .report()
            .get_report_id(creator, target, ReportTypeNumber::ProfileText)? else {
                return Ok(None);
            };

        let value = profile_report_profile_text
            .filter(report_id.eq(id))
            .select(profile_text)
            .first::<Option<String>>(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        Ok(value.flatten())
    }

    pub fn get_profile_name_report_list(
        &mut self,
    ) -> Result<GetProfileNameReportList, DieselDatabaseError> {
        let waiting_reports = self.read()
            .common_admin()
            .report()
            .get_waiting_reports_page(ReportTypeNumber::ProfileName)?;

        let mut values = vec![];

        for r in waiting_reports {
            use crate::schema::profile_report_profile_name::dsl::*;

            let name = profile_report_profile_name
                .find(r.id)
                .select(profile_name)
                .first(self.conn())
                .into_db_error(())?;

            values.push(ProfileNameReportDetailed {
                info: ReportDetailedInfo {
                    creator: r.creator,
                    target: r.target,
                    processing_state: r.state(),
                },
                profile_name: name,
            });
        }

        Ok(GetProfileNameReportList { values })
    }

    pub fn get_profile_text_report_list(
        &mut self,
    ) -> Result<GetProfileTextReportList, DieselDatabaseError> {
        let waiting_reports = self.read()
            .common_admin()
            .report()
            .get_waiting_reports_page(ReportTypeNumber::ProfileText)?;

        let mut values = vec![];

        for r in waiting_reports {
            use crate::schema::profile_report_profile_text::dsl::*;

            let text = profile_report_profile_text
                .find(r.id)
                .select(profile_text)
                .first(self.conn())
                .into_db_error(())?;

            values.push(ProfileTextReportDetailed {
                info: ReportDetailedInfo {
                    creator: r.creator,
                    target: r.target,
                    processing_state: r.state(),
                },
                profile_text: text,
            });
        }

        Ok(GetProfileTextReportList { values })
    }
}
