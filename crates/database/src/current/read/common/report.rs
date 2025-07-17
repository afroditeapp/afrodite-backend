use base64::Engine;
use config::file::Components;
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{
    AccountId, AccountIdDb, AccountIdInternal, AccountInteractionInternal, ChatMessageReport,
    ContentId, CustomReportContent, MessageId, ReportAccountInfo, ReportChatInfo,
    ReportChatInfoInteractionState, ReportContent, ReportDetailed, ReportDetailedInfo,
    ReportDetailedInfoInternal, ReportDetailedWithId, ReportIdDb, ReportInternal,
    ReportProcessingState, ReportTypeNumberInternal, UnixTime,
};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonReport);

impl CurrentReadCommonReport<'_> {
    fn get_all_internal_reports(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportTypeNumberInternal,
    ) -> Result<Vec<ReportInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, common_report::dsl::*};

        let (creator_aid, target_aid) = alias!(account_id as creator_aid, account_id as target_aid);

        let values: Vec<(
            AccountId,
            AccountIdDb,
            AccountId,
            AccountIdDb,
            ReportIdDb,
            ReportProcessingState,
        )> = common_report
            .inner_join(creator_aid.on(creator_account_id.eq(creator_aid.field(account_id::id))))
            .inner_join(target_aid.on(target_account_id.eq(target_aid.field(account_id::id))))
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .filter(report_type_number.eq(report_type.db_value()))
            .select((
                creator_aid.field(account_id::uuid),
                creator_account_id,
                target_aid.field(account_id::uuid),
                target_account_id,
                id,
                processing_state,
            ))
            .load(self.conn())
            .into_db_error(())?;

        let values = values
            .into_iter()
            .map(
                |(creator, creator_db_id, target, target_db_id, report_id, state)| ReportInternal {
                    info: ReportDetailedInfoInternal {
                        creator,
                        target,
                        processing_state: state,
                        report_type,
                    },
                    id: report_id,
                    creator_db_id,
                    target_db_id,
                },
            )
            .collect();

        Ok(values)
    }

    pub fn get_all_detailed_reports(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportTypeNumberInternal,
        components: Components,
    ) -> Result<Vec<ReportDetailedWithId>, DieselDatabaseError> {
        let internal = self.get_all_internal_reports(creator, target, report_type)?;

        let mut reports = vec![];
        for r in internal {
            let detailed = self.convert_to_detailed_report(r, components)?;
            reports.push(detailed);
        }

        Ok(reports)
    }

    pub fn convert_to_detailed_report(
        &mut self,
        report: ReportInternal,
        components: Components,
    ) -> Result<ReportDetailedWithId, DieselDatabaseError> {
        let mut profile_name = None;
        let mut profile_text = None;
        let mut profile_content = None;
        let mut chat_message = None;
        let mut custom_report = None;

        match report.info.report_type {
            ReportTypeNumberInternal::ProfileName => {
                profile_name = self.profile_name_report(report.id)?
            }
            ReportTypeNumberInternal::ProfileText => {
                profile_text = self.profile_text_report(report.id)?
            }
            ReportTypeNumberInternal::ProfileContent => {
                profile_content = self.profile_content_report(report.id)?
            }
            ReportTypeNumberInternal::ChatMessage => {
                chat_message = self.chat_message_report(report.id)?
            }
            ReportTypeNumberInternal::CustomReport(_) => {
                // Currently custom reports don't support any content
                custom_report = Some(CustomReportContent);
            }
        }

        let detailed = ReportDetailed {
            content: ReportContent {
                profile_name,
                profile_text,
                profile_content,
                chat_message,
                custom_report,
            },
            info: ReportDetailedInfo {
                id: report.id.into(),
                creator: report.info.creator,
                target: report.info.target,
                processing_state: report.info.processing_state,
                report_type: report.info.report_type.into(),
            },
            creator_info: if components.profile {
                self.get_report_account_info(report.creator_db_id)?
            } else {
                None
            },
            target_info: if components.profile {
                self.get_report_account_info(report.target_db_id)?
            } else {
                None
            },
            chat_info: if components.chat {
                self.get_report_chat_info(report.creator_db_id, report.target_db_id)?
            } else {
                None
            },
        };

        let detailed = ReportDetailedWithId {
            report: detailed,
            id: report.id,
        };

        Ok(detailed)
    }

    fn profile_name_report(
        &mut self,
        id: ReportIdDb,
    ) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_report_profile_name::dsl::*;

        profile_report_profile_name
            .find(id)
            .select(profile_name)
            .first(self.conn())
            .optional()
            .into_db_error(())
            .map(|v| v.flatten())
    }

    fn profile_text_report(
        &mut self,
        id: ReportIdDb,
    ) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_report_profile_text::dsl::*;

        profile_report_profile_text
            .find(id)
            .select(profile_text)
            .first(self.conn())
            .optional()
            .into_db_error(())
            .map(|v| v.flatten())
    }

    fn profile_content_report(
        &mut self,
        id: ReportIdDb,
    ) -> Result<Option<ContentId>, DieselDatabaseError> {
        use crate::schema::media_report_profile_content::dsl::*;

        media_report_profile_content
            .find(id)
            .select(profile_content_uuid)
            .first(self.conn())
            .optional()
            .into_db_error(())
            .map(|v| v.flatten())
    }

    fn chat_message_report(
        &mut self,
        id: ReportIdDb,
    ) -> Result<Option<ChatMessageReport>, DieselDatabaseError> {
        use crate::schema::chat_report_chat_message::dsl::*;

        let value: Option<(AccountId, AccountId, UnixTime, MessageId, Vec<u8>)> =
            chat_report_chat_message
                .find(id)
                .select((
                    message_sender_account_id_uuid,
                    message_receiver_account_id_uuid,
                    message_unix_time,
                    message_id,
                    client_message_bytes,
                ))
                .first(self.conn())
                .optional()
                .into_db_error(())?;

        if let Some((sender, receiver, message_time, m, data)) = value {
            Ok(Some(ChatMessageReport {
                sender,
                receiver,
                message_time,
                message_id: m,
                message_base64: base64::engine::general_purpose::STANDARD.encode(data),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_report_account_info(
        &mut self,
        id: AccountIdDb,
    ) -> Result<Option<ReportAccountInfo>, DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        let value = profile
            .find(id)
            .select((age, name))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        let info = value.map(|(age_value, name_value)| ReportAccountInfo {
            age: age_value,
            name: name_value,
        });

        Ok(info)
    }

    fn get_report_chat_info(
        &mut self,
        creator: AccountIdDb,
        target: AccountIdDb,
    ) -> Result<Option<ReportChatInfo>, DieselDatabaseError> {
        let interaction_id = {
            use crate::schema::account_interaction_index::dsl::*;

            account_interaction_index
                .find((creator, target))
                .select(interaction_id)
                .first(self.conn())
                .optional()
                .into_db_error(())?
        };

        let interaction_id: i64 = match interaction_id {
            Some(id_value) => id_value,
            None => return Ok(Some(ReportChatInfo::default())),
        };

        use crate::schema::account_interaction::dsl::*;

        let Some(interaction): Option<AccountInteractionInternal> = account_interaction
            .find(interaction_id)
            .select(AccountInteractionInternal::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?
        else {
            return Ok(Some(ReportChatInfo::default()));
        };

        Ok(Some(ReportChatInfo {
            state: if interaction.is_match() {
                ReportChatInfoInteractionState::Match
            } else if interaction.is_direction_liked(creator, target) {
                ReportChatInfoInteractionState::CreatorLiked
            } else if interaction.is_direction_liked(target, creator) {
                ReportChatInfoInteractionState::TargetLiked
            } else {
                ReportChatInfoInteractionState::Match
            },
            creator_blocked_target: interaction.is_direction_blocked(creator, target),
            target_blocked_creator: interaction.is_direction_blocked(target, creator),
            creator_sent_messages_count: interaction.message_count_for_account(creator),
            target_sent_messages_count: interaction.message_count_for_account(target),
        }))
    }
}
