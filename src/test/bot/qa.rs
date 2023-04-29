//! QA testing
//!

pub mod account;
pub mod profile;
pub mod media;

use std::{fmt::Debug, time::{Duration, Instant}, sync::atomic::AtomicBool};

use api_client::{apis::profile_api::{get_profile, get_default_profile}, models::AccountState};
use async_trait::async_trait;
use tokio::time::sleep;

use crate::test::client::TestError;

use self::{account::ACCOUNT_TESTS, media::MEDIA_TESTS};

use super::{BotState, BotStruct, actions::{BotAction, admin::ModerateMediaModerationRequest, account::{SetAccountSetup, AssertAccountState, Register, Login, CompleteAccountSetup}, media::{SendImageToSlot, MakeModerationRequest}, AssertFailure, SleepMillis}, Completed, utils::{Timer, Counters}, benchmark::UpdateProfileBenchmark};


use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn, log::info};

use super::super::client::{ApiClient};

use crate::{
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

static ADMIN_QUIT_NOTIFICATION: AtomicBool = AtomicBool::new(false);

pub type SingleTest = (&'static str, &'static [&'static [&'static dyn BotAction]]);

#[macro_export]
macro_rules! test {
    ($s:expr,[ $( $actions:expr, )* ] ) => {
        (
            $s,
            &[
                &[   $( &($actions) as &dyn BotAction, )*    ]
            ]
        )
    };
}

pub const ALL_QA_TESTS: &'static [&'static [SingleTest]] = &[
    ACCOUNT_TESTS,
    MEDIA_TESTS,
];

pub fn test_count() -> usize {
    ALL_QA_TESTS
        .iter()
        .map(|tests| tests.len())
        .sum()
}

#[derive(Debug)]
pub struct QaState {
    // pub update_profile_timer: Timer,
    // pub print_info_timer: Timer,
    // pub action_duration: Instant,
}

impl QaState {
    pub fn new() -> Self {
        Self {
            // update_profile_timer: Timer::new(Duration::from_millis(1000)),
            // print_info_timer: Timer::new(Duration::from_millis(1000)),
            // action_duration: Instant::now(),
        }
    }
}

pub struct Qa {
    state: BotState,
    test_name: &'static str,
    actions: Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>,
}

impl Debug for Qa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(self.test_name).finish()
    }
}

impl Qa {
    pub fn user_test(
        state: BotState,
        test_name: &'static str,
        actions: Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>,
    ) -> Self {
        Self {
            state,
            test_name,
            actions,
        }
    }

    pub fn admin(state: BotState) -> Self {
        let setup = [
            &Register as &dyn BotAction,
            &Login,
            SetAccountSetup::admin(),
            &SendImageToSlot(0),
            &SendImageToSlot(1),
            &MakeModerationRequest { camera: true },
            &CompleteAccountSetup,
            &AssertAccountState(AccountState::Normal),
        ];
        let admin_actions = [
            &SleepMillis(250) as &dyn BotAction,
            &ModerateMediaModerationRequest as &dyn BotAction,
        ];

        let iter = setup
            .into_iter()
            .chain(admin_actions.into_iter().cycle().take_while(|_| {
                !ADMIN_QUIT_NOTIFICATION.load(std::sync::atomic::Ordering::Relaxed)
            }));
        Self {
            state,
            test_name: "Admin bot",
            actions: Box::new(iter),
        }
    }
}

#[async_trait]
impl BotStruct for Qa {
    fn next_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState) {
        (self.actions.next(), &mut self.state)
    }

    fn state(&self) -> &BotState {
        &self.state
    }


    fn notify_task_bot_count_decreased(&mut self, bot_count: usize) {
        if bot_count <= 1 {
            // Only admin bot running

            if bot_count == 1 {
                info!("User bots quited. Quiting admin bot.");
            }

            ADMIN_QUIT_NOTIFICATION.store(true, std::sync::atomic::Ordering::Relaxed)
        }
    }
}
