use api_client::models::EditVerificationProfileAgeRange;
use config::bot_config_file::internal::AccountVerificationConfigInternal;
use error_stack::Result;
use test_mode_utils::{AccountVerificationErrorFlags, client::TestError};

use super::{LazyProfileAgeAndName, VerificationMethodAction};

pub async fn handle_profile_age_range_verification(
    config: &AccountVerificationConfigInternal,
    method_action: &VerificationMethodAction,
    age_and_name: &mut LazyProfileAgeAndName<'_>,
) -> Result<
    (
        Option<EditVerificationProfileAgeRange>,
        AccountVerificationErrorFlags,
    ),
    TestError,
> {
    if !config.profile_age_range {
        return Ok((
            None,
            AccountVerificationErrorFlags::PROFILE_AGE_RANGE_VERIFICATION_FAILED,
        ));
    }

    let current_profile_age = age_and_name.age().await?;
    let (accepted, flags) = match method_action {
        VerificationMethodAction::Accept => (true, AccountVerificationErrorFlags::empty()),
        VerificationMethodAction::Reject => (false, AccountVerificationErrorFlags::empty()),
        VerificationMethodAction::_PersonIdentificationData { age: None, .. } => (
            false,
            AccountVerificationErrorFlags::PROFILE_AGE_RANGE_VERIFICATION_FAILED,
        ),
        VerificationMethodAction::_PersonIdentificationData { age: Some(age), .. } => {
            if current_profile_age == Into::<i32>::into(*age) {
                (true, AccountVerificationErrorFlags::empty())
            } else {
                (
                    false,
                    AccountVerificationErrorFlags::PROFILE_AGE_RANGE_VERIFICATION_MISMATCH,
                )
            }
        }
    };

    Ok((
        Some(EditVerificationProfileAgeRange {
            current_profile_age,
            verified_value: Some(Some(accepted)),
        }),
        flags,
    ))
}
