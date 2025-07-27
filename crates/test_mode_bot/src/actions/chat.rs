use test_mode_utils::state::BotEncryptionKeys;

#[derive(Debug, Default)]
pub struct ChatState {
    pub keys: Option<BotEncryptionKeys>,
}
