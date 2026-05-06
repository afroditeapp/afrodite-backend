use database::define_current_write_commands;

mod moderation;
mod search;
mod verification;

define_current_write_commands!(CurrentWriteProfileAdmin);

impl<'a> CurrentWriteProfileAdmin<'a> {
    pub fn moderation(self) -> moderation::CurrentWriteProfileAdminModeration<'a> {
        moderation::CurrentWriteProfileAdminModeration::new(self.cmds)
    }
    pub fn search(self) -> search::CurrentWriteProfileAdminSearch<'a> {
        search::CurrentWriteProfileAdminSearch::new(self.cmds)
    }

    pub fn verification(self) -> verification::CurrentWriteProfileAdminVerification<'a> {
        verification::CurrentWriteProfileAdminVerification::new(self.cmds)
    }
}
