

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


pub const ACCOUNT_TESTS: &[SingleTest] = &[
    test!(
        "Initial setup: correct account state after login",
        [
            Register,
            Login,
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: complete setup fails if no setup info is set",
        [
            Register,
            Login,
            SendImageToSlot(0),
            SendImageToSlot(1),
            MakeModerationRequest { camera: true },
            AssertFailure(CompleteAccountSetup),
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: complete setup fails if no image moderation request",
        [
            Register,
            Login,
            SetAccountSetup::new(),
            AssertFailure(CompleteAccountSetup),
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: complete setup fails if image request does not contain camera image",
        [
            Register,
            Login,
            SetAccountSetup::new(),
            SendImageToSlot(0),
            SendImageToSlot(1),
            MakeModerationRequest { camera: false },
            AssertFailure(CompleteAccountSetup),
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: successful",
        [
            Register,
            Login,
            SetAccountSetup::new(),
            SendImageToSlot(0),
            SendImageToSlot(1),
            MakeModerationRequest { camera: true },
            CompleteAccountSetup,
            AssertAccountState(AccountState::Normal),
        ]
    ),
];
