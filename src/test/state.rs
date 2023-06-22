//! Save and load state
//!

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StateData {
    pub test_name: String,
    pub bot_states: Vec<BotPersistentState>,
}

impl StateData {
    pub fn find_matching(&self, task: u32, bot: u32) -> Option<&BotPersistentState> {
        self.bot_states.iter().find(|s| s.task == task && s.bot == bot)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotPersistentState {
    pub account_id: uuid::Uuid,
    pub task: u32,
    pub bot: u32,
}