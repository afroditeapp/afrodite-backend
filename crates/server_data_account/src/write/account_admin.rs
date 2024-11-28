use server_data::define_cmd_wrapper_write;

mod news;

define_cmd_wrapper_write!(WriteCommandsAccountAdmin);

impl<'a> WriteCommandsAccountAdmin<'a> {
    pub fn news(self) -> news::WriteCommandsAccountNewsAdmin<'a> {
        news::WriteCommandsAccountNewsAdmin::new(self.0)
    }
}
