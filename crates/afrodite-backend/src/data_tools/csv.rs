use std::{collections::HashSet, path::PathBuf};

use database::{
    DbReaderRaw, DbWriter,
    current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon},
};
use model_server_data::{Attribute, AttributeValue, GroupValues, Language, Translation};

#[derive(Debug)]
enum CsvFileError {
    Load,
    SelectedColumnDoesNotExists,
    UnsupportedDelimiterCharacter,
}

#[derive(Debug, Clone)]
struct GroupValuesCsvConfig {
    csv_file: PathBuf,
    delimiter: char,
    values_column_index: usize,
    group_values_column_index: usize,
    start_row_index: usize,
    translations: Vec<GroupValuesCsvTranslationColumn>,
}

#[derive(Debug, Clone)]
struct GroupValuesCsvTranslationColumn {
    lang: String,
    values_column_index: usize,
    group_values_column_index: usize,
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_load_profile_attributes_values_from_csv(
    reader: &DbReaderRaw<'_>,
    writer: &DbWriter<'_>,
    attribute_id: usize,
    csv_file: PathBuf,
    delimiter: char,
    values_column_index: usize,
    group_values_column_index: usize,
    start_row_index: usize,
    translations: Vec<String>,
) {
    let db_attribute_json = reader
        .db_read(move |mut cmds| {
            let row = cmds
                .common()
                .profile_attributes()
                .all_profile_attributes()?
                .into_iter()
                .find(|(id, _, _)| *id == attribute_id as i16);
            Ok(row.map(|(_, json, _)| json))
        })
        .await
        .unwrap_or_else(|e| panic!("Failed to read profile attributes from DB: {e:?}"));

    let db_attribute_json = db_attribute_json
        .unwrap_or_else(|| panic!("Attribute ID {} not found in database", attribute_id));

    let mut attribute: Attribute = serde_json::from_str(&db_attribute_json)
        .unwrap_or_else(|e| panic!("Failed to parse attribute JSON from DB: {}", e));

    let translation_columns = parse_csv_translations(translations);
    let csv_config = GroupValuesCsvConfig {
        csv_file,
        delimiter,
        values_column_index,
        group_values_column_index,
        start_row_index,
        translations: translation_columns,
    };

    let (values, translations) =
        load_for_attribute(&csv_config).unwrap_or_else(|e| panic!("CSV loading failed: {e:?}"));

    attribute.values = values;
    attribute.translations = translations;

    let (attribute_for_db, hash) = attribute
        .to_attribute_and_hash_with_validation()
        .unwrap_or_else(|e| panic!("Validation failed: {}", e));

    let attr_id = attribute_for_db.id.to_i16();
    let json = serde_json::to_string(&attribute_for_db)
        .unwrap_or_else(|e| panic!("JSON serialization failed: {}", e));
    let hash = hash.as_str().to_string();

    writer
        .db_transaction_raw(move |mut cmds| {
            cmds.common()
                .profile_attributes()
                .upsert_profile_attribute(attr_id, &json, &hash)?;
            Ok(())
        })
        .await
        .unwrap();

    println!(
        "Imported CSV data for attribute ID {} and updated database",
        attribute_id
    );
}

fn parse_csv_translations(translations: Vec<String>) -> Vec<GroupValuesCsvTranslationColumn> {
    translations
        .into_iter()
        .map(|value| {
            let mut parts = value.split(':');
            let lang = parts
                .next()
                .unwrap_or_else(|| panic!("Invalid --translation '{}': missing language", value));
            let values_column_index = parts
                .next()
                .unwrap_or_else(|| {
                    panic!(
                        "Invalid --translation '{}': missing values column index",
                        value
                    )
                })
                .parse::<usize>()
                .unwrap_or_else(|_| {
                    panic!(
                        "Invalid --translation '{}': values column index must be usize",
                        value
                    )
                });
            let group_values_column_index = parts
                .next()
                .unwrap_or_else(|| {
                    panic!(
                        "Invalid --translation '{}': missing group values column index",
                        value
                    )
                })
                .parse::<usize>()
                .unwrap_or_else(|_| {
                    panic!(
                        "Invalid --translation '{}': group values column index must be usize",
                        value
                    )
                });

            if parts.next().is_some() {
                panic!(
                    "Invalid --translation '{}': expected format lang:values_col:group_values_col",
                    value
                );
            }

            GroupValuesCsvTranslationColumn {
                lang: lang.to_string(),
                values_column_index,
                group_values_column_index,
            }
        })
        .collect()
}

fn load_for_attribute(
    config: &GroupValuesCsvConfig,
) -> Result<(Vec<AttributeValue>, Vec<Language>), CsvFileError> {
    let delimiter: u8 = config
        .delimiter
        .try_into()
        .map_err(|_| CsvFileError::UnsupportedDelimiterCharacter)?;

    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_path(&config.csv_file)
        .map_err(|_| CsvFileError::Load)?;

    let group_value_rows = reader.into_records().skip(config.start_row_index);

    let mut values_hash_set = HashSet::new();
    let mut values: Vec<AttributeValue> = vec![];
    let mut translations: Vec<Language> = vec![];

    for row in group_value_rows {
        let row = row.map_err(|_| CsvFileError::Load)?;

        let value = row
            .get(config.values_column_index)
            .ok_or(CsvFileError::SelectedColumnDoesNotExists)?
            .trim()
            .to_string();

        if !values_hash_set.contains(&value) {
            values_hash_set.insert(value.clone());
            let key = Attribute::attribute_name_to_attribute_key(&value);
            let next_id: u16 = (values.len() + 1)
                .try_into()
                .map_err(|_| CsvFileError::Load)?;
            values.push(AttributeValue {
                key,
                name: value.clone(),
                id: next_id,
                order_number: next_id,
                editable: true,
                visible: true,
                icon: None,
                group_values: None,
            });
        }

        let group_value = row
            .get(config.group_values_column_index)
            .ok_or(CsvFileError::SelectedColumnDoesNotExists)?
            .trim()
            .to_string();

        let key = Attribute::attribute_name_to_attribute_key(&value);
        let top_level = values
            .iter_mut()
            .find(|v| v.key == key)
            .ok_or(CsvFileError::Load)?;

        let group_value_key = Attribute::attribute_name_to_attribute_key(&group_value);
        let group_values = top_level.group_values.get_or_insert_with(|| GroupValues {
            key: key.clone(),
            values: vec![],
        });
        if !group_values.values.iter().any(|v| v.key == group_value_key) {
            let next_id: u16 = (group_values.values.len() + 1)
                .try_into()
                .map_err(|_| CsvFileError::Load)?;
            group_values.values.push(AttributeValue {
                key: group_value_key.clone(),
                name: group_value.clone(),
                id: next_id,
                order_number: next_id,
                editable: true,
                visible: true,
                icon: None,
                group_values: None,
            });
        }

        for translation_column in &config.translations {
            let value_translation = row
                .get(translation_column.values_column_index)
                .ok_or(CsvFileError::SelectedColumnDoesNotExists)?
                .trim()
                .to_string();

            let group_value_translation = row
                .get(translation_column.group_values_column_index)
                .ok_or(CsvFileError::SelectedColumnDoesNotExists)?
                .trim()
                .to_string();

            if let Some(lang) = translations
                .iter_mut()
                .find(|lang| lang.lang == translation_column.lang)
            {
                if !lang.values.iter().any(|translation| translation.key == key) {
                    lang.values.push(Translation {
                        key: key.clone(),
                        name: value_translation,
                    });
                }
                if !lang
                    .values
                    .iter()
                    .any(|translation| translation.key == group_value_key)
                {
                    lang.values.push(Translation {
                        key: group_value_key.clone(),
                        name: group_value_translation,
                    });
                }
            } else {
                translations.push(Language {
                    lang: translation_column.lang.clone(),
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

    Ok((values, translations))
}
