use std::collections::HashSet;

use error_stack::{Result, ResultExt};
use model_server_data::{
    AttributeInternal, AttributesFileInternal, GroupValuesInternal, Language, Translation,
};
use sha2::{Sha256, digest::Update};
use simple_backend_utils::ContextExt;

use super::CsvFileError;

#[derive(Debug, Default)]
pub struct AttributeValuesCsvLoader;

impl AttributeValuesCsvLoader {
    pub fn load_if_needed(
        file: &mut AttributesFileInternal,
        attributes_file_sha256: &mut Sha256,
    ) -> Result<(), CsvFileError> {
        for a in &mut file.attribute {
            Self::handle_attribute(a, attributes_file_sha256)?;
        }

        Ok(())
    }

    fn handle_attribute(
        a: &mut AttributeInternal,
        attributes_file_sha256: &mut Sha256,
    ) -> Result<(), CsvFileError> {
        let Some(config) = &a.group_values_csv else {
            return Ok(());
        };

        let csv_string =
            std::fs::read_to_string(&config.csv_file).change_context(CsvFileError::Load)?;
        attributes_file_sha256.update(csv_string.as_bytes());

        if !a.values.is_empty() {
            return Err(CsvFileError::InvalidConfig.report()).attach_printable(format!(
                "Attribute ID {} values must be empty when group_values_csv is defined",
                a.id.to_usize()
            ));
        }
        if !a.group_values.is_empty() {
            return Err(CsvFileError::InvalidConfig.report()).attach_printable(format!(
                "Attribute ID {} group_values must be empty when group_values_csv is defined",
                a.id.to_usize()
            ));
        }
        if !a.translations.is_empty() {
            return Err(CsvFileError::InvalidConfig.report()).attach_printable(format!(
                "Attribute ID {} translations must be empty when group_values_csv is defined",
                a.id.to_usize()
            ));
        }

        let delimiter: u8 = TryInto::<u8>::try_into(config.delimiter)
            .change_context(CsvFileError::UnsupportedDelimiterCharacter)
            .attach_printable(config.delimiter.to_string())?;

        let r = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(delimiter)
            .from_path(&config.csv_file)
            .change_context(CsvFileError::Load)
            .attach_printable(format!("File: {}", config.csv_file.display(),))?;

        let group_value_rows = r.into_records().skip(config.start_row_index);

        let mut values_hash_set = HashSet::new();
        let mut values = vec![];
        let mut group_values: Vec<GroupValuesInternal> = vec![];
        let mut translations: Vec<Language> = vec![];

        for r in group_value_rows {
            let r = r.change_context(CsvFileError::Load)?;
            let value = r
                .get(config.values_column_index)
                .ok_or(CsvFileError::SelectedColumnDoesNotExists.report())
                .attach_printable(format!(
                    "File: {}, Column: {}",
                    config.csv_file.display(),
                    config.values_column_index,
                ))?
                .trim()
                .to_string();

            if !values_hash_set.contains(&value) {
                values_hash_set.insert(value.clone());
                values.push(toml::Value::String(value.clone()));
            }

            let group_value = r
                .get(config.group_values_column_index)
                .ok_or(CsvFileError::SelectedColumnDoesNotExists.report())
                .attach_printable(format!(
                    "File: {}, Column: {}",
                    config.csv_file.display(),
                    config.group_values_column_index,
                ))?
                .trim()
                .to_string();

            let key = AttributeInternal::attribute_name_to_attribute_key(&value);
            if let Some(group_values) = group_values.iter_mut().find(|v| v.key == key) {
                group_values
                    .values
                    .push(toml::Value::String(group_value.to_string()));
            } else {
                group_values.push(GroupValuesInternal {
                    key: key.clone(),
                    values: vec![toml::Value::String(group_value.to_string())],
                });
            }

            let group_value_key = AttributeInternal::attribute_name_to_attribute_key(&group_value);
            for t in &config.translations {
                let value_translation = r
                    .get(t.values_column_index)
                    .ok_or(CsvFileError::SelectedColumnDoesNotExists.report())
                    .attach_printable(format!(
                        "File: {}, Column: {}",
                        config.csv_file.display(),
                        t.values_column_index,
                    ))?
                    .trim()
                    .to_string();

                let group_value_translation = r
                    .get(t.group_values_column_index)
                    .ok_or(CsvFileError::SelectedColumnDoesNotExists.report())
                    .attach_printable(format!(
                        "File: {}, Column: {}",
                        config.csv_file.display(),
                        t.group_values_column_index,
                    ))?
                    .trim()
                    .to_string();

                if let Some(lang) = translations.iter_mut().find(|v| v.lang == t.lang) {
                    if !lang.values.iter().any(|v| v.key == key) {
                        lang.values.push(Translation {
                            key: key.clone(),
                            name: value_translation,
                        });
                    }
                    if !lang.values.iter().any(|v| v.key == group_value_key) {
                        lang.values.push(Translation {
                            key: group_value_key.clone(),
                            name: group_value_translation,
                        });
                    }
                } else {
                    translations.push(Language {
                        lang: t.lang.clone(),
                        values: vec![
                            Translation {
                                key: key.clone(),
                                name: value_translation,
                            },
                            Translation {
                                key: group_value_key.clone(),
                                name: group_value_translation,
                            },
                        ],
                    });
                }
            }
        }

        a.values = values;
        a.group_values = group_values;
        a.translations = translations;

        Ok(())
    }
}
