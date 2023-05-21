mod actions;
mod benchmark;
mod qa;
mod utils;

use std::{fmt::Debug, sync::Arc, vec};

use api_client::models::AccountIdLight;

use async_trait::async_trait;
use tokio::{
    select,
    sync::{mpsc, watch}, net::TcpStream,
};

use error_stack::{Result, ResultExt};

use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tracing::{error, info, log::warn};

use self::{
    actions::{media::MediaState, BotAction, DoNothing, PreviousValue},
    benchmark::{Benchmark, BenchmarkState},
    qa::Qa,
};

use super::client::{ApiClient, TestError};

use crate::config::args::{Test, TestMode};

#[derive(Debug, Default)]
pub struct TaskState {
    pub bot_count_update_location_to_lat_lon_10: u64,
}

pub type WsConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Default)]
pub struct BotConnections {
    account: Option<WsConnection>,
    profile: Option<WsConnection>,
    media: Option<WsConnection>,
}

#[derive(Debug)]
pub struct BotState {
    pub id: Option<AccountIdLight>,
    pub config: Arc<TestMode>,
    pub task_id: u32,
    pub bot_id: u32,
    pub api: ApiClient,
    pub previous_action: &'static dyn BotAction,
    pub previous_value: PreviousValue,
    pub action_history: Vec<&'static dyn BotAction>,
    pub benchmark: BenchmarkState,
    pub media: MediaState,
    pub connections: BotConnections,
    pub refresh_token: Option<Vec<u8>>,
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
            previous_value: PreviousValue::Empty,
            action_history: vec![],
            media: MediaState::new(),
            connections: BotConnections::default(),
            refresh_token: None,
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

/// Bot completed
pub struct Completed;

#[async_trait]
pub trait BotStruct: Debug + Send + 'static {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState);
    fn next_action(&mut self);
    fn state(&self) -> &BotState;

    async fn run_action(&mut self, task_state: &mut TaskState) -> Result<Option<Completed>, TestError> {
        let mut result = self.run_action_impl(task_state).await;
        if let Test::Qa = self.state().config.test {
            result = result.attach_printable_lazy(|| format!("{:?}", self.state().action_history))
        }
        result.attach_printable_lazy(|| format!("{:?}", self))
    }

    async fn run_action_impl(&mut self, task_state: &mut TaskState) -> Result<Option<Completed>, TestError> {
        match self.peek_action_and_state() {
            (None, _) => Ok(Some(Completed)),
            (Some(action), state) => {
                let result = action.excecute(state, task_state).await;

                let result = match result {
                    Err(e) if e.current_context() == &TestError::BotIsWaiting => return Ok(None),
                    Err(e) => Err(e),
                    Ok(()) => Ok(None)
                };

                state.previous_action = action;
                if let Test::Qa = state.config.test {
                    state.action_history.push(action)
                }
                self.next_action();
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
    config: Arc<TestMode>,
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
            Test::BenchmarkGetProfileList | Test::BenchmarkGetProfile => {
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

        match config.test {
            Test::BenchmarkGetProfile => {
                for bot_i in 0..config.bot_count {
                    let state = BotState::new(
                        id,
                        config.clone(),
                        task_id,
                        bot_i,
                        ApiClient::new(config.server.api_urls.clone()),
                    );
                    bots.push(Box::new(Benchmark::benchmark_get_profile(state)))
                }
            },
            Test::BenchmarkGetProfileList => {
                for bot_i in 0..config.bot_count {
                    let state = BotState::new(
                        id,
                        config.clone(),
                        task_id,
                        bot_i,
                        ApiClient::new(config.server.api_urls.clone()),
                    );
                    let benchmark = match bot_i {
                        0 => Benchmark::benchmark_get_profile_list(state),
                        _ => Benchmark::benchmark_get_profile_list_bot(state),
                    };
                    bots.push(Box::new(benchmark))
                }
            },
            _ => panic!("Invalid test {:?}", config.test),
        };

        Self {
            bots,
            _bot_running_handle,
            task_id,
            config,
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
            config,
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
        let mut task_state: TaskState = TaskState::default();
        loop {
            if self.config.early_quit && errors {
                error!("Error occurred.");
                return;
            }

            if self.bots.is_empty() {
                if errors {
                    error!("All bots closed. Errors occurred.");
                } else {
                    info!("All bots closed. No errors.");
                }
                return;
            }

            if let Some(remove_i) = self.iter_bot_list(&mut errors, &mut task_state).await {
                self.bots
                    .swap_remove(remove_i)
                    .notify_task_bot_count_decreased(self.bots.len());
            }
        }
    }

    /// If Some(bot_index) is returned remove the bot.
    async fn iter_bot_list(&mut self, errors: &mut bool, task_state: &mut TaskState) -> Option<usize> {
        for (i, b) in self.bots.iter_mut().enumerate() {
            match b.run_action(task_state).await {
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
