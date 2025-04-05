use crate::state::BotEncryptionKeys;

#[derive(Debug, Default)]
pub struct ChatState {
    pub keys: Option<BotEncryptionKeys>,
}
