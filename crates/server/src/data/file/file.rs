//! Types related to files

use model::ContentSlot;

#[derive(Debug)]
pub struct StaticFileName<'a>(&'a str);

impl StaticFileName<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for StaticFileName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub trait GetStaticFileName {
    fn file_name(&self) -> StaticFileName<'static>;
}

impl GetStaticFileName for ContentSlot {
    fn file_name(&self) -> StaticFileName<'static> {
        StaticFileName(match self {
            Self::Content0 => "slot0.jpg",
            Self::Content1 => "slot1.jpg",
            Self::Content2 => "slot2.jpg",
            Self::Content3 => "slot3.jpg",
            Self::Content4 => "slot4.jpg",
            Self::Content5 => "slot5.jpg",
            Self::Content6 => "slot6.jpg",
        })
    }
}

// TODO: Set max limit for IP
// address changes or something (limit IP address history size)?
