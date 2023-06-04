pub mod account;
pub mod admin;
pub mod common;
pub mod media;
pub mod profile;

use std::collections::HashSet;
use std::{fmt::Debug, time::Duration};

use api_client::models::{AccountState, Location};
use async_trait::async_trait;

use error_stack::{FutureExt, Result, ResultExt};

use self::account::{AssertAccountState, CompleteAccountSetup, Login, Register, SetAccountSetup};
use self::media::{MakeModerationRequest, SendImageToSlot};

use super::super::client::TestError;

use super::{BotState, TaskState};

#[macro_export]
macro_rules! action_array {
    [ $( $actions:expr, )* ] => {
        &[   $( &($actions) as &dyn BotAction, )*    ]
    };
}

pub type ActionArray = &'static [&'static dyn BotAction];

#[derive(Debug, PartialEq, Clone)]
pub enum PreviousValue {
    Empty,
    Profiles(HashSet<String>),
}

impl PreviousValue {
    pub fn profile_count(&self) -> usize {
        if let PreviousValue::Profiles(p) = self {
            p.len()
        } else {
            0
        }
    }
}

/// Implementing excecute_impl or excecute_impl_task_state is required.
///
/// If action saves something to previous value attribute, then implement
/// previous_value_supported.
#[async_trait]
pub trait BotAction: Debug + Send + Sync + 'static {
    async fn excecute(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        self.excecute_impl_task_state(state, task_state)
            .await
            .attach_printable_lazy(|| format!("{:?}", self))
    }

    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        self.excecute_impl(state).await
    }

    fn previous_value_supported(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct DoNothing;

#[async_trait]
impl BotAction for DoNothing {
    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct AssertFailure<T: BotAction>(pub T);

#[async_trait]
impl<T: BotAction> BotAction for AssertFailure<T> {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        match self.0.excecute(state, task_state).await {
            Err(e) => match e.current_context() {
                TestError::ApiRequest => Ok(()),
                _ => Err(e),
            },
            Ok(()) => Err(TestError::AssertError("API request did not fail".to_string()).into()),
        }
    }
}

/// Sleep milliseconds
#[derive(Debug)]
pub struct SleepMillis(pub u64);

#[async_trait]
impl BotAction for SleepMillis {
    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        tokio::time::sleep(Duration::from_millis(self.0)).await;
        Ok(())
    }
}

/// Bot sleeps (this task is not removed) until the function evalues true.
pub struct SleepUntil(pub fn(&TaskState) -> bool);

#[async_trait]
impl BotAction for SleepUntil {
    async fn excecute_impl_task_state(
        &self,
        _state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        if self.0(task_state) {
            Ok(())
        } else {
            Err(TestError::BotIsWaiting.into())
        }
    }
}

impl Debug for SleepUntil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SleepUntil"))
    }
}

pub struct ModifyTaskState(pub fn(&mut TaskState));

#[async_trait]
impl BotAction for ModifyTaskState {
    async fn excecute_impl_task_state(
        &self,
        _state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        self.0(task_state);
        Ok(())
    }
}

impl Debug for ModifyTaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ModifyTaskState"))
    }
}

#[derive(Debug)]
pub struct AssertEquals(pub PreviousValue, pub &'static dyn BotAction);

#[async_trait]
impl BotAction for AssertEquals {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        if !self.1.previous_value_supported() {
            return Err(TestError::AssertError(format!(
                "Previous value not supported for action {:?}",
                self.1
            ))
            .into());
        }

        self.1.excecute(state, task_state).await?;

        if self.0 != state.previous_value {
            Err(TestError::AssertError(format!(
                "action: {:?}, was: {:?}, expected: {:?}",
                self.1, state.previous_value, self.0
            ))
            .into())
        } else {
            Ok(())
        }
    }
}

pub struct AssertEqualsFn<T: PartialEq>(
    pub fn(PreviousValue, &BotState) -> T,
    pub T,
    pub &'static dyn BotAction,
);

impl<T: PartialEq> Debug for AssertEqualsFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("AssertEqualsFn for action {:?}", self.2))
    }
}

#[async_trait]
impl<T: PartialEq + Send + Sync + 'static + Debug> BotAction for AssertEqualsFn<T> {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        if !self.2.previous_value_supported() {
            return Err(TestError::AssertError(format!(
                "Previous value not supported for action {:?}",
                self.2
            ))
            .into());
        }

        self.2.excecute(state, task_state).await?;

        let value = self.0(state.previous_value.clone(), state);
        if value != self.1 {
            Err(TestError::AssertError(format!(
                "action: {:?}, was: {:?}, expected: {:?}",
                self.2, value, self.1
            ))
            .into())
        } else {
            Ok(())
        }
    }
}

pub struct RepeatUntilFn<T: PartialEq>(
    pub fn(PreviousValue, &BotState) -> T,
    pub T,
    pub &'static dyn BotAction,
);

impl<T: PartialEq> Debug for RepeatUntilFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RepeatUntilFn for action {:?}", self.2))
    }
}

#[async_trait]
impl<T: PartialEq + Send + Sync + 'static + Debug> BotAction for RepeatUntilFn<T> {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        if !self.2.previous_value_supported() {
            return Err(TestError::AssertError(format!(
                "Previous value not supported for action {:?}",
                self.2
            ))
            .into());
        }

        loop {
            self.2.excecute(state, task_state).await?;

            let value = self.0(state.previous_value.clone(), state);
            if value == self.1 {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RunActions(pub ActionArray);

#[async_trait]
impl BotAction for RunActions {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        for a in self.0.iter() {
            a.excecute(state, task_state).await?;
        }

        Ok(())
    }
}

pub const TO_NORMAL_STATE: ActionArray = action_array![
    Register,
    Login,
    SetAccountSetup::new(),
    SendImageToSlot(0),
    SendImageToSlot(1),
    MakeModerationRequest { camera: true },
    CompleteAccountSetup,
    AssertAccountState(AccountState::Normal),
];
