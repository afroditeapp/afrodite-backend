use std::sync::Arc;

use api_client::{
    apis::{account_admin_api, profile_admin_api},
    models::{
        AccountVerificationErrorFlagsValue, AccountVerificationQueueAdminItem,
        EditVerificationValues, PostAccountVerificationQueueRemoveNextItem, VerificationMethod,
    },
};
use async_openai::{Client, config::OpenAIConfig};
use config::bot_config_file::internal::{
    AccountVerificationConfig, LlmSecurityContentVerificationConfig,
};
use error_stack::{Result, ResultExt};
use test_mode_utils::{
    AccountVerificationErrorFlags,
    client::{ApiClient, TestError},
};

use super::EmptyPage;

mod profile_age_range;
mod profile_name;
mod security_content;

#[derive(Debug, Default)]
pub struct AccountVerificationState {
    llm: Option<LlmConfigAndClient>,
}

impl AccountVerificationState {
    pub fn new(config: &AccountVerificationConfig, reqwest_client: reqwest::Client) -> Self {
        let llm = config
            .security_content
            .as_ref()
            .and_then(|v| v.llm.clone())
            .map(|config| LlmConfigAndClient {
                client: Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base(config.openai_api_url.to_string())
                        .with_api_key(""),
                )
                .with_http_client(reqwest_client.clone()),
                config: config.into(),
            });

        Self { llm }
    }
}

#[derive(Debug, Clone)]
struct LlmConfigAndClient {
    config: Arc<LlmSecurityContentVerificationConfig>,
    client: Client<OpenAIConfig>,
}

#[derive(Debug)]
pub struct AdminBotAccountVerificationLogic;

struct LazyProfileAgeAndName<'a> {
    api: &'a ApiClient,
    aid: &'a str,
    value: Option<api_client::models::GetProfileAgeAndName>,
}

impl<'a> LazyProfileAgeAndName<'a> {
    fn new(api: &'a ApiClient, aid: &'a str) -> Self {
        Self {
            api,
            aid,
            value: None,
        }
    }

    async fn get(&mut self) -> Result<&api_client::models::GetProfileAgeAndName, TestError> {
        if let Some(value) = self.value.take() {
            Ok(self.value.insert(value))
        } else {
            let value = profile_admin_api::get_profile_age_and_name(&self.api.api(), self.aid)
                .await
                .change_context(TestError::ApiRequest)?;
            Ok(self.value.insert(value))
        }
    }

    async fn age(&mut self) -> Result<i32, TestError> {
        Ok(self.get().await?.age)
    }

    async fn name(&mut self) -> Result<Option<String>, TestError> {
        Ok(self.get().await?.name.clone())
    }
}

enum VerificationMethodAction {
    Accept,
    Reject,
    _PersonIdentificationData {
        jpeg_image: Option<Vec<u8>>,
        age: Option<u8>,
        names: Vec<String>,
    },
}

impl AdminBotAccountVerificationLogic {
    async fn verify_one_page(
        api: &ApiClient,
        config: &AccountVerificationConfig,
        state: &AccountVerificationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let item = match Self::get_next_queue_item(api).await? {
            Some(item) => item,
            None => return Ok(Some(EmptyPage)),
        };

        let account_id = (*item.account_id).clone();
        let method_action = match Self::parse_verification_method_action(
            config,
            &item.verification_method,
            &item.verification_data,
        ) {
            Ok(method_action) => method_action,
            Err(verification_error_flags) => {
                Self::remove_next_queue_item(
                    api,
                    account_id.clone(),
                    EditVerificationValues::default(),
                    verification_error_flags,
                )
                .await?;
                return Ok(None);
            }
        };

        let mut age_and_name = LazyProfileAgeAndName::new(api, &account_id.aid);

        let (profile_age_range, profile_age_range_flags) = if item
            .verification_scope
            .profile_age_range
            .unwrap_or_default()
        {
            profile_age_range::handle_profile_age_range_verification(
                config,
                &method_action,
                &mut age_and_name,
            )
            .await?
        } else {
            (None, AccountVerificationErrorFlags::empty())
        };

        let (profile_name, profile_name_flags) =
            if item.verification_scope.profile_name.unwrap_or_default() {
                profile_name::handle_profile_name_verification(
                    config,
                    &method_action,
                    &mut age_and_name,
                )
                .await?
            } else {
                (None, AccountVerificationErrorFlags::empty())
            };

        let (security_content, security_content_flags) =
            if item.verification_scope.security_content.unwrap_or_default() {
                security_content::handle_security_content_verification(
                    api,
                    config,
                    state,
                    &account_id,
                    &method_action,
                )
                .await?
            } else {
                (None, AccountVerificationErrorFlags::empty())
            };

        Self::remove_next_queue_item(
            api,
            account_id,
            EditVerificationValues {
                profile_age_range: Some(profile_age_range.map(Box::new)),
                profile_name: Some(profile_name.map(Box::new)),
                security_content: Some(security_content.map(Box::new)),
            },
            profile_age_range_flags | profile_name_flags | security_content_flags,
        )
        .await?;

        Ok(None)
    }

    fn parse_verification_method_action(
        config: &AccountVerificationConfig,
        verification_method: &VerificationMethod,
        _verification_data: &str,
    ) -> std::result::Result<VerificationMethodAction, AccountVerificationErrorFlags> {
        match verification_method {
            VerificationMethod::DebugAccept => {
                if config.allowed_methods.debug_accept {
                    Ok(VerificationMethodAction::Accept)
                } else {
                    Err(AccountVerificationErrorFlags::VERIFICATION_METHOD_DISABLED)
                }
            }
            VerificationMethod::DebugReject => {
                if config.allowed_methods.debug_reject {
                    Ok(VerificationMethodAction::Reject)
                } else {
                    Err(AccountVerificationErrorFlags::VERIFICATION_METHOD_DISABLED)
                }
            }
            VerificationMethod::Eudi => {
                if config.allowed_methods.eudi {
                    // TODO: Implement eudi verification method
                    Ok(VerificationMethodAction::Reject)
                } else {
                    Err(AccountVerificationErrorFlags::VERIFICATION_METHOD_DISABLED)
                }
            }
        }
    }

    async fn get_next_queue_item(
        api: &ApiClient,
    ) -> Result<Option<AccountVerificationQueueAdminItem>, TestError> {
        let response = account_admin_api::get_account_verification_queue_next_item(&api.api())
            .await
            .change_context(TestError::ApiRequest)?
            .item
            .flatten()
            .map(|item| *item);

        Ok(response)
    }

    async fn remove_next_queue_item(
        api: &ApiClient,
        account_id: api_client::models::AccountId,
        edit: EditVerificationValues,
        verification_error_flags: AccountVerificationErrorFlags,
    ) -> Result<(), TestError> {
        account_admin_api::post_account_verification_queue_remove_next_item(
            &api.api(),
            PostAccountVerificationQueueRemoveNextItem {
                account_id: Box::new(account_id),
                edit: Some(Some(Box::new(edit))),
                verification_error_flags: Box::new(AccountVerificationErrorFlagsValue {
                    v: verification_error_flags.bits().into(),
                }),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?;

        Ok(())
    }

    pub async fn run_account_verification(
        api: &ApiClient,
        config: &AccountVerificationConfig,
        state: &AccountVerificationState,
    ) -> Result<(), TestError> {
        loop {
            if Self::verify_one_page(api, config, state).await?.is_some() {
                return Ok(());
            }
        }
    }
}
