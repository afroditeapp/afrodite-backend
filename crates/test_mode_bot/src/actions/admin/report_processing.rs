use std::sync::Arc;

use api_client::{
    apis::{
        account_admin_api::post_set_ban_state, account_api::get_account_ban_time,
        common_admin_api::post_get_report_queue_page,
    },
    models::{
        AccountId, GetReportQueuePage, ProcessReport, ProcessReports, ReportQueueType, ReportType,
        SetAccountBanState, UnixTime,
    },
};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest, ImageUrl,
    },
};
use base64::display::Base64Display;
use config::bot_config_file::{
    ReasoningEffortConfig,
    internal::{
        AcceptOrReject, AutomaticBanningDayCountConfig, AutomaticBanningExpectedLlmResponsesConfig,
        ReportProcessingConfigInternal, ReportProcessingMessagesConfigInternal,
        ReportProcessingProfileContentConfigInternal, ReportProcessingProfileStringConfigInternal,
    },
};
use error_stack::ResultExt;
use futures::{StreamExt, stream};
use internal::{MessageInternal, ReportInternal};
use simple_backend_utils::Result;
use test_mode_utils::client::{ApiClient, TestError};
use tracing::info;

use super::EmptyPage;

pub mod internal;
pub mod message;

#[derive(Clone, Copy)]
enum ReportSeverity {
    Low,
    Medium,
    High,
}

impl ReportSeverity {
    fn from_llm_response(
        response_first_line: &str,
        expected_responses: &AutomaticBanningExpectedLlmResponsesConfig,
    ) -> Option<Self> {
        let line = response_first_line.trim();
        if line.contains(&expected_responses.high.to_lowercase()) {
            Some(Self::High)
        } else if line.contains(&expected_responses.medium.to_lowercase()) {
            Some(Self::Medium)
        } else if line.contains(&expected_responses.low.to_lowercase()) {
            Some(Self::Low)
        } else {
            None
        }
    }

    fn ban_days(&self, day_counts: &AutomaticBanningDayCountConfig) -> u32 {
        match self {
            Self::Low => day_counts.low,
            Self::Medium => day_counts.medium,
            Self::High => day_counts.high,
        }
    }
}

struct BanAction {
    target: AccountId,
    ban_until: UnixTime,
    reason_category: i32,
}

struct BanConfig {
    enabled: bool,
    day_counts: AutomaticBanningDayCountConfig,
}

impl BanConfig {
    fn from_profile_string(c: &ReportProcessingProfileStringConfigInternal) -> Self {
        Self {
            enabled: c.automatic_banning_enabled,
            day_counts: c.automatic_banning_day_counts.clone(),
        }
    }

    fn from_profile_content(c: &ReportProcessingProfileContentConfigInternal) -> Self {
        Self {
            enabled: c.automatic_banning_enabled,
            day_counts: c.automatic_banning_day_counts.clone(),
        }
    }

    fn from_messages(c: &ReportProcessingMessagesConfigInternal) -> Self {
        Self {
            enabled: c.automatic_banning_enabled,
            day_counts: c.automatic_banning_day_counts.clone(),
        }
    }
}

struct ProcessedReportWithBans {
    processed: Vec<ProcessReport>,
    ban: Option<BanAction>,
}

#[derive(Clone)]
struct ProfileStringLlmConfigAndClient {
    config: Arc<ReportProcessingProfileStringConfigInternal>,
    client: Client<OpenAIConfig>,
}

#[derive(Clone)]
struct ProfileContentLlmConfigAndClient {
    config: Arc<ReportProcessingProfileContentConfigInternal>,
    client: Client<OpenAIConfig>,
}

#[derive(Clone)]
struct MessagesLlmConfigAndClient {
    config: Arc<ReportProcessingMessagesConfigInternal>,
    client: Client<OpenAIConfig>,
}

pub struct ReportProcessingState {
    profile_name_llm: Option<ProfileStringLlmConfigAndClient>,
    profile_text_llm: Option<ProfileStringLlmConfigAndClient>,
    profile_content_llm: Option<ProfileContentLlmConfigAndClient>,
    messages_llm: Option<MessagesLlmConfigAndClient>,
}

