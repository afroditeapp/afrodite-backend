//! Types for files in different repositories


/// Files in profile repository
#[derive(Debug)]
pub enum CoreFile {
    /// Plain text containing profile ID
    Id,

    /// JSON text file
    ProfileJson,

    /// JSON text file
    PrivateUserInfoJson,
}

impl GitRepositoryPath for CoreFile {
    fn relative_path(&self) -> &str {
        match self {
            Self::Id => "id.txt",
            Self::ProfileJson => "profile.txt",
            Self::PrivateUserInfoJson => "user.txt",
        }
    }
}


pub trait GitRepositoryPath {
    // Get path relative to git repository root. This is relative path.
    fn relative_path(&self) -> &str;
}

// TODO: Append only files,
// TODO: Files w
