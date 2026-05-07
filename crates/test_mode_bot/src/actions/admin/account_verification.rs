use api_client::{
    apis::{account_admin_api, media_admin_api},
    models::{
        AccountVerificationQueueAdminItem, PostAccountVerificationQueueRemoveNextItem,
        PostSecurityContentVerifiedValue,
    },
};
use config::bot_config_file::internal::AccountVerificationConfig;
use error_stack::{Result, ResultExt};
use test_mode_utils::client::{ApiClient, TestError};

use super::EmptyPage;

mod security_content;

pub use security_content::AccountVerificationState;

#[derive(Debug)]
pub struct AdminBotAccountVerificationLogic;

enum VerificationMethodAction {
    Accept,
    Reject,
    _CheckImage(Vec<u8>),
}

impl AdminBotAccountVerificationLogic {
    async fn verify_one_page(
        api: &ApiClient,
        config: &AccountVerificationConfig,
        state: &mut AccountVerificationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let item = match Self::get_next_queue_item(api).await? {
            Some(item) => item,
            None => return Ok(Some(EmptyPage)),
        };

        let security_content =
            media_admin_api::get_security_content_admin_info(&api.api(), &item.account_id.aid)
                .await
                .change_context(TestError::ApiRequest)?;

        let Some(security_content) = security_content.content.flatten().map(|v| v.cid) else {
            Self::remove_next_queue_item(api, *item.account_id).await?;
            return Ok(None);
        };

        let value = match Self::parse_verification_method_action(
            config,
            &item.verification_method,
            &item.verification_data,
        )? {
            VerificationMethodAction::Accept => Some(true),
            VerificationMethodAction::Reject => Some(false),
            VerificationMethodAction::_CheckImage(verification_image) => {
                if let Some(config) = &config.security_content {
                    security_content::handle_check_image_method(
                        api,
                        config,
                        state,
                        &item.account_id,
                        &security_content,
                        verification_image,
                    )
                    .await?
                } else {
                    None
                }
            }
        };

        let account_id = (*item.account_id).clone();

        media_admin_api::post_security_content_verified_value(
            &api.api(),
            PostSecurityContentVerifiedValue {
                account_id: Box::new(account_id.clone()),
                security_content,
                value: Some(value),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?;

        Self::remove_next_queue_item(api, account_id).await?;

        Ok(None)
    }

    fn parse_verification_method_action(
        config: &AccountVerificationConfig,
        verification_method: &str,
        _verification_data: &str,
    ) -> Result<VerificationMethodAction, TestError> {
        match verification_method.trim().to_lowercase().as_str() {
            "debug_accept" if config.allowed_methods.debug_accept => {
                Ok(VerificationMethodAction::Accept)
            }
            "debug_reject" if config.allowed_methods.debug_reject => {
                Ok(VerificationMethodAction::Reject)
            }
            // TODO: eudi
            _ => Err(TestError::AdminBotInternalError).attach_printable(
                "Unsupported or disabled account verification method".to_string(),
            ),
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
    ) -> Result<(), TestError> {
        account_admin_api::post_account_verification_queue_remove_next_item(
            &api.api(),
            PostAccountVerificationQueueRemoveNextItem {
                account_id: Box::new(account_id),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?;

        Ok(())
    }

    pub async fn run_account_verification(
        api: &ApiClient,
        config: &AccountVerificationConfig,
        state: &mut AccountVerificationState,
    ) -> Result<(), TestError> {
        loop {
            if Self::verify_one_page(api, config, state).await?.is_some() {
                return Ok(());
            }
        }
    }
}