impl ReportProcessingState {
    pub fn new(config: &ReportProcessingConfigInternal, reqwest_client: reqwest::Client) -> Self {
        fn make_profile_string_client(
            config: &Option<ReportProcessingProfileStringConfigInternal>,
            client: &reqwest::Client,
        ) -> Option<ProfileStringLlmConfigAndClient> {
            let c = config.as_ref()?;
            Some(ProfileStringLlmConfigAndClient {
                client: Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base(c.llm.openai_api_url.to_string())
                        .with_api_key(""),
                )
                .with_http_client(client.clone()),
                config: Arc::new(c.clone()),
            })
        }

        fn make_profile_content_client(
            config: &Option<ReportProcessingProfileContentConfigInternal>,
            client: &reqwest::Client,
        ) -> Option<ProfileContentLlmConfigAndClient> {
            let c = config.as_ref()?;
            Some(ProfileContentLlmConfigAndClient {
                client: Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base(c.llm.openai_api_url.to_string())
                        .with_api_key(""),
                )
                .with_http_client(client.clone()),
                config: Arc::new(c.clone()),
            })
        }

        fn make_messages_client(
            config: &Option<ReportProcessingMessagesConfigInternal>,
            client: &reqwest::Client,
        ) -> Option<MessagesLlmConfigAndClient> {
            let c = config.as_ref()?;
            Some(MessagesLlmConfigAndClient {
                client: Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base(c.llm.openai_api_url.to_string())
                        .with_api_key(""),
                )
                .with_http_client(client.clone()),
                config: Arc::new(c.clone()),
            })
        }

        Self {
            profile_name_llm: make_profile_string_client(&config.profile_name, &reqwest_client),
            profile_text_llm: make_profile_string_client(&config.profile_text, &reqwest_client),
            profile_content_llm: make_profile_content_client(
                &config.profile_content,
                &reqwest_client,
            ),
            messages_llm: make_messages_client(&config.messages, &reqwest_client),
        }
    }
}

pub struct AdminBotReportProcessingLogic;

impl AdminBotReportProcessingLogic {
    pub async fn run_report_processing(
        api: &ApiClient,
        config: &ReportProcessingConfigInternal,
        state: &mut ReportProcessingState,
    ) -> Result<(), TestError> {
        loop {
            if let Some(EmptyPage) = Self::process_one_page(api, config, state).await? {
                break;
            }
        }

        Ok(())
    }

