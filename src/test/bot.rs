mod actions;
mod benchmark;
mod utils;

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant}, fmt::Debug,
};

use api_client::{
    models::{AccountIdLight, Profile},
};

use async_trait::async_trait;
use tokio::{
    select,
    sync::{mpsc, watch},
    time::sleep,
};

use error_stack::{Result, ResultExt};

use tracing::{error, log::warn};

use self::{benchmark::{Benchmark, BenchmarkState}, actions::{BotAction, DoNothing}, utils::{Counters, Timer}};

use super::client::{ApiClient, TestError};

use crate::{
    api::model::{AccountId},
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

#[derive(Debug)]
pub struct BotState {
    pub id: Option<AccountIdLight>,
    pub config: Arc<TestMode>,
    pub task_id: u32,
    pub bot_id: u32,
    pub api: ApiClient,
    pub previous_action: &'static dyn BotAction,
    pub benchmark: BenchmarkState,
}

impl BotState {
    pub fn new(
        id: Option<AccountIdLight>, config: Arc<TestMode>, task_id: u32, bot_id: u32, api: ApiClient
    ) -> Self { Self { id, config, task_id, bot_id, api, benchmark: BenchmarkState::new(), previous_action: &DoNothing } }

    pub fn id(&self) -> Result<AccountIdLight, TestError> {
        self.id.ok_or(TestError::AccountIdMissing.into())
    }

    pub fn id_string(&self) -> Result<String, TestError> {
        self.id.ok_or(TestError::AccountIdMissing.into()).map(|id| id.to_string())
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

    async fn run_action(&mut self) -> Result<Option<Completed>, TestError> {
        self.run_action_impl()
            .await
            .attach_printable_lazy(|| format!("{:?}", self))
    }

    async fn run_action_impl(&mut self) -> Result<Option<Completed>, TestError> {
        match self.next_action_and_state() {
            (None, _) => Ok(Some(Completed)),
            (Some(action), state) => {
                let result = action.excecute(state).await.map(|_| None);
                state.previous_action = action;
                result
            }
        }
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
            Test::Normal | Test::Default =>
                Self::benchmark(task_id, id, config, _bot_running_handle),
        };

        tokio::spawn(bot.run(bot_quit_receiver));
    }

    pub fn benchmark(task_id: u32, id: Option<AccountIdLight>, config: Arc<TestMode>, _bot_running_handle: mpsc::Sender<()>) -> Self {
        let mut bots = Vec::<Box<dyn BotStruct>>::new();
        for bot_i in 0..config.bot_count {
            let state = BotState::new(id, config.clone(), task_id, bot_i, ApiClient::new(config.server.api_urls.clone()));
            let benchmark = match config.test {
                Test::Normal =>
                    Benchmark::get_profile_benchmark(state),
                Test::Default =>
                    Benchmark::get_default_profile_benchmark(state),
            };
            bots.push(Box::new(benchmark))
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
                result = self.run_bot() => {
                    if let Err(e) = result {
                        error!("Task {} returned error: {:?}", self.task_id, e);
                    }
                    break;
                }
            }
        }
    }

    async fn run_bot(&mut self) -> Result<(), TestError> {
        loop {
            if self.bots.is_empty() {
                return Ok(());
            }

            if let Some(remove_i) = self.iter_bot_list().await {
                self.bots.swap_remove(remove_i);
            }
        }
    }

    /// If Some(bot_index) is returned remove the bot.
    async fn iter_bot_list(&mut self) -> Option<usize> {
        for (i, b) in self.bots.iter_mut().enumerate() {
            match b.run_action().await {
                Ok(None) => (),
                Ok(Some(Completed)) => return Some(i),
                Err(e) => {
                    error!("Taks {}, bot returned error: {}", self.task_id, e);
                    return Some(i);
                }
            }
        }
        None
    }
}
