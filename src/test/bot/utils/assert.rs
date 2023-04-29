
use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login, post_account_setup, get_account_state}, profile_api::{post_profile, get_profile, get_default_profile}}, models::{Profile, account_setup, AccountSetup, AccountState}};
use async_trait::async_trait;

use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn};

use super::{super::super::client::{ApiClient, TestError}, BotAction};

use crate::{
    config::args::{Test, TestMode},
    utils::IntoReportExt, test::bot::utils::name::NameProvider,
};

pub fn bot_assert_eq<
    T: Debug + PartialEq,
>(value: T, expected: T) -> Result<(), TestError> {
    if value == expected {
        Ok(())
    } else {
        Err(TestError::AssertError(format!("value: {:?}, expected: {:?}", value, expected)).into())
    }
}
