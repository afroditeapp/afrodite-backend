





use super::utils::FileDir;





pub struct FileWriteCommands<'a> {
    dir: &'a FileDir,
}

impl<'a> FileWriteCommands<'a> {
    pub fn new(dir: &'a FileDir) -> Self {
        Self { dir }
    }
}
