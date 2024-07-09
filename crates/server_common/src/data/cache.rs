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

    #[error("Cache operation failed because of server feature was not enabled")]
    FeatureNotEnabled,
}

impl CacheError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        error_stack::report!(self)
    }

    #[track_caller]
    pub fn error<Ok>(self) -> error_stack::Result<Ok, Self> {
        Err(error_stack::report!(self))
    }
}
