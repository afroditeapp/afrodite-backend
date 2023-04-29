mod actions;
mod benchmark;
mod qa;
mod utils;

use std::{fmt::Debug, sync::Arc, vec};

use api_client::models::AccountIdLight;

use async_trait::async_trait;
use tokio::{
    select,
    sync::{mpsc, watch},
};

use error_stack::{Result, ResultExt};

use tracing::{error, info, log::warn};

use self::{
    actions::{media::MediaState, BotAction, DoNothing},
    benchmark::{Benchmark, BenchmarkState},
    qa::Qa,
};

use super::client::{ApiClient, TestError};

use crate::config::args::{Test, TestMode};

#[derive(Debug)]
pub struct BotState {
    pub id: Option<AccountIdLight>,
    pub config: Arc<TestMode>,
    pub task_id: u32,
    pub bot_id: u32,
    pub api: ApiClient,
    pub previous_action: &'static dyn BotAction,
    pub action_history: Vec<&'static dyn BotAction>,
    pub benchmark: BenchmarkState,
    pub media: MediaState,
}

impl BotState {
    pub fn new(
        id: Option<AccountIdLight>,
        config: Arc<TestMode>,
        task_id: u32,
        bot_id: u32,
        api: ApiClient,
    ) -> Self {
        Self {
            id,
            config,
            task_id,
            bot_id,
            api,
            benchmark: BenchmarkState::new(),
            previous_action: &DoNothing,
            action_history: vec![],
            media: MediaState::new(),
        }
    }

    pub fn id(&self) -> Result<AccountIdLight, TestError> {
        self.id.ok_or(TestError::AccountIdMissing.into())
    }

    pub fn id_string(&self) -> Result<String, TestError> {
        self.id
            .ok_or(TestError::AccountIdMissing.into())
            .map(|id| id.to_string())
    }

    pub fn is_first_bot(&self) -> bool {
        self.task_id == 0 && self.bot_id == 0
    }

    pub fn print_info(&mut self) -> bool {
        self.is_first_bot() && self.benchmark.print_info_timer.passed()
    }
}

pub struct Completed;

#[async_trait]
pub trait BotStruct: Debug + Send + 'static {
    fn next_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState);
    fn state(&self) -> &BotState;

    async fn run_action(&mut self) -> Result<Option<Completed>, TestError> {
        let mut result = self.run_action_impl().await;
        if let Test::Qa = self.state().config.test {
            result = result.attach_printable_lazy(|| format!("{:?}", self.state().action_history))
        }
        result.attach_printable_lazy(|| format!("{:?}", self))
    }

    async fn run_action_impl(&mut self) -> Result<Option<Completed>, TestError> {
        match self.next_action_and_state() {
            (None, _) => Ok(Some(Completed)),
            (Some(action), state) => {
                let result = action.excecute(state).await.map(|_| None);
                state.previous_action = action;
                if let Test::Qa = state.config.test {
                    state.action_history.push(action)
                }
                result
            }
        }
    }

    fn notify_task_bot_count_decreased(&mut self, bot_count: usize) {
        let _ = bot_count;
    }
}

pub struct BotManager {
    bots: Vec<Box<dyn BotStruct>>,
    _bot_running_handle: mpsc::Sender<()>,
    task_id: u32,
}

impl BotManager {
    pub fn spawn(
        task_id: u32,
        config: Arc<TestMode>,
        id: impl Into<Option<AccountIdLight>>,
        bot_quit_receiver: watch::Receiver<()>,
        _bot_running_handle: mpsc::Sender<()>,
    ) {
        let id = id.into();
        let bot = match config.test {
            Test::BenchmarkDefault | Test::BenchmarkNormal => {
                Self::benchmark(task_id, id, config, _bot_running_handle)
            }
            Test::Qa => Self::qa(task_id, id, config, _bot_running_handle),
        };

        tokio::spawn(bot.run(bot_quit_receiver));
    }

    pub fn benchmark(
        task_id: u32,
        id: Option<AccountIdLight>,
        config: Arc<TestMode>,
        _bot_running_handle: mpsc::Sender<()>,
    ) -> Self {
        let mut bots = Vec::<Box<dyn BotStruct>>::new();
        for bot_i in 0..config.bot_count {
            let state = BotState::new(
                id,
                config.clone(),
                task_id,
                bot_i,
                ApiClient::new(config.server.api_urls.clone()),
            );
            let benchmark = match config.test {
                Test::BenchmarkNormal => Benchmark::get_profile_benchmark(state),
                Test::BenchmarkDefault => Benchmark::get_default_profile_benchmark(state),
                _ => panic!("Invalid test {:?}", config.test),
            };
            bots.push(Box::new(benchmark))
        }

        Self {
            bots,
            _bot_running_handle,
            task_id,
        }
    }

    pub fn qa(
        task_id: u32,
        id: Option<AccountIdLight>,
        config: Arc<TestMode>,
        _bot_running_handle: mpsc::Sender<()>,
    ) -> Self {
        if task_id >= 1 {
            panic!("Only task count 1 is supported for QA tests");
        }

        let required_bots = qa::test_count() + 1;

        if (config.bot_count as usize) < required_bots {
            warn!("Increasing bot count to {}", required_bots);
        }

        let mut bots = Vec::<Box<dyn BotStruct>>::new();
        let new_bot_state = |bot_i| {
            BotState::new(
                id,
                config.clone(),
                task_id,
                bot_i,
                ApiClient::new(config.server.api_urls.clone()),
            )
        };

        bots.push(Box::new(Qa::admin(new_bot_state(0))));

        for (i, (test_name, test)) in qa::ALL_QA_TESTS
            .into_iter()
            .map(|tests| *tests)
            .flatten()
            .enumerate()
        {
            let state = new_bot_state(i as u32 + 1);
            let actions = test
                .into_iter()
                .map(|actions| *actions)
                .flatten()
                .map(|action| *action);
            let bot = Qa::user_test(state, test_name, Box::new(actions));
            bots.push(Box::new(bot));
        }

        Self {
            bots,
            _bot_running_handle,
            task_id,
        }
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        loop {
            select! {
                result = bot_quit_receiver.changed() => {
                    if result.is_err() {
                        break
                    }
                }
                _ = self.run_bot() => {
                    break;
                }
            }
        }
    }

    async fn run_bot(&mut self) {
        let mut errors = false;
        loop {
            if self.bots.is_empty() {
                if errors {
                    error!("All bots closed. Errors occurred.");
                } else {
                    info!("All bots closed. No errors.");
                }
                return;
            }

            if let Some(remove_i) = self.iter_bot_list(&mut errors).await {
                self.bots
                    .swap_remove(remove_i)
                    .notify_task_bot_count_decreased(self.bots.len());
            }
        }
    }

    /// If Some(bot_index) is returned remove the bot.
    async fn iter_bot_list(&mut self, errors: &mut bool) -> Option<usize> {
        for (i, b) in self.bots.iter_mut().enumerate() {
            match b.run_action().await {
                Ok(None) => (),
                Ok(Some(Completed)) => return Some(i),
                Err(e) => {
                    error!("Task {}, bot returned error: {:?}", self.task_id, e);
                    *errors = true;
                    return Some(i);
                }
            }
        }
        None
    }
}
