pub mod account;
pub mod admin;
pub mod chat;
pub mod media;
pub mod profile;

use std::{collections::HashSet, fmt::Debug, time::Duration};

use account::AccountState;
use api_client::models::{Account, Location, Profile};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use test_mode_utils::client::TestError;

use self::{
    account::{AssertAccountState, CompleteAccountSetup, Login, Register, SetAccountSetup},
    media::{SendImageToSlot, SetContent},
};
use super::{BotState, TaskState};
use crate::actions::profile::ChangeBotAgeAndOtherSettings;

#[macro_export]
macro_rules! action_array {
    [ $( $actions:expr ),* $(,)? ] => {
        &[   $( &($actions) as &dyn $crate::actions::BotAction, )*    ]
    };
}

pub type ActionArray = &'static [&'static dyn BotAction];

#[derive(Debug, PartialEq, Clone)]
pub enum PreviousValue {
    Empty,
    Account(Account),
    Profiles(HashSet<String>),
    Profile(Profile),
    Location(Location),
}

impl PreviousValue {
    pub fn profile_count(&self) -> usize {
        if let PreviousValue::Profiles(p) = self {
            p.len()
        } else {
            0
        }
    }

    pub fn account(&self) -> Account {
        if let PreviousValue::Account(a) = self {
            a.clone()
        } else {
            Account::default()
        }
    }

    pub fn profile(&self) -> Profile {
        if let PreviousValue::Profile(p) = self {
            p.clone()
        } else {
            Profile::default()
        }
    }

    pub fn location(&self) -> Location {
        if let PreviousValue::Location(location) = self {
            *location
        } else {
            Location::default()
        }
    }
}

/// Implementing excecute_impl or excecute_impl_task_state is required.
///
/// If action saves something to previous value attribute, then implement
/// previous_value_supported.
#[async_trait]
pub trait BotAction: Debug + Send + Sync {
    async fn excecute(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        self.excecute_impl_task_state(state, task_state)
            .await
            .attach_printable_lazy(|| format!("{__self:?}"))
    }

    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        _task_state: &mut TaskState,
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
            Ok(()) => Err(TestError::AssertError("API request did not fail".to_string()).report()),
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
            Err(TestError::BotIsWaiting.report())
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

pub struct AssertEqualsTestFn<T: PartialEq>(
    pub fn(PreviousValue, &BotState) -> T,
    pub fn() -> T,
    pub &'static dyn BotAction,
);

impl<T: PartialEq> Debug for AssertEqualsTestFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("AssertEqualsTestFn for action {:?}", self.2))
    }
}

#[async_trait]
impl<T: PartialEq + Send + Sync + 'static + Debug> BotAction for AssertEqualsTestFn<T> {
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
        let expected = self.1();
        if value != expected {
            Err(TestError::AssertError(format!(
                "action: {:?}, was: {:?}, expected: {:?}",
                self.2, value, expected,
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

pub struct RepeatUntilFnSimple<T: PartialEq>(
    pub fn(&BotState) -> T,
    pub T,
    pub &'static dyn BotAction,
);

impl<T: PartialEq> Debug for RepeatUntilFnSimple<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RepeatUntilFnSimple for action {:?}", self.2))
    }
}

#[async_trait]
impl<T: PartialEq + Send + Sync + 'static + Debug> BotAction for RepeatUntilFnSimple<T> {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        loop {
            self.2.excecute(state, task_state).await?;

            let value = self.0(state);
            if value == self.1 {
                break;
            }
        }

        Ok(())
    }
}

pub struct RunFn(pub fn(&BotState));

impl Debug for RunFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RunFn"))
    }
}

#[async_trait]
impl BotAction for RunFn {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        _task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        self.0(state);
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

#[derive(Debug)]
pub struct RunActionsIf(pub ActionArray, pub fn(&BotState) -> bool);

#[async_trait]
impl BotAction for RunActionsIf {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        if self.1(state) {
            for a in self.0.iter() {
                a.excecute(state, task_state).await?;
            }
        }
        Ok(())
    }
}

pub const TO_NORMAL_STATE: ActionArray = action_array![
    Register,
    Login,
    SetAccountSetup::new(),
    SendImageToSlot::slot(0),
    SetContent {
        security_content_slot_i: Some(0),
        content_0_slot_i: Some(0),
    },
    ChangeBotAgeAndOtherSettings { admin: false },
    CompleteAccountSetup,
    AssertAccountState::account(AccountState::Normal),
];

pub const TO_ADMIN_NORMAL_STATE: ActionArray = action_array![
    Register,
    Login,
    SetAccountSetup::admin(),
    SendImageToSlot::slot(0),
    SetContent {
        security_content_slot_i: Some(0),
        content_0_slot_i: Some(0),
    },
    ChangeBotAgeAndOtherSettings { admin: true },
    CompleteAccountSetup,
    AssertAccountState::account(AccountState::Normal),
];
