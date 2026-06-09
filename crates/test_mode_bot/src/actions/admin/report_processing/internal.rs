use std::collections::HashSet;

use api_client::{
    apis::common_admin_api::post_get_chat_message_reports,
    models::{AccountId, ChatMessageReport, GetChatMessageReports, ReportDetailed},
};
use base64::Engine;
use error_stack::{Result, ResultExt};
use test_mode_utils::client::{ApiClient, TestError};

use crate::actions::admin::report_processing::message::message_bytes_to_text;

pub struct MessageInternal {
    pub decoded_text: String,
    pub chat_message: ChatMessageReport,
    pub report: ReportDetailed,
}

pub enum ReportInternal {
    ProfileName(ReportDetailed),
    ProfileText(ReportDetailed),
    ProfileContent(ReportDetailed),
    Conversation { messages: Vec<MessageInternal> },
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ConversationKey {
    pub creator_aid: String,
    pub target_aid: String,
}

fn extract_chat_message(report: &ReportDetailed) -> Option<&ChatMessageReport> {
    Some(&**report.content.chat_message.as_ref()?.as_ref()?)
}

pub async fn convert_to_report_internal(
    api: &ApiClient,
    values: Vec<ReportDetailed>,
) -> Result<Vec<ReportInternal>, TestError> {
    let mut conversations: HashSet<ConversationKey> = HashSet::new();

    let mut result: Vec<ReportInternal> = Vec::new();

    for report in values {
        let n = report.info.report_type.n;
        match n {
            // ProfileName = 0
            0 => {
                result.push(ReportInternal::ProfileName(report));
            }
            // ProfileText = 1
            1 => {
                result.push(ReportInternal::ProfileText(report));
            }
            // ProfileContent = 2
            2 => {
                result.push(ReportInternal::ProfileContent(report));
            }
            // ChatMessage = 3
            3 => {
                let key = ConversationKey {
                    creator_aid: report.info.creator.aid.clone(),
                    target_aid: report.info.target.aid.clone(),
                };
                conversations.insert(key);
            }
            _ => {
                return Err(TestError::InvalidValue.report())
                    .attach_printable(format!("Unknown report type: {n}"));
            }
        }
    }

    for ConversationKey {
        creator_aid,
        target_aid,
    } in conversations
    {
        let chat_reports = post_get_chat_message_reports(
            &api.api(),
            GetChatMessageReports::new(
                AccountId::new(creator_aid.clone()),
                true,
                AccountId::new(target_aid.clone()),
            ),
        )
        .await
        .map_err(|_| TestError::ApiRequest)?;

        let mut messages = Vec::new();

        for report in chat_reports.values {
            if let Some(msg) = extract_chat_message(&report) {
                let decoded_bytes: Vec<u8> = Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &msg.message_base64,
                )
                .unwrap_or_default();
                let decoded_text = message_bytes_to_text(&decoded_bytes)
                    .unwrap_or("Invalid or unsupported message".to_string());
                messages.push(MessageInternal {
                    decoded_text,
                    chat_message: msg.clone(),
                    report,
                });
            } else {
                return Err(TestError::InvalidValue.report()).attach_printable(format!(
                    "Unknown report type {} from post_get_chat_message_reports API",
                    report.info.report_type.n
                ));
            }
        }

        messages.sort_by_key(|v| v.chat_message.message_number.mn);

        result.push(ReportInternal::Conversation { messages });
    }

    Ok(result)
}
