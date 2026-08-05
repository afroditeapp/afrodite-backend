use error_stack::IntoReport;
use simple_backend_utils::ComponentError;

impl ComponentError for CacheError {
    const COMPONENT_NAME: &'static str = "Cache";
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Key already exists")]
    AlreadyExists,

    #[error("Key not exists")]
    KeyNotExists,

    #[error("Data is not in cache")]
    NotInCache,

    #[error("Cache init error")]
    Init,
}

impl CacheError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        self.into_report()
    }

    #[track_caller]
    pub fn error<Ok>(self) -> simple_backend_utils::Result<Ok, Self> {
        Err(self.into_report())
    }
}
