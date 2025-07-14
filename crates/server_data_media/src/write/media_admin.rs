use server_data::define_cmd_wrapper_write;

pub mod content;
mod notification;

define_cmd_wrapper_write!(WriteCommandsMediaAdmin);

// TODO(prod): Move event sending to WriteCommands or db_write_multiple instead
// directly to route handlers to avoid disappearing events in case client
// disconnects before event is sent.
// Update: Change EventManagerProvider to only give access to functionality
// which should not be moved to WriteCommands or db_write_multiple.

impl<'a> WriteCommandsMediaAdmin<'a> {
    pub fn content(self) -> content::WriteCommandsProfileAdminContent<'a> {
        content::WriteCommandsProfileAdminContent::new(self.0)
    }
    pub fn notification(self) -> notification::WriteCommandsMediaAdminNotification<'a> {
        notification::WriteCommandsMediaAdminNotification::new(self.0)
    }
}
