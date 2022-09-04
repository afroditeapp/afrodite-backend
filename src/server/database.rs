pub mod git;
pub mod command;


use std::{sync::Arc, path::Path};

use tokio::{sync::{oneshot, mpsc}, task::JoinHandle};

use crate::{config::Config, utils::{QuitSender, QuitReceiver}, api::core::profile::{LoginBody, LoginResponse}};

use self::{git::GitDatabase, command::DatabaseCommand};

use crate::api::core::profile::{RegisterBody, RegisterResponse};

pub type DatabaseTaskResult<T> = Result<T, DatabaseTaskError>;

#[derive(Debug)]
pub enum DatabaseTaskError {
    UnknownError
}

#[derive(Debug)]
pub struct DatabaseTask<C: DatabaseCommand> {
    result_sender: oneshot::Sender<DatabaseTaskResult<C::Response>>,
    command: C,
}

#[derive(Debug)]
pub enum DatabaseMessage {
    QueueRegister(DatabaseTask<RegisterBody>),
    QueueLogin(DatabaseTask<LoginBody>),
}

#[derive(Debug, Clone)]
pub struct DatabaseTaskSender {
    sender: mpsc::Sender<DatabaseMessage>,
}

impl DatabaseTaskSender {
    pub async fn send_command<C: DatabaseCommand>(
        &mut self,
        command: C,
    ) -> oneshot::Receiver<DatabaseTaskResult<C::Response>>
    where DatabaseMessage: From<DatabaseTask<C>> {
        let (result_sender, result_receiver) = oneshot::channel();

        let task: DatabaseTask<C> = DatabaseTask {
            result_sender,
            command,
        };

        // TODO: make sure that this is not called after DatabaseManager is closed.
        self.sender.send(task.into()).await.unwrap();

        result_receiver
    }
}


pub struct DatabaseManager {
    config: Arc<Config>,
    receiver: mpsc::Receiver<DatabaseMessage>,
    sender: mpsc::Sender<DatabaseMessage>,
}

impl DatabaseManager {
    pub fn start_task(
        config: Arc<Config>,
        sender: mpsc::Sender<DatabaseMessage>,
        receiver: mpsc::Receiver<DatabaseMessage>,
    ) -> (JoinHandle<()>, QuitSender, DatabaseTaskSender) {
        let database_task_sender = DatabaseTaskSender { sender: sender.clone() };
        let (quit_sender, quit_receiver) = oneshot::channel();

        let dm = Self {
            sender,
            receiver,
            config,
        };

        let task = async move {
            dm.run(quit_receiver).await;
        };

        (tokio::spawn(task), quit_sender, database_task_sender)
    }

    pub fn database_task_sender(&self) -> DatabaseTaskSender {
        DatabaseTaskSender { sender: self.sender.clone() }
    }

    /// Run device manager logic.
    pub async fn run(mut self, mut quit_receiver: QuitReceiver) {
        loop {
            tokio::select! {
                result = &mut quit_receiver => break result.unwrap(),
                event = self.receiver.recv() => {
                    tokio::select! {
                        result = &mut quit_receiver => break result.unwrap(),
                        _ = self.handle_message(event.unwrap()) => (),
                    };
                }
            }
        }

        // Quit
    }

    async fn handle_message(&mut self, event: DatabaseMessage) {
        match event {
            DatabaseMessage::QueueRegister(profile) => {
                let mut git = GitDatabase::create(&self.config.database_dir, "id123").unwrap();

                let mut test = self.config.database_dir.to_owned();
                test.push("profile.json");
                std::fs::File::create(&test).unwrap();


                git.commit(Path::new("profile.json"), "Just a test").unwrap();

                let _ = profile.result_sender.send(
                    Ok(RegisterResponse::success("test-from-database".to_string()))
                );
            }
            DatabaseMessage::QueueLogin(profile) => {
                let _ = profile.result_sender.send(
                    Ok(LoginResponse::database_error())
                );
            }
        }
    }
}
