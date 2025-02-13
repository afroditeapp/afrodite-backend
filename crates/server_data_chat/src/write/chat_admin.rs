use server_data::define_cmd_wrapper_write;

mod report;

define_cmd_wrapper_write!(WriteCommandsChatAdmin);

impl<'a> WriteCommandsChatAdmin<'a> {
    pub fn report(self) -> report::WriteCommandsChatReport<'a> {
        report::WriteCommandsChatReport::new(self.0)
    }
}
