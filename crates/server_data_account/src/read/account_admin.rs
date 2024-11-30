use server_data::define_cmd_wrapper_read;

pub mod news;

define_cmd_wrapper_read!(ReadCommandsAccountAdmin);

impl<'a> ReadCommandsAccountAdmin<'a> {
    pub fn news(self) -> news::ReadCommandsAccountNewsAdmin<'a> {
        news::ReadCommandsAccountNewsAdmin::new(self.0)
    }
}
