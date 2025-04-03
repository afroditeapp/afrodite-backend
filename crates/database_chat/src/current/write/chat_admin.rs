use database::define_current_write_commands;

mod public_key;

define_current_write_commands!(CurrentWriteChatAdmin);

impl<'a> CurrentWriteChatAdmin<'a> {
    pub fn public_key(self) -> public_key::CurrentWriteChatAdminPublicKey<'a> {
        public_key::CurrentWriteChatAdminPublicKey::new(self.cmds)
    }
}
