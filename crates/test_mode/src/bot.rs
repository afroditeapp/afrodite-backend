pub mod actions;
mod benchmark;
mod client_bot;
mod utils;

use std::{fmt::Debug, sync::Arc, vec};

use api_client::models::{AccountId, EventToClient};
use async_trait::async_trait;
use config::{
    args::{SelectedBenchmark, TestMode, TestModeSubMode}, bot_config_file::{self, BotConfigFile}, Config
};
use error_stack::{Result, ResultExt};
use tokio::{
    net::TcpStream,
    select,
    sync::{broadcast, mpsc, watch}, task::JoinHandle,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, info, log::warn};

use self::{
    actions::{media::MediaState, BotAction, DoNothing, PreviousValue},
    benchmark::{Benchmark, BenchmarkState},
    client_bot::ClientBot,
};
use super::{
    client::{ApiClient, TestError},
    state::{BotPersistentState, StateData},
};

#[derive(Debug, Default)]
pub struct TaskState;

pub fn create_event_channel(enable_event_sending: bool) -> (EventSenderAndQuitWatcher, EventReceiver, broadcast::Sender<()>) {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let (quit_handle, quit_watcher) = broadcast::channel(1);
    (
        EventSenderAndQuitWatcher {
            event_sender: EventSender {
                enable_event_sending,
                event_sender,
            },
            quit_watcher,
        },
        EventReceiver { event_receiver },
        quit_handle,
    )
}

#[derive(Debug, Clone)]
pub struct EventSender {
    enable_event_sending: bool,
    event_sender: mpsc::UnboundedSender<EventToClient>,
}

impl EventSender {
    pub async fn send_if_sending_enabled(&self, event: EventToClient) {
        if self.enable_event_sending {
            let _ = self.event_sender.send(event);
        }
    }
}

#[derive(Debug)]
pub struct EventSenderAndQuitWatcher {
    pub event_sender: EventSender,
    pub quit_watcher: broadcast::Receiver<()>,
}

impl Clone for EventSenderAndQuitWatcher {
    fn clone(&self) -> Self {
        Self {
            event_sender: self.event_sender.clone(),
            quit_watcher: self.quit_watcher.resubscribe(),
        }
    }
}

#[derive(Debug)]
pub struct EventReceiver {
    event_receiver: mpsc::UnboundedReceiver<EventToClient>,
}

impl EventReceiver {
    pub async fn recv(&mut self) -> Option<EventToClient> {
        self.event_receiver.recv().await
    }
}

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct WsConnection {
    task: JoinHandle<()>,
}

impl WsConnection {
    /// Close EventReceiver before calling this.
    pub async fn close(self) {
        let _ = self.task.await;
    }
}

#[derive(Debug)]
pub struct AccountConnections {
    pub account: Option<WsConnection>,
    pub profile: Option<WsConnection>,
    pub media: Option<WsConnection>,
    /// Drop this to close all WebSockets
    pub quit_handle: broadcast::Sender<()>,
}

impl AccountConnections {
    pub async fn close(mut self) {
        drop(self.quit_handle);
        if let Some(account) = self.account.take() {
            let _ = account.close().await;
        }
        if let Some(profile) = self.profile.take() {
            let _ = profile.close().await;
        }
        if let Some(media) = self.media.take() {
            let _ = media.close().await;
        }
    }
}

#[derive(Debug, Default)]
pub struct BotConnections {
    /// If this true before `Login` action is exceuted, the connection
    /// events will be sent to event channel.
    pub enable_event_sending: bool,
    connections: Option<AccountConnections>,
    events: Option<EventReceiver>,
}

impl BotConnections {
    pub fn unwrap_account_connections(&mut self) -> AccountConnections {
        self.connections.take().expect("Account connections are missing")
    }

    /// Wait event if event sending is enabled or timeout after 5 seconds
    pub async fn wait_event(&mut self, check: impl Fn(&EventToClient) -> bool) -> Result<(), TestError> {
        if !self.enable_event_sending {
            return Ok(());
        }

        let events = self
            .events
            .as_mut()
            .ok_or(TestError::EventReceivingHandleMissing.report())?;

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => Err(TestError::EventReceivingTimeout.report()),
            event_or_error = wait_until_specific_event(check, events) => event_or_error,
        }
    }
}

async fn wait_until_specific_event(check: impl Fn(&EventToClient) -> bool, events: &mut EventReceiver) -> Result<(), TestError> {
    loop {
        let event = events.recv().await.ok_or(TestError::EventChannelClosed.report())?;
        if check(&event) {
            return Ok(());
        }
    }
}

