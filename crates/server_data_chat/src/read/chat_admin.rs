use server_data::define_cmd_wrapper_read;

mod report;

define_cmd_wrapper_read!(ReadCommandsChatAdmin);

impl<'a> ReadCommandsChatAdmin<'a> {
    pub fn report(self) -> report::ReadCommandsChatReport<'a> {
        report::ReadCommandsChatReport::new(self.0)
    }
}
