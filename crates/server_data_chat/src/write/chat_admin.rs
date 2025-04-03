use server_data::define_cmd_wrapper_write;

mod public_key;

define_cmd_wrapper_write!(WriteCommandsChatAdmin);

impl<'a> WriteCommandsChatAdmin<'a> {
    pub fn public_key(self) -> public_key::WriteCommandsChatAdminPublicKey<'a> {
        public_key::WriteCommandsChatAdminPublicKey::new(self.0)
    }
}
