use api_client::models::EditVerificationProfileName;
use config::bot_config_file::internal::AccountVerificationConfig;
use error_stack::Result;
use test_mode_utils::{AccountVerificationErrorFlags, client::TestError};

use super::{LazyProfileAgeAndName, VerificationMethodAction};

pub async fn handle_profile_name_verification(
    config: &AccountVerificationConfig,
    method_action: &VerificationMethodAction,
    age_and_name: &mut LazyProfileAgeAndName<'_>,
) -> Result<
    (
        Option<EditVerificationProfileName>,
        AccountVerificationErrorFlags,
    ),
    TestError,
> {
    if !config.profile_name {
        return Ok((
            None,
            AccountVerificationErrorFlags::PROFILE_NAME_VERIFICATION_FAILED,
        ));
    }

    let Some(profile_name) = age_and_name.name().await? else {
        return Ok((
            None,
            AccountVerificationErrorFlags::PROFILE_NAME_VERIFICATION_FAILED,
        ));
    };

    let (accepted, flags) = match method_action {
        VerificationMethodAction::Accept => (true, AccountVerificationErrorFlags::empty()),
        VerificationMethodAction::Reject => (false, AccountVerificationErrorFlags::empty()),
        VerificationMethodAction::_PersonIdentificationData { names, .. } => {
            let profile_name = profile_name.trim().to_lowercase();
            if profile_name.is_empty() || names.is_empty() {
                (
                    false,
                    AccountVerificationErrorFlags::PROFILE_NAME_VERIFICATION_FAILED,
                )
            } else {
                let mut accepted = false;
                for name in names {
                    if name.trim().to_lowercase() == profile_name {
                        accepted = true;
                        break;
                    }
                }
                if accepted {
                    (true, AccountVerificationErrorFlags::empty())
                } else {
                    (
                        false,
                        AccountVerificationErrorFlags::PROFILE_NAME_VERIFICATION_MISMATCH,
                    )
                }
            }
        }
    };

    Ok((
        Some(EditVerificationProfileName {
            current_profile_name: Some(profile_name),
            verified_value: Some(Some(accepted)),
        }),
        flags,
    ))
}
