use crate::api::core::profile::{RegisterBody, RegisterResponse, LoginBody, LoginResponse};

use super::{DatabaseTask, DatabaseMessage};

pub trait DatabaseCommand {
    type Response;
}

// Command implementations

impl DatabaseCommand for RegisterBody {
    type Response = RegisterResponse;
}

impl From<DatabaseTask<RegisterBody>> for DatabaseMessage {
    fn from(task: DatabaseTask<RegisterBody>) -> Self {
        DatabaseMessage::QueueRegister(task)
    }
}

impl DatabaseCommand for LoginBody {
    type Response = LoginResponse;
}

impl From<DatabaseTask<LoginBody>> for DatabaseMessage {
    fn from(task: DatabaseTask<LoginBody>) -> Self {
        DatabaseMessage::QueueLogin(task)
    }
}
