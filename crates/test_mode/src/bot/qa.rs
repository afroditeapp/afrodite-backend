//! QA testing
//!

pub mod profile;

use std::{fmt::Debug, iter::Peekable, sync::atomic::AtomicBool};

use api_client::models::AccountState;
use async_trait::async_trait;
use tracing::log::info;

use self::{
    profile::PROFILE_TESTS,
};
use super::{
    actions::{
        account::{AssertAccountState, CompleteAccountSetup, Login, Register, SetAccountSetup},
        admin::ModerateMediaModerationRequest,
        media::{MakeModerationRequest, SendImageToSlot},
        BotAction, SleepMillis,
    },
    BotState, BotStruct,
};
use crate::{action_array, bot::actions::ActionArray};

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
    PROFILE_TESTS,
];

pub fn test_count() -> usize {
    ALL_QA_TESTS.iter().map(|tests| tests.len()).sum()
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
    actions: Peekable<Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>>,
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
            actions: actions.peekable(),
        }
    }

    pub fn admin(state: BotState) -> Self {
        const SETUP: ActionArray = action_array![
            Register,
            Login,
            SetAccountSetup::admin(),
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            MakeModerationRequest { slot_0_secure_capture: true },
            CompleteAccountSetup,
            AssertAccountState(AccountState::Normal),
        ];
        const ADMIN_ACTIONS: ActionArray =
            action_array![SleepMillis(1), ModerateMediaModerationRequest::moderate_initial_content()];

        let iter = SETUP
            .into_iter()
            .chain(ADMIN_ACTIONS.into_iter().cycle().take_while(|_| {
                !ADMIN_QUIT_NOTIFICATION.load(std::sync::atomic::Ordering::Relaxed)
            }))
            .map(|a| *a);
        Self {
            state,
            test_name: "Admin bot",
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }
}

#[async_trait]
impl BotStruct for Qa {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState) {
        let action = self.actions.peek().map(|a| *a);
        (action, &mut self.state)
    }

    fn next_action(&mut self) {
        self.actions.next();
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
