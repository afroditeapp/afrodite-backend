use std::collections::HashSet;

use model::{
    Attribute, AttributeId, AttributeOrderMode, PartialProfileAttributesConfig,
    ProfileAttributeInfo, ProfileAttributesConfigQueryItem, ValidatedAttribute,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileAttributesSchemaExport {
    attribute_order: AttributeOrderMode,
    attributes: Vec<Attribute>,
}

impl ProfileAttributesSchemaExport {
    fn validate_attributes(mut self) -> Result<(AttributeOrderMode, Vec<Attribute>), String> {
        let mut keys = HashSet::new();
        let mut ids = HashSet::new();
        let mut order_numbers = HashSet::new();
        // Validate uniquenes of keys, IDs and order numbers.
        for attribute in &self.attributes {
            if keys.contains(&attribute.key) {
                return Err(format!("Duplicate key {}", attribute.key));
            }
            keys.insert(attribute.key.clone());

            if ids.contains(&attribute.id) {
                return Err(format!("Duplicate id {}", attribute.id.to_usize()));
            }
            ids.insert(attribute.id);

            if order_numbers.contains(&attribute.order_number) {
                return Err(format!("Duplicate order number {}", attribute.order_number));
            }
            order_numbers.insert(attribute.order_number);
        }

        // Check that correct IDs are used.
        for i in 1..=self.attributes.len() {
            let i: i16 = i
                .try_into()
                .map_err(|e: std::num::TryFromIntError| e.to_string())?;
            if i < 0 {
                return Err(format!("ID {i} is is negative"));
            }
            let id = AttributeId::new(i);
            if !ids.contains(&id) {
                return Err(format!(
                    "ID {} is missing from attribute ID values, all numbers between 1 and {} should be used",
                    i,
                    self.attributes.len()
                ));
            }
        }
        self.attributes.sort_by_key(|a| a.id);

        Ok((self.attribute_order, self.attributes))
    }

    pub fn validate(self) -> Result<ProfileAttributesInternal, String> {
        let (attribute_order, internal_attributes) = self.validate_attributes()?;

        let mut attributes = vec![];
        let mut attributes_for_info = vec![];
        for a in internal_attributes {
            let validated = a.validate()?;
            let id_and_hash = ProfileAttributeInfo {
                id: validated.attribute().id,
                h: validated.hash().clone(),
            };
            attributes.push(validated);
            attributes_for_info.push(id_and_hash);
        }
        Ok(ProfileAttributesInternal {
            attributes,
            config: PartialProfileAttributesConfig {
                attribute_order,
                attributes: attributes_for_info,
            },
        })
    }
}

#[derive(Debug, Default)]
pub struct ProfileAttributesInternal {
    /// List of attributes.
    ///
    /// Attributes are sorted by Attribute ID.
    /// Indexing with the ID is not possible as ID values start from 1.
    attributes: Vec<ValidatedAttribute>,
    config: PartialProfileAttributesConfig,
}

impl ProfileAttributesInternal {
    pub fn get_attribute(&self, id: AttributeId) -> Option<&Attribute> {
        self.get_attribute_and_hash(id).map(|v| v.attribute())
    }

    pub fn attributes(&self) -> &[ValidatedAttribute] {
        &self.attributes
    }

    pub fn attribute_order(&self) -> AttributeOrderMode {
        self.config.attribute_order
    }

    fn get_attribute_and_hash(&self, id: AttributeId) -> Option<&ValidatedAttribute> {
        self.attributes.get(id.to_usize().saturating_sub(1))
    }

    pub fn from_db_data(
        attributes: Vec<Attribute>,
        attribute_order: AttributeOrderMode,
    ) -> Result<Self, String> {
        let mut validated_attributes: Vec<ValidatedAttribute> = Vec::new();
        for a in attributes {
            validated_attributes.push(a.validate()?);
        }

        validated_attributes.sort_by_key(|v| v.attribute().id);

        let attributes_for_info = validated_attributes
            .iter()
            .map(|validated| ProfileAttributeInfo {
                id: validated.attribute().id,
                h: validated.hash().clone(),
            })
            .collect();

        Ok(Self {
            attributes: validated_attributes,
            config: PartialProfileAttributesConfig {
                attribute_order,
                attributes: attributes_for_info,
            },
        })
    }

    pub fn config_for_client(&self) -> &PartialProfileAttributesConfig {
        &self.config
    }

    pub fn query_attributes(&self, ids: Vec<AttributeId>) -> Vec<ProfileAttributesConfigQueryItem> {
        ids.into_iter()
            .filter_map(|id| {
                self.get_attribute_and_hash(id)
                    .cloned()
                    .map(|validated| validated.into())
            })
            .collect()
    }

    pub fn export(&self) -> ProfileAttributesSchemaExport {
        ProfileAttributesSchemaExport {
            attribute_order: self.attribute_order(),
            attributes: self
                .attributes()
                .iter()
                .map(|validated| validated.attribute().clone())
                .collect(),
        }
    }
}
