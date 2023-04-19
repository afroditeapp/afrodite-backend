




use super::{utils::FileDir};


pub struct FileReadCommands<'a> {
    dir: &'a FileDir,
}

impl<'a> FileReadCommands<'a> {
    pub fn new(dir: &'a FileDir) -> Self {
        Self { dir }
    }
}
