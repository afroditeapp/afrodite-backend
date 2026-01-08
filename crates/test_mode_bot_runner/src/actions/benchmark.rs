use std::{fmt::Debug, time::Instant};

use api_client::{
    apis::profile_api::{
        get_profile, get_profile_from_database_debug_mode_benchmark,
        post_profile_to_database_debug_mode_benchmark,
    },
    models::ProfileUpdate,
};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use test_mode_bot::{
    BotState,
    actions::{
        BotAction,
        profile::{ChangeProfileText, GetProfileList, ProfileText},
    },
};
use test_mode_utils::client::TestError;
use tracing::log::info;

use crate::utils::Counters;

static COUNTERS: Counters = Counters::new();

#[derive(Debug)]
pub struct GetProfile;

#[async_trait]
impl BotAction for GetProfile {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        get_profile(state.api(), &state.account_id_string()?, None, None)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetProfileListBenchmark;

#[async_trait]
impl BotAction for GetProfileListBenchmark {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let result = GetProfileList.excecute_impl(state).await;
        COUNTERS.inc_sub();
        result
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GetProfileFromDatabase;

#[async_trait]
impl BotAction for GetProfileFromDatabase {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        get_profile_from_database_debug_mode_benchmark(state.api(), &state.account_id_string()?)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PostProfile;

#[async_trait]
impl BotAction for PostProfile {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        ChangeProfileText {
            mode: ProfileText::Random,
        }
        .excecute(state)
        .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PostProfileToDatabase;

#[async_trait]
impl BotAction for PostProfileToDatabase {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        // Uuid has same string size every time.
        let profile = simple_backend_utils::UuidBase64Url::new_random_id();
        let profile = ProfileUpdate {
            attributes: vec![],
            age: 18,
            name: "Bot".to_string(),
            ptext: Some(profile.to_string()),
        };
        post_profile_to_database_debug_mode_benchmark(state.api(), profile)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ActionsBeforeIteration;

#[async_trait]
impl BotAction for ActionsBeforeIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        state.benchmark.action_duration = Instant::now();
        Ok(())
    }
}

#[derive(Debug)]
pub struct ActionsAfterIteration;

#[async_trait]
impl BotAction for ActionsAfterIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        COUNTERS.inc_main();

        if state.benchmark.print_benchmark_info && state.benchmark.print_info_timer.passed() {
            info!(
                "{:?}, total: {}, details: {}",
                state.benchmark.action_duration.elapsed(),
                COUNTERS.reset_main(),
                COUNTERS.reset_sub(),
            );
        }
        Ok(())
    }
}
