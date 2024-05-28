
#[derive(thiserror::Error, Debug)]
pub enum IndexError {
    #[error("Profile location index error")]
    ProfileIndex,
    // TODO: more detailed errors
}
