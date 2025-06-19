pub mod attribute_values;
pub mod profile_name_allowlist;

#[derive(thiserror::Error, Debug)]
pub enum CsvFileError {
    #[error("Loading CSV file failed")]
    Load,
    #[error("Selected column does not exist")]
    SelectedColumnDoesNotExists,
    #[error("Delimiter character is unsupported")]
    UnsupportedDelimiterCharacter,
    #[error("Invalid config")]
    InvalidConfig,
}
