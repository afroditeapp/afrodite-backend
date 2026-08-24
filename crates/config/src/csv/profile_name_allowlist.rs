use error_stack::ResultExt;
use simple_backend_utils::{ContextExt, Result};

use super::CsvFileError;
use crate::file::ProfileNameAllowlistConfig;

#[derive(Debug, Default)]
pub struct ProfileNameAllowlistBuilder {
    names: Vec<String>,
}

impl ProfileNameAllowlistBuilder {
    pub fn load(&mut self, config: &ProfileNameAllowlistConfig) -> Result<(), CsvFileError> {
        let delimiter: u8 = TryInto::<u8>::try_into(config.delimiter)
            .change_context(CsvFileError::UnsupportedDelimiterCharacter)
            .attach(config.delimiter.to_string())?;

        let r = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(delimiter)
            .from_path(&config.csv_file)
            .change_context(CsvFileError::Load)
            .attach(format!("File: {}", config.csv_file.display(),))?;

        let name_rows = r.into_records().skip(config.start_row_index);

        for r in name_rows {
            let r = r.change_context(CsvFileError::Load)?;
            let name = r
                .get(config.column_index)
                .ok_or(CsvFileError::SelectedColumnDoesNotExists.report())
                .attach(format!(
                    "File: {}, Column: {}",
                    config.csv_file.display(),
                    config.column_index,
                ))?
                .trim()
                .to_lowercase();
            self.names.push(name);
        }

        Ok(())
    }

    pub fn build(mut self) -> ProfileNameAllowlistData {
        self.names.sort_unstable();
        ProfileNameAllowlistData { names: self.names }
    }
}

/// Sorted list of lowercase and trimmed profile names.
#[derive(Debug, Default)]
pub struct ProfileNameAllowlistData {
    names: Vec<String>,
}

impl ProfileNameAllowlistData {
    pub fn name_exists(&self, name: &str) -> bool {
        self.names
            .binary_search_by(|list_name| list_name.as_str().cmp(name))
            .is_ok()
    }
}