    async fn process_one_page(
        api: &ApiClient,
        config: &ReportProcessingConfigInternal,
        state: &mut ReportProcessingState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let mut wanted_report_types = Vec::new();

        if config.profile_name.is_some() {
            wanted_report_types.push(ReportType::new(0)); // ProfileName
        }
        if config.profile_text.is_some() {
            wanted_report_types.push(ReportType::new(1)); // ProfileText
        }
        if config.profile_content.is_some() {
            wanted_report_types.push(ReportType::new(2)); // ProfileContent
        }
        if config.messages.is_some() {
            wanted_report_types.push(ReportType::new(3)); // ChatMessage
        }

        if wanted_report_types.is_empty() {
            return Ok(Some(EmptyPage));
        }

        let queue = GetReportQueuePage::new(ReportQueueType::Waiting, wanted_report_types);

        let list = post_get_report_queue_page(&api.api(), queue)
            .await
            .map_err(|_| TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        // Convert API reports into internal representation, grouping
        // chat messages per conversation (fetches additional messages
        // via post_get_chat_message_reports).
        let reports = internal::convert_to_report_internal(api, list.values).await?;

        let mut stream = stream::iter(reports)
            .map(|report| {
                let api = api.clone();
                let config = config.clone();
                let profile_name_llm = state.profile_name_llm.clone();
                let profile_text_llm = state.profile_text_llm.clone();
                let profile_content_llm = state.profile_content_llm.clone();
                let messages_llm = state.messages_llm.clone();
                async move {
                    Self::process_one_report(
                        &api,
                        &config,
                        &report,
                        &profile_name_llm,
                        &profile_text_llm,
                        &profile_content_llm,
                        &messages_llm,
                    )
                    .await
                }
            })
            .buffer_unordered(config.concurrency.into());

        let mut processed: Vec<ProcessReport> = Vec::new();
        let mut bans: Vec<BanAction> = Vec::new();

        loop {
            match stream.next().await {
                Some(Ok(result)) => {
                    processed.extend(result.processed);
                    if let Some(ban) = result.ban {
                        bans.push(ban);
                    }
                }
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        if !processed.is_empty() {
            api_client::apis::common_admin_api::post_process_reports(
                &api.api(),
                ProcessReports::new(processed),
            )
            .await
            .map_err(|_| TestError::ApiRequest)?;
        }

        Self::execute_bans(api, bans).await?;

        Ok(None)
    }

    async fn process_one_report(
        api: &ApiClient,
        config: &ReportProcessingConfigInternal,
        report: &ReportInternal,
        profile_name_llm: &Option<ProfileStringLlmConfigAndClient>,
        profile_text_llm: &Option<ProfileStringLlmConfigAndClient>,
        profile_content_llm: &Option<ProfileContentLlmConfigAndClient>,
        messages_llm: &Option<MessagesLlmConfigAndClient>,
    ) -> Result<ProcessedReportWithBans, TestError> {
        let report_target = match report {
            ReportInternal::ProfileName(r)
            | ReportInternal::ProfileText(r)
            | ReportInternal::ProfileContent(r) => Some(r.info.target.clone()),
            ReportInternal::Conversation { messages } => {
                messages.first().map(|m| m.report.info.target.clone())
            }
        };

        let (accepted, severity, per_type_cfg): (bool, Option<ReportSeverity>, Option<BanConfig>) =
            match report {
                ReportInternal::ProfileName(r) => {
                    let cfg = config.profile_name.as_ref();
                    if let Some(text) = r.content.profile_name.as_ref()
                        && let Some(llm) = profile_name_llm
                    {
                        let (a, s) = Self::llm_profile_string_decision(text, llm).await?;
                        (a, s, cfg.map(BanConfig::from_profile_string))
                    } else {
                        Self::default_decision_from_opt(cfg.map(|v| v.default_action))
                    }
                }
                ReportInternal::ProfileText(r) => {
                    let cfg = config.profile_text.as_ref();
                    if let Some(text) = r.content.profile_text.as_ref()
                        && let Some(llm) = profile_text_llm
                    {
                        let (a, s) = Self::llm_profile_string_decision(text, llm).await?;
                        (a, s, cfg.map(BanConfig::from_profile_string))
                    } else {
                        Self::default_decision_from_opt(cfg.map(|v| v.default_action))
                    }
                }
                ReportInternal::ProfileContent(r) => {
                    let cfg = config.profile_content.as_ref();
                    if let Some(content_id) = r.content.profile_content.as_ref()
                        && let Some(llm) = profile_content_llm
                    {
                        let (a, s) = Self::llm_profile_content_decision(
                            api,
                            content_id,
                            &r.info.target,
                            llm,
                        )
                        .await?;
                        (a, s, cfg.map(BanConfig::from_profile_content))
                    } else {
                        Self::default_decision_from_opt(cfg.map(|v| v.default_action))
                    }
                }
                ReportInternal::Conversation { messages } => {
                    let cfg = config.messages.as_ref();
                    if !messages.is_empty()
                        && let Some(llm) = messages_llm
                    {
                        let (a, s) = Self::llm_conversation_decision(messages, llm).await?;
                        (a, s, cfg.map(BanConfig::from_messages))
                    } else {
                        Self::default_decision_from_opt(cfg.map(|v| v.default_action))
                    }
                }
            };

        // Build ProcessReports from originals without consuming
        let processed: Vec<ProcessReport> = match report {
            ReportInternal::ProfileName(r)
            | ReportInternal::ProfileText(r)
            | ReportInternal::ProfileContent(r) => {
                vec![ProcessReport::new(
                    accepted,
                    r.content.clone(),
                    r.info.creator.clone(),
                    r.info.report_type.clone(),
                    r.info.target.clone(),
                )]
            }
            ReportInternal::Conversation { messages } => messages
                .iter()
                .map(|msg| {
                    ProcessReport::new(
                        accepted,
                        msg.report.content.clone(),
                        msg.report.info.creator.clone(),
                        msg.report.info.report_type.clone(),
                        msg.report.info.target.clone(),
                    )
                })
                .collect(),
        };

        let ban = if accepted
            && let Some(severity) = severity
            && let Some(cfg) = per_type_cfg
            && cfg.enabled
            && let Some(target) = report_target
        {
            let reason_category = match report {
                ReportInternal::ProfileName(_) => 0,
                ReportInternal::ProfileText(_) => 1,
                ReportInternal::ProfileContent(_) => 2,
                ReportInternal::Conversation { .. } => 3,
            };

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let ban_until =
                UnixTime::new(now + (severity.ban_days(&cfg.day_counts) as i64) * 86400);
            Some(BanAction {
                target,
                ban_until,
                reason_category,
            })
        } else {
            None
        };

        Ok(ProcessedReportWithBans { processed, ban })
    }

    async fn execute_bans(api: &ApiClient, bans: Vec<BanAction>) -> Result<(), TestError> {
        let mut longest_bans: std::collections::HashMap<String, BanAction> =
            std::collections::HashMap::new();
        for ban in bans {
            let key = ban.target.aid.clone();
            if let Some(existing) = longest_bans.get(&key) {
                if ban.ban_until.ut > existing.ban_until.ut {
                    longest_bans.insert(key, ban);
                }
            } else {
                longest_bans.insert(key, ban);
            }
        }

        for ban in longest_bans.into_values() {
            let current_ban = get_account_ban_time(&api.api(), &ban.target.aid)
                .await
                .map_err(|_| TestError::ApiRequest)?;

            let should_ban = current_ban
                .banned_until
                .is_none_or(|existing| ban.ban_until.ut > existing.ut);

            if should_ban {
                post_set_ban_state(
                    &api.api(),
                    SetAccountBanState {
                        account: ban.target.clone(),
                        ban_until: Some(ban.ban_until.clone()),
                        reason_category: Some(api_client::models::AccountBanReasonCategory::new(
                            ban.reason_category,
                        )),
                        reason_details: None,
                    },
                )
                .await
                .map_err(|_| TestError::ApiRequest)?;
            }
        }

        Ok(())
    }

    fn default_decision_from_opt(
        default_action: Option<AcceptOrReject>,
    ) -> (bool, Option<ReportSeverity>, Option<BanConfig>) {
        let accepted = match default_action {
            None | Some(AcceptOrReject::Reject) => false,
            Some(AcceptOrReject::Accept) => true,
        };
        (accepted, None, None)
    }

    async fn llm_profile_string_decision(
        text: &str,
        llm: &ProfileStringLlmConfigAndClient,
    ) -> Result<(bool, Option<ReportSeverity>), TestError> {
        let config = &llm.config;
        let expected_response_lowercase = config.db.base.expected_response.to_lowercase();
        let user_text = config.db.base.user_text_template.replace(
            ReportProcessingProfileStringConfigInternal::TEMPLATE_PLACEHOLDER_TEXT,
            text,
        );

        #[allow(deprecated)]
        let r = llm
            .client
            .chat()
            .create(CreateChatCompletionRequest {
                messages: vec![
                    ChatCompletionRequestMessage::System(config.db.base.system_text.clone().into()),
                    ChatCompletionRequestMessage::User(user_text.into()),
                ],
                model: config.llm.model.clone(),
                temperature: config.llm.temperature,
                seed: config.llm.seed,
                reasoning_effort: config
                    .llm
                    .reasoning_effort
                    .map(ReasoningEffortConfig::to_openai_api_type),
                max_completion_tokens: Some(config.llm.max_tokens),
                max_tokens: Some(config.llm.max_tokens),
                ..Default::default()
            })
            .await;

        Self::parse_llm_response(
            r,
            expected_response_lowercase,
            &config.db.automatic_banning_expected_responses,
            config.llm.debug_log_results,
            "report processing (profile string)",
        )
        .await
    }

    async fn llm_profile_content_decision(
        api: &ApiClient,
        content_id: &api_client::models::ContentId,
        target: &api_client::models::AccountId,
        llm: &ProfileContentLlmConfigAndClient,
    ) -> Result<(bool, Option<ReportSeverity>), TestError> {
        let image_data = api_client::apis::media_api::get_content(
            &api.api(),
            &target.aid,
            &content_id.cid,
            Some(false),
            None,
        )
        .await
        .map_err(|_| TestError::ApiRequest)?
        .bytes()
        .await
        .map_err(|_| TestError::ApiRequest)?
        .to_vec();

        let config = &llm.config;
        let expected_response_lowercase = config.db.base.expected_response.to_lowercase();

        let image_part = ChatCompletionRequestMessageContentPartImage {
            image_url: ImageUrl {
                url: format!(
                    "data:image/jpeg;base64,{}",
                    Base64Display::new(&image_data, &base64::engine::general_purpose::STANDARD),
                ),
                detail: None,
            },
        };

        let user_message = ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Array(vec![image_part.into()]),
            name: None,
        };

        #[allow(deprecated)]
        let r = llm
            .client
            .chat()
            .create(CreateChatCompletionRequest {
                messages: vec![
                    ChatCompletionRequestMessage::System(config.db.base.system_text.clone().into()),
                    ChatCompletionRequestMessage::User(user_message),
                ],
                model: config.llm.model.clone(),
                temperature: config.llm.temperature,
                seed: config.llm.seed,
                reasoning_effort: config
                    .llm
                    .reasoning_effort
                    .map(ReasoningEffortConfig::to_openai_api_type),
                max_completion_tokens: Some(config.llm.max_tokens),
                max_tokens: Some(config.llm.max_tokens),
                ..Default::default()
            })
            .await;

        Self::parse_llm_response(
            r,
            expected_response_lowercase,
            &config.db.automatic_banning_expected_responses,
            config.llm.debug_log_results,
            "report processing (profile content)",
        )
        .await
    }

    async fn llm_conversation_decision(
        messages: &[MessageInternal],
        llm: &MessagesLlmConfigAndClient,
    ) -> Result<(bool, Option<ReportSeverity>), TestError> {
        let config = &llm.config;
        let expected_response_lowercase = config.db.base.expected_response.to_lowercase();

        // Build combined conversation text: one entry per message,
        // using the appropriate template based on who sent the message
        // relative to the report creator.
        let mut combined = String::new();
        for msg in messages {
            let template = if msg.chat_message.sender.aid == msg.report.info.creator.aid {
                &config.db.report_creator_message_template
            } else {
                &config.db.report_target_message_template
            };
            let message_paragraph = msg.decoded_text.lines().collect::<Vec<&str>>().join(" ");
            let line = template
                .replace(
                    ReportProcessingMessagesConfigInternal::TEMPLATE_PLACEHOLDER_TEXT,
                    &message_paragraph,
                )
                .replace(
                    ReportProcessingMessagesConfigInternal::TEMPLATE_PLACEHOLDER_MESSAGE_NUMBER,
                    &msg.chat_message.message_number.mn.to_string(),
                );
            combined.push_str(&line);
            combined.push('\n');
        }

        #[allow(deprecated)]
        let r = llm
            .client
            .chat()
            .create(CreateChatCompletionRequest {
                messages: vec![
                    ChatCompletionRequestMessage::System(config.db.base.system_text.clone().into()),
                    ChatCompletionRequestMessage::User(combined.into()),
                ],
                model: config.llm.model.clone(),
                temperature: config.llm.temperature,
                seed: config.llm.seed,
                reasoning_effort: config
                    .llm
                    .reasoning_effort
                    .map(ReasoningEffortConfig::to_openai_api_type),
                max_completion_tokens: Some(config.llm.max_tokens),
                max_tokens: Some(config.llm.max_tokens),
                ..Default::default()
            })
            .await;

        Self::parse_llm_response(
            r,
            expected_response_lowercase,
            &config.db.automatic_banning_expected_responses,
            config.llm.debug_log_results,
            "report processing (messages)",
        )
        .await
    }

    async fn parse_llm_response(
        r: std::result::Result<
            async_openai::types::chat::CreateChatCompletionResponse,
            async_openai::error::OpenAIError,
        >,
        expected_response_lowercase: String,
        expected_responses: &AutomaticBanningExpectedLlmResponsesConfig,
        debug_log_results: bool,
        log_label: &'static str,
    ) -> Result<(bool, Option<ReportSeverity>), TestError> {
        let response = match r.map(|r| r.choices.into_iter().next()) {
            Ok(Some(r)) => match r.message.content {
                Some(response) => response,
                None => {
                    return Err(TestError::LlmError)
                        .attach(format!("{log_label} error: no response content"));
                }
            },
            Ok(None) => {
                return Err(TestError::LlmError).attach(format!("{log_label} error: no response"));
            }
            Err(e) => {
                return Err(TestError::LlmError).attach(format!("{log_label} failed: {e}"));
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_first_line.contains(&expected_response_lowercase);

        let severity = if accepted {
            ReportSeverity::from_llm_response(response_first_line, expected_responses)
        } else {
            None
        };

        if debug_log_results {
            info!("LLM {log_label} result: '{}'", response);
        }

        Ok((accepted, severity))
    }
}
