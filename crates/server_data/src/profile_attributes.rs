use std::sync::Arc;

use database::current::read::GetDbReadCommandsCommon;
use error_stack::ResultExt;
use model_server_data::{ProfileAttributesInternal, ProfileAttributesSchemaExport};
use simple_backend_utils::IntoReportFromString;

use crate::DataError;

#[derive(Debug, Default, Clone)]
pub struct ProfileAttributesSchemaManager {
    schema: Arc<ProfileAttributesInternal>,
}

impl ProfileAttributesSchemaManager {
    /// Create a new manager with initial profile attributes data.
    pub fn new(schema: ProfileAttributesInternal) -> Self {
        Self {
            schema: schema.into(),
        }
    }

    pub fn schema(&self) -> &ProfileAttributesInternal {
        &self.schema
    }

    pub fn export(&self) -> ProfileAttributesSchemaExport {
        ProfileAttributesSchemaExport {
            attribute_order: self.schema.attribute_order(),
            attributes: self
                .schema
                .attributes()
                .iter()
                .map(|validated| validated.attribute().clone())
                .collect(),
        }
    }
}

/// Load profile attributes from the database and create a ProfileAttributesManager.
///
/// This reads all profile attribute definitions from the `profile_attributes` table
/// and the attribute order mode from the `profile_attributes_schema` table,
/// then constructs a ProfileAttributesManager for runtime use.
///
/// If no attributes are found in the database, returns an empty ProfileAttributesInternal.
pub async fn load_profile_attributes_from_db(
    reader: &database::DbReaderRaw<'_>,
) -> error_stack::Result<ProfileAttributesSchemaManager, DataError> {
    let (attributes, order_mode): (
        Vec<(i16, String, String)>,
        Option<model::profile::AttributeOrderMode>,
    ) = reader
        .db_read(|mut mode| {
            let attrs = mode
                .common()
                .profile_attributes()
                .all_profile_attributes()?;
            let order = mode.common().profile_attributes().attribute_order_mode()?;
            Ok((attrs, order))
        })
        .await
        .change_context(DataError::Diesel)?;

    let attribute_order = order_mode.unwrap_or_default();

    // Deserialize each attribute from JSON
    let mut parsed_attributes = Vec::new();
    for (attr_id, attribute_json, hash_text) in attributes {
        let attribute: model_server_data::Attribute = serde_json::from_str(&attribute_json)
            .map_err(|e| {
                tracing::error!("Failed to deserialize attribute {}: {}", attr_id, e);
                DataError::Diesel
            })?;

        let hash = model::profile::AttributeHash::new(hash_text);
        parsed_attributes.push((attribute, hash));
    }

    // Build ProfileAttributesInternal from the parsed data
    let profile_attributes =
        ProfileAttributesInternal::from_db_data(parsed_attributes, attribute_order)
            .into_error_string(DataError::NotAllowed)?;

    Ok(ProfileAttributesSchemaManager::new(profile_attributes))
}
