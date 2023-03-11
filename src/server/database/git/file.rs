//! Types for files in different repositories

#[derive(Debug)]
pub struct GitPath<'a>(&'a str);

impl GitPath<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for GitPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct LiveVersionPath<'a>(&'a str);

impl LiveVersionPath<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for LiveVersionPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct TmpPath<'a>(&'a str);

impl TmpPath<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for TmpPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Get file name which is used for committing this file to Git.
pub trait GetGitPath {
    /// Path relative to git repository root.
    fn git_path(&self) -> GitPath<'static>;
}

/// Get file name for file which is ment to be consumed in web requests.
pub trait GetLiveVersionPath {
    /// Path relative to git repository root.
    fn live_path(&self) -> LiveVersionPath<'static>;
}

/// Get file name which is used for creating tmp files which are only used
/// when modifying files with no version history.
pub trait GetTmpPath {
    /// Path relative to git repository root.
    fn tmp_path(&self) -> TmpPath<'static>;
}

pub trait GetReplaceMessage {
    fn commit_message_for_replace(&self) -> &'static str;
}

/// Files in profile repository
#[derive(Debug, Clone, Copy)]
pub enum CoreFile {
    /// Plain text containing profile ID
    Id,

    /// JSON text file for public profile.
    ProfileJson,

    /// JSON text file for private account state.
    AccountStateJson,

    /// JSON text file for private account data which does not change
    /// after initial setup.
    AccountSetupJson,
}

impl GetGitPath for CoreFile {
    fn git_path(&self) -> GitPath<'static> {
        GitPath(match self {
            Self::Id => "id.txt.git",
            Self::ProfileJson => "profile.txt.git",
            Self::AccountStateJson => "account.txt.git",
            Self::AccountSetupJson => "account_setup.txt.git",
        })
    }
}

impl GetLiveVersionPath for CoreFile {
    fn live_path(&self) -> LiveVersionPath<'static> {
        LiveVersionPath(match self {
            Self::Id => "id.txt",
            Self::ProfileJson => "profile.txt",
            Self::AccountStateJson => "account.txt",
            Self::AccountSetupJson => "account_setup.txt",
        })
    }
}

impl GetReplaceMessage for CoreFile {
    fn commit_message_for_replace(&self) -> &'static str {
        match self {
            Self::Id => "Update id.txt.git",
            Self::ProfileJson => "Update profile.txt.git",
            Self::AccountStateJson => "Update account.txt.git",
            Self::AccountSetupJson => "Update account_setup.txt.git",
        }
    }
}

/// Files not in version history but in profile history.
#[derive(Debug, Clone, Copy)]
pub enum CoreFileNoHistory {
    /// Plain text containing API key
    ApiToken,
}

impl GetTmpPath for CoreFileNoHistory {
    fn tmp_path(&self) -> TmpPath<'static> {
        TmpPath(match self {
            Self::ApiToken => "api_key.txt.tmp",
        })
    }
}

impl GetLiveVersionPath for CoreFileNoHistory {
    fn live_path(&self) -> LiveVersionPath<'static> {
        LiveVersionPath(match self {
            Self::ApiToken => "api_key.txt",
        })
    }
}

// TODO: Append only files (possibly for IP addresses). Set max limit for ip
// address changes or something?