#[derive(Debug)]
pub struct BotState {
    pub id: Option<AccountId>,
    pub server_config: Arc<Config>,
    pub config: Arc<TestMode>,
    pub bot_config_file: Arc<BotConfigFile>,
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
        id: Option<AccountId>,
        server_config: Arc<Config>,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        task_id: u32,
        bot_id: u32,
        api: ApiClient,
    ) -> Self {
        Self {
            id,
            server_config,
            config,
            bot_config_file,
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

    /// Wait event if event sending enabled or timeout after 5 seconds
    pub async fn wait_event(&mut self, check: impl Fn(&EventToClient) -> bool) -> Result<(), TestError> {
        self.connections.wait_event(check).await
    }

    pub fn account_id(&self) -> Result<AccountId, TestError> {
        self.id.ok_or(TestError::AccountIdMissing.report())
    }

    pub fn account_id_string(&self) -> Result<String, TestError> {
        self.id
            .ok_or(TestError::AccountIdMissing.report())
            .map(|id| id.to_string())
    }

    pub fn is_first_bot(&self) -> bool {
        self.task_id == 0 && self.bot_id == 0
    }

    pub fn print_info(&mut self) -> bool {
        self.is_first_bot() && self.benchmark.print_info_timer.passed()
    }

    pub fn persistent_state(&self) -> Option<BotPersistentState> {
        if let Some(id) = self.id {
            Some(BotPersistentState {
                account_id: id.account_id,
                task: self.task_id,
                bot: self.bot_id,
            })
        } else {
            None
        }
    }

    /// Is current bot an admin bot.
    ///
    /// Admin bot is detected from bot ID. First `n` bots are user bots
    /// and the rest are admin bots.
    pub fn is_admin_bot(&self) -> bool {
        self
            .config
            .bot_mode()
            .map(|bot_config| self.bot_id < bot_config.admins)
            .unwrap_or(false)
    }
}

/// Bot completed
pub struct Completed;

#[async_trait]
pub trait BotStruct: Debug + Send + 'static {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState);
    fn next_action(&mut self);
    fn state(&self) -> &BotState;

    async fn run_action(
        &mut self,
        task_state: &mut TaskState,
    ) -> Result<Option<Completed>, TestError> {
        let mut result = self.run_action_impl(task_state).await;
        if self.state().config.qa_mode().is_some() {
            result = result.attach_printable_lazy(|| format!("{:?}", self.state().action_history))
        }
        result.attach_printable_lazy(|| format!("{:?}", self))
    }

    async fn run_action_impl(
        &mut self,
        task_state: &mut TaskState,
    ) -> Result<Option<Completed>, TestError> {
        match self.peek_action_and_state() {
            (None, _) => Ok(Some(Completed)),
            (Some(action), state) => {
                let result = action.excecute(state, task_state).await;

                let result = match result {
                    Err(e) if e.current_context() == &TestError::BotIsWaiting => return Ok(None),
                    Err(e) => Err(e),
                    Ok(()) => Ok(None),
                };

                state.previous_action = action;
                if state.config.qa_mode().is_some() {
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
    _bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
    task_id: u32,
    config: Arc<TestMode>,
}

impl BotManager {
    pub fn spawn(
        task_id: u32,
        server_config: Arc<Config>,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        old_state: Option<Arc<StateData>>,
        bot_quit_receiver: watch::Receiver<()>,
        _bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
    ) {
        let bot = match config.mode {
            TestModeSubMode::Benchmark(_) | TestModeSubMode::Bot(_) => Self::benchmark_or_bot(
                task_id,
                old_state,
                server_config,
                bot_config_file,
                config,
                _bot_running_handle,
            ),
            TestModeSubMode::Qa(_) => panic!("Server tests use different test runner"),
        };

        tokio::spawn(bot.run(bot_quit_receiver));
    }

    pub fn benchmark_or_bot(
        task_id: u32,
        old_state: Option<Arc<StateData>>,
        server_config: Arc<Config>,
        bot_config_file: Arc<BotConfigFile>,
        config: Arc<TestMode>,
        _bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
    ) -> Self {
        let mut bots = Vec::<Box<dyn BotStruct>>::new();
        for bot_i in 0..config.bots() {
            let state = BotState::new(
                old_state
                    .as_ref()
                    .map(|d| {
                        d.find_matching(task_id, bot_i)
                            .map(|s| AccountId::new(s.account_id))
                    })
                    .flatten(),
                server_config.clone(),
                config.clone(),
                bot_config_file.clone(),
                task_id,
                bot_i,
                ApiClient::new(config.server.api_urls.clone()),
            );

            match (config.selected_benchmark(), config.bot_mode()) {
                (Some(benchmark), _) => match benchmark {
                    SelectedBenchmark::GetProfile => {
                        bots.push(Box::new(Benchmark::benchmark_get_profile(state)))
                    }
                    SelectedBenchmark::GetProfileFromDatabase => bots.push(Box::new(
                        Benchmark::benchmark_get_profile_from_database(state),
                    )),
                    SelectedBenchmark::GetProfileList => {
                        let benchmark = match bot_i {
                            0 => Benchmark::benchmark_get_profile_list(state),
                            _ => Benchmark::benchmark_get_profile_list_bot(state),
                        };
                        bots.push(Box::new(benchmark))
                    }
                    SelectedBenchmark::PostProfile => {
                        bots.push(Box::new(Benchmark::benchmark_post_profile(state)))
                    }
                    SelectedBenchmark::PostProfileToDatabase => bots.push(Box::new(
                        Benchmark::benchmark_post_profile_to_database(state),
                    )),
                },
                (_, Some(_)) => bots.push(Box::new(ClientBot::new(state))),
                test_config => panic!("Invalid test config {:?}", test_config),
            };
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

        let data = self.iter_persistent_state();
        self._bot_running_handle.send(data).await.unwrap();
    }

    fn iter_persistent_state(&self) -> Vec<BotPersistentState> {
        self.bots
            .iter()
            .filter_map(|bot| bot.state().persistent_state())
            .collect()
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

            if let Some(bot_mode_config) = self.config.bot_mode() {
                if !bot_mode_config.no_sleep {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// If Some(bot_index) is returned remove the bot.
    async fn iter_bot_list(
        &mut self,
        errors: &mut bool,
        task_state: &mut TaskState,
    ) -> Option<usize> {
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
