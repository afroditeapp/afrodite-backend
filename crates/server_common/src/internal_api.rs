#[derive(thiserror::Error, Debug)]
pub enum InternalApiError {
    #[error("API request failed")]
    ApiRequest,

    #[error("Database call failed")]
    DataError,

    #[error("Account API URL not configured")]
    AccountApiUrlNotConfigured,

    #[error("Media API URL not configured")]
    MediaApiUrlNotConfigured,
    // #[error("Wrong status code")]
    // StatusCode,

    // #[error("Joining text to URL failed")]
    // ApiUrlJoinError,
    #[error("Missing value")]
    MissingValue,

    #[error("Invalid value")]
    InvalidValue,

    #[error("Required server component is not enabled")]
    MissingComponent,
}
