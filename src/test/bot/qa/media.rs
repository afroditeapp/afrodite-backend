

use std::{fmt::Debug, time::{Duration, Instant}, sync::atomic::AtomicBool};

use api_client::{apis::profile_api::{get_profile, get_default_profile}, models::AccountState};
use async_trait::async_trait;
use tokio::time::sleep;

use crate::test::{client::TestError, bot::actions::BotAction};

use super::{BotState, BotStruct, super::actions::{admin::ModerateMediaModerationRequest, account::{SetAccountSetup, AssertAccountState, Register, Login, CompleteAccountSetup}, media::{SendImageToSlot, MakeModerationRequest}, AssertFailure}, Completed, super::utils::{Timer, Counters}, super::benchmark::UpdateProfileBenchmark, SingleTest};


use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn, log::info};

use super::super::super::client::{ApiClient};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

use crate::test;

pub const MEDIA_TESTS: &[SingleTest] = &[
    test!(
        "Save image to slot: max 3 slots",
        [
            Register,
            Login,
            AssertFailure(SendImageToSlot(3)),
        ]
    ),
];
