use std::{collections::HashSet, num::NonZeroU8, str::FromStr};

use base64::Engine;
use model::{
    AttributeHash, AttributeId, AttributeOrderMode, PartialProfileAttributesConfig,
    ProfileAttributeInfo,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProfileAttributesSchemaExport {
    pub attribute_order: AttributeOrderMode,
    pub attributes: Vec<Attribute>,
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

fn value_bool_true() -> bool {
    true
}

fn value_bool_false() -> bool {
    false
}

fn value_is_true(v: &bool) -> bool {
    *v
}

fn value_is_false(v: &bool) -> bool {
    !*v
}

fn value_empty_vec<T>() -> Vec<T> {
    vec![]
}

fn value_is_empty<T>(v: &[T]) -> bool {
    v.is_empty()
}

fn value_non_zero_u8_one() -> NonZeroU8 {
    NonZeroU8::new(1).unwrap()
}

fn value_non_zero_u8_is_one(v: &NonZeroU8) -> bool {
    v.get() == 1
}

struct ModeAndIdSequenceNumber;

impl ModeAndIdSequenceNumber {
    const FIRST_INTEGER_ID: u16 = 1;
    const FIRST_BITFLAG_ID: u16 = 1;
    const LAST_BITFLAG_ID: u32 = 0x8000;

    fn validate_integer_id(id: u16) -> Result<(), String> {
        if id < Self::FIRST_INTEGER_ID {
            return Err(format!(
                "Invalid ID {}, id < {}",
                id,
                Self::FIRST_INTEGER_ID
            ));
        }

        Ok(())
    }

    fn validate_bitflag_id(id: u16) -> Result<(), String> {
        if id.count_ones() != 1 {
            return Err(format!("Invalid ID {id}, id.count_ones() != 1"));
        }

        if id < Self::FIRST_BITFLAG_ID {
            return Err(format!(
                "Invalid ID {}, id < {}",
                id,
                Self::FIRST_BITFLAG_ID
            ));
        }

        let id_u32: u32 = id.into();
        if id_u32 > Self::LAST_BITFLAG_ID {
            return Err(format!("Invalid ID {}, id > {}", id, Self::LAST_BITFLAG_ID));
        }

        Ok(())
    }
}

impl Attribute {
    pub fn attribute_name_to_attribute_key(s: &str) -> String {
        s.to_lowercase().replace(' ', "_")
    }

    pub fn validate(mut self) -> Result<ValidatedAttribute, String> {
        self.validate_internal()?;
        for top_level_value in &mut self.values {
            top_level_value.group_values.sort_by_key(|v| v.id);
        }
        let hash = self.hash()?;

        Ok(ValidatedAttribute::new(self, hash))
    }

    fn validate_internal(&self) -> Result<(), String> {
        let mut keys = HashSet::new();
        keys.insert(self.key.clone());

        let mut top_level_ids = HashSet::new();
        let mut top_level_order_numbers = HashSet::new();

        for value in &self.values {
            if top_level_ids.contains(&value.id) {
                return Err(format!("Duplicate id {}", value.id));
            }
            top_level_ids.insert(value.id);

            if top_level_order_numbers.contains(&value.order_number) {
                return Err(format!("Duplicate order number {}", value.order_number));
            }
            top_level_order_numbers.insert(value.order_number);

            if keys.contains(&value.key) {
                return Err(format!("Duplicate key {}", value.key));
            }
            keys.insert(value.key.clone());

            if self.mode.is_bitflag() {
                ModeAndIdSequenceNumber::validate_bitflag_id(value.id)?;
            } else {
                ModeAndIdSequenceNumber::validate_integer_id(value.id)?;
            }
        }

        if self.values.is_empty() {
            return Err(format!(
                "Attribute {} must have at least one value",
                self.key
            ));
        }

        // Check that correct IDs are used.
        if self.mode.is_bitflag() {
            let mut current = 1;
            for _ in 0..self.values.len() {
                if !top_level_ids.contains(&current) {
                    return Err(format!(
                        "ID {} is missing from attribute value IDs for attribute {}, all bitflags between 0 and {} should be used",
                        current,
                        self.key,
                        1 << (self.values.len() - 1)
                    ));
                }
                current <<= 1;
            }
        } else {
            for i in 1..=self.values.len() {
                let i = i as u16;
                if !top_level_ids.contains(&i) {
                    return Err(format!(
                        "ID {} is missing from attribute value IDs for attribute {}, all numbers between 1 and {} should be used",
                        i,
                        self.key,
                        self.values.len()
                    ));
                }
            }
        }

        let mut has_group_values = false;
        for top_level_value in &self.values {
            let mut sub_level_ids = HashSet::new();
            let mut sub_level_order_numbers = HashSet::new();

            for value in &top_level_value.group_values {
                has_group_values = true;
                if !value.group_values.is_empty() {
                    return Err(format!(
                        "Value {} in group {} cannot contain nested group values",
                        value.key, top_level_value.key
                    ));
                }

                ModeAndIdSequenceNumber::validate_integer_id(value.id)?;

                if sub_level_ids.contains(&value.id) {
                    return Err(format!("Duplicate id {}", value.id));
                }
                sub_level_ids.insert(value.id);

                if sub_level_order_numbers.contains(&value.order_number) {
                    return Err(format!("Duplicate order number {}", value.order_number));
                }
                sub_level_order_numbers.insert(value.order_number);

                if keys.contains(&value.key) {
                    return Err(format!("Duplicate key {}", value.key));
                }
                keys.insert(value.key.clone());
            }

            for i in 1..=top_level_value.group_values.len() {
                let i = i as u16;
                if !sub_level_ids.contains(&i) {
                    return Err(format!(
                        "ID {} is missing from value IDs for value group {}, all numbers between 1 and {} should be used",
                        i,
                        top_level_value.key,
                        top_level_value.group_values.len()
                    ));
                }
            }
        }

        if self.mode.is_bitflag() && has_group_values {
            return Err("Bitflag mode cannot have group values".to_string());
        }

        if self.mode.is_one_level() && has_group_values {
            return Err("One level attribute cannot have group values".to_string());
        }

        for t in self.translations.clone() {
            for l in t.values {
                if !keys.contains(&l.key) {
                    return Err(format!(
                        "Missing attribute key definition for translation key {}",
                        l.key
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributeValue {
    /// Unique string identifier for the attribute value.
    pub key: String,
    /// Default name for the attribute value if translated value
    /// is not available.
    pub name: String,
    /// Numeric unique identifier for the attribute value.
    /// Note that the value must only be unique within a group of values, so
    /// value in top level group A, sub level group C and sub level group B
    /// can have the same ID.
    pub id: u16,
    /// Order number for client to determine in what order the
    /// values should be displayed.
    pub order_number: u16,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub editable: bool,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub icon: Option<IconResource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(no_recursion, default = json!([]))]
    /// Change attribute value to be a group identifier. Max depth 2.
    ///
    /// Vec values are sorted by [AttributeValue::id].
    /// Indexing with the ID is not possible as ID values start from 1.
    pub group_values: Vec<AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Language {
    /// Language code.
    pub lang: String,
    pub values: Vec<Translation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Translation {
    /// Attribute name or attribute value key.
    pub key: String,
    /// Translated text.
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum AttributeMode {
    /// u16 bitflag
    Bitflag,
    /// u16 values
    OneLevel,
    /// u32 values
    TwoLevel,
}

impl AttributeMode {
    pub fn is_bitflag(&self) -> bool {
        *self == Self::Bitflag
    }

    pub fn is_one_level(&self) -> bool {
        *self == Self::OneLevel
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum AttributeValueOrderMode {
    AlphabethicalKey,
    AlphabethicalValue,
    OrderNumber,
}

#[derive(Debug, Clone, Copy)]
pub enum IconLocation {
    /// Icon is located in the Material icon set.
    Material,
}

impl From<IconLocation> for &str {
    fn from(src: IconLocation) -> Self {
        match src {
            IconLocation::Material => "material",
        }
    }
}

impl FromStr for IconLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "material" => Ok(IconLocation::Material),
            _ => Err(format!("Unknown icon location {s}")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct IconResource {
    pub location: IconLocation,
    pub identifier: String,
}

impl TryFrom<String> for IconResource {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (location, identifier) = value
            .split_once(':')
            .ok_or(format!("Missing delimiter in {value}"))?;
        let location = location.parse()?;
        Ok(Self {
            location,
            identifier: identifier.to_string(),
        })
    }
}

impl From<IconResource> for String {
    fn from(icon: IconResource) -> Self {
        let location: &str = icon.location.into();
        format!("{}:{}", location, icon.identifier)
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedAttribute {
    attribute: Attribute,
    hash: AttributeHash,
}

impl ValidatedAttribute {
    fn new(attribute: Attribute, hash: AttributeHash) -> Self {
        Self { attribute, hash }
    }

    pub fn attribute(&self) -> &Attribute {
        &self.attribute
    }

    pub fn hash(&self) -> &AttributeHash {
        &self.hash
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
        attributes: Vec<(Attribute, AttributeHash)>,
        attribute_order: AttributeOrderMode,
    ) -> Result<Self, String> {
        let mut validated_attributes: Vec<ValidatedAttribute> = Vec::new();
        for (a, _) in attributes {
            validated_attributes.push(a.validate()?);
        }

        validated_attributes.sort_by_key(|v| v.attribute.id);

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
                self.get_attribute_and_hash(id).cloned().map(|validated| {
                    ProfileAttributesConfigQueryItem {
                        a: validated.attribute,
                        h: validated.hash,
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributesConfigQueryItem {
    pub a: Attribute,
    pub h: AttributeHash,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Attribute {
    /// String unique identifier for the attribute.
    pub key: String,
    /// Default name for the attribute if translated value is not available.
    pub name: String,
    /// Mode of the attribute.
    pub mode: AttributeMode,
    #[serde(
        default = "value_non_zero_u8_one",
        skip_serializing_if = "value_non_zero_u8_is_one"
    )]
    #[schema(default = 1, value_type = u8)]
    pub max_selected: NonZeroU8,
    #[serde(
        default = "value_non_zero_u8_one",
        skip_serializing_if = "value_non_zero_u8_is_one"
    )]
    #[schema(default = 1, value_type = u8)]
    pub max_filters: NonZeroU8,
    /// Client should show this attribute when editing a profile.
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub editable: bool,
    /// Client should show this attribute when viewing a profile.
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub visible: bool,
    /// Client should ask this attribute when doing account initial setup.
    #[serde(default = "value_bool_false", skip_serializing_if = "value_is_false")]
    #[schema(default = false)]
    pub required: bool,
    /// Icon for the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub icon: Option<IconResource>,
    /// Numeric unique identifier for the attribute.
    pub id: AttributeId,
    /// Attribute order number.
    pub order_number: u16,
    /// Attribute value ordering mode for client to determine in what order
    /// the values should be displayed.
    pub value_order: AttributeValueOrderMode,
    /// Top level values for the attribute.
    ///
    /// Values are sorted by AttributeValue ID. Indexing with it is
    /// not possible as ID might be a bitflag value.
    pub values: Vec<AttributeValue>,
    /// Translations for attribute name and attribute values.
    #[serde(default = "value_empty_vec", skip_serializing_if = "value_is_empty")]
    #[schema(default = json!([]))]
    pub translations: Vec<Language>,
}

impl Attribute {
    pub fn hash(&self) -> Result<AttributeHash, String> {
        let attribute_json = serde_json::to_string(self).map_err(|e| e.to_string())?;

        let mut hasher = Sha256::new();
        hasher.update(attribute_json);
        let result = hasher.finalize();

        let h = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result);

        Ok(AttributeHash::new(h))
    }
}
