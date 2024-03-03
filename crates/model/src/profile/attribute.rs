use std::{collections::HashSet, fmt::format, str::FromStr};

use base64::Engine;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize, Serializer};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, Account, AccountState, Capabilities, ContentProcessingId, ContentProcessingState, MessageNumber, ModerationQueueNumber, ModerationQueueType, Profile, ProfileVisibility
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributesFileInternal {
    attribute: Vec<AttributeInternal>,
}

impl AttributesFileInternal {
    fn validate_uniquenes_of_keys_and_ids(self) -> Result<Vec<AttributeInternal>, String> {
        let mut keys = HashSet::new();
        let mut ids = HashSet::new();
        for attribute in &self.attribute {
            if keys.contains(&attribute.key) {
                return Err(format!("Duplicate key {}", attribute.key));
            }
            keys.insert(attribute.key.clone());
            if ids.contains(&attribute.id) {
                return Err(format!("Duplicate id {}", attribute.id));
            }
            ids.insert(attribute.id);
        }
        Ok(self.attribute)
    }

    pub fn validate(self) -> Result<ProfileAttributes, String> {
        ProfileAttributes::from_file(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInternal {
    pub key: String,
    pub name: String,
    pub mode: AttributeMode,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    pub editable: bool,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    pub visible: bool,
    #[serde(default = "value_bool_false", skip_serializing_if = "value_is_false")]
    pub required: bool,
    pub icon: IconResource,
    pub id: u16,
    /// Array of strings or objects
    pub values: toml::value::Array,
    #[serde(default = "value_empty_vec")]
    pub group_values: Vec<GroupValuesInternal>,
    #[serde(default = "value_empty_vec")]
    pub translations: Vec<Language>,
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

fn value_is_empty<T>(v: &Vec<T>) -> bool {
    v.is_empty()
}

struct ModeAndIdSequenceNumber {
    mode: AttributeMode,
    current_id: u16,
}

impl ModeAndIdSequenceNumber {
    const FIRST_BITFLAG_ID: u16 = 2;
    const LAST_BITFLAG_ID: u16 = 0x80;

    fn new(mode: AttributeMode) -> Self {
        Self {
            mode,
            current_id: match mode {
                AttributeMode::SelectSingleFilterSingle => 0,
                AttributeMode::SelectSingleFilterMultiple |
                AttributeMode::SelectMultipleFilterMultiple => Self::FIRST_BITFLAG_ID,
            }
        }
    }

    fn set_id(&mut self, id: u16) -> Result<(), String> {
        if id < self.current_id {
            return Err(format!("Invalid ID {}, id < current_id {}", id, self.current_id));
        }

        match self.mode {
            AttributeMode::SelectSingleFilterSingle => {
                self.current_id = id;
            }
            AttributeMode::SelectSingleFilterMultiple |
            AttributeMode::SelectMultipleFilterMultiple => {
                Self::validate_bitflag_id(id)?;
                self.current_id = id;
            }
        }

        Ok(())
    }

    fn current_id(&self) -> u16 {
        self.current_id
    }

    fn validate_bitflag_id(id: u16) -> Result<(), String> {
        if id.count_ones() != 1 {
            return Err(format!("Invalid ID {}, id.count_ones() != 1", id));
        }

        if id < Self::FIRST_BITFLAG_ID {
            return Err(format!("Invalid ID {}, id < {}", id, Self::FIRST_BITFLAG_ID));
        }

        if id > Self::LAST_BITFLAG_ID {
            return Err(format!("Invalid ID {}, id > {}", id, Self::LAST_BITFLAG_ID));
        }

        Ok(())
    }

    /// Increment the current ID and return the updated current ID.
    fn increment_id(&mut self) -> Result<u16, String> {
        match self.mode {
            AttributeMode::SelectSingleFilterSingle => {
                self.current_id += 1;
                Ok(self.current_id)
            }
            AttributeMode::SelectSingleFilterMultiple |
            AttributeMode::SelectMultipleFilterMultiple => {
                let tmp = self.current_id << 1;
                Self::validate_bitflag_id(tmp)?;
                self.current_id = tmp;
                Ok(self.current_id)
            }
        }
    }
}

struct AttributeInfoValidated {
    values: Vec<AttributeValue>,
    group_values: Vec<GroupValues>,
    translations: Vec<Language>,
}

impl AttributeInternal {
    fn validate(&self) -> Result<AttributeInfoValidated, String> {
        let mut top_level_ids = HashSet::new();
        let mut current_top_level_id = ModeAndIdSequenceNumber::new(self.mode);
        let mut keys = HashSet::new();
        let mut values = Vec::new();

        keys.insert(self.key.clone());

        fn handle_attribute_value(
            v: toml::Value,
            all_ids: &mut HashSet<u16>,
            all_keys: &mut HashSet<String>,
            id_state: &mut ModeAndIdSequenceNumber,
        ) -> Result<AttributeValue, String> {
            match v {
                toml::Value::Table(t) => {
                    let value: AttributeValueInternal = t
                        .try_into()
                        .map_err(|e| format!("Attribute value error: {}", e))?;

                    match value.id {
                        Some(id) => id_state.set_id(id)?,
                        None => {
                            id_state.increment_id()?;
                        }
                    }

                    let id = id_state.current_id();
                    if all_ids.contains(&id) {
                        return Err(format!("Duplicate id {}", id));
                    }
                    all_ids.insert(id);

                    if all_keys.contains(&value.key) {
                        return Err(format!("Duplicate key {}", value.key));
                    }
                    all_keys.insert(value.key.clone());

                    let value = AttributeValue {
                        key: value.key,
                        value: value.value,
                        id: id_state.current_id(),
                        editable: value.editable,
                        visible: value.visible,
                        icon: value.icon,
                    };
                    Ok(value)
                }
                toml::Value::String(s) => {
                    let value = AttributeValue {
                        key: s.to_lowercase(),
                        value: s,
                        id: id_state.increment_id()?,
                        editable: true,
                        visible: true,
                        icon: None,
                    };

                    if all_ids.contains(&value.id) {
                        return Err(format!("Duplicate id {}", value.id));
                    }
                    all_ids.insert(value.id);

                    if all_keys.contains(&value.key) {
                        return Err(format!("Duplicate key {}", value.key));
                    }
                    all_keys.insert(value.key.clone());

                    Ok(value)
                }
                _ => return Err(format!("Invalid value type: {:?}", v)),
            }
        }

        for v in  self
            .values
            .clone()
            {
                values.push(
                    handle_attribute_value(
                        v,
                        &mut top_level_ids,
                        &mut keys,
                        &mut current_top_level_id
                    )?
                );
            }

        if values.is_empty() {
            return Err(format!("Attribute {} must have at least one value", self.key));
        }

        let mut group_values = Vec::new();

        for g in self
            .group_values
            .clone() {
                if !keys.contains(&g.key) {
                    return Err(format!("Missing attribute value definition for key {}", g.key));
                }

                let mut sub_level_ids = HashSet::new();
                let mut current_sub_level_id = ModeAndIdSequenceNumber::new(self.mode);
                let mut values = Vec::new();

                for v in g.values {
                    let value = handle_attribute_value(
                        v,
                        &mut sub_level_ids,
                        &mut keys,
                        &mut current_sub_level_id,
                    )?;
                    values.push(value);
                }

                group_values.push(
                    GroupValues {
                        key: g.key,
                        values,
                    }
                );
            }

        if self.mode.is_bitflag_mode() && !group_values.is_empty() {
            return Err("Bitflag mode cannot have group values".to_string());
        }

        for t in self
            .translations
            .clone() {
                for l in t.values {
                    if !keys.contains(&l.key) {
                        return Err(format!("Missing attribute value definition for key {}", l.key));
                    }
                }
            }

        Ok(AttributeInfoValidated {
            values,
            group_values,
            translations: self.translations.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupValuesInternal {
    pub key: String,
    /// Array of strings or objects
    pub values: toml::value::Array,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GroupValues {
    pub key: String,
    pub values: Vec<AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeValueInternal {
    pub key: String,
    pub value: String,
    pub id: Option<u16>,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    pub editable: bool,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    pub visible: bool,
    pub icon: Option<IconResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributeValue {
    /// Unique string identifier for the attribute value.
    pub key: String,
    /// English text for the attribute value.
    pub value: String,
    /// Numeric unique identifier for the attribute value.
    /// Note that the value must only be unique within a group of values, so
    /// value in top level group A, sub level group C and sub level group B
    /// can have the same ID.
    pub id: u16,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub editable: bool,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub visible: bool,
    pub icon: Option<IconResource>,
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
    pub value: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AttributeMode {
    SelectSingleFilterSingle,
    SelectSingleFilterMultiple,
    SelectMultipleFilterMultiple,
}

impl AttributeMode {
    pub fn is_bitflag_mode(&self) -> bool {
        match self {
            AttributeMode::SelectSingleFilterSingle => false,
            AttributeMode::SelectSingleFilterMultiple |
            AttributeMode::SelectMultipleFilterMultiple => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IconSrc {
    Material,
}

impl From<IconSrc> for &str {
    fn from(src: IconSrc) -> Self {
        match src {
            IconSrc::Material => "material",
        }
    }
}

impl FromStr for IconSrc {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "material" => Ok(IconSrc::Material),
            _ => Err(format!("Unknown icon src {}", s)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct IconResource {
    pub src: IconSrc,
    pub identifier: String,
}

impl TryFrom<String> for IconResource {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (src, identifier) = value
            .split_once(':')
            .ok_or(format!("Missing delimiter in {}", value))?;
        let src = src.parse()?;
        Ok(Self { src , identifier: identifier.to_string() })
    }
}

impl From<IconResource> for String {
    fn from(icon: IconResource) -> Self {
        let src_str: &str = icon.src.into();
        format!("{}:{}", src_str, icon.identifier)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributes {
    pub attributes: Vec<Attribute>,
}

impl ProfileAttributes {
    pub fn from_file(file: AttributesFileInternal) -> Result<Self, String> {
        let internal_attributes = file.validate_uniquenes_of_keys_and_ids()?;

        let mut attributes = vec![];
        for a in internal_attributes {
            let info = a.validate()?;
            let a = Attribute {
                key: a.key,
                name: a.name,
                mode: a.mode,
                editable: a.editable,
                visible: a.visible,
                required: a.required,
                icon: a.icon,
                id: a.id,
                values: info.values,
                group_values: info.group_values,
                translations: info.translations,
            };
            attributes.push(a);
        }
        Ok(Self { attributes })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Attribute {
    /// String unique identifier for the attribute.
    pub key: String,
    /// English text for the attribute.
    pub name: String,
    /// Mode of the attribute.
    pub mode: AttributeMode,
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
    pub icon: IconResource,
    /// Numeric unique identifier for the attribute.
    pub id: u16,
    /// Top level values for the attribute.
    pub values: Vec<AttributeValue>,
    /// Sub level values for the attribute.
    /// When group_values is not empty, the there is 2 levels of values.
    #[serde(default = "value_empty_vec", skip_serializing_if = "value_is_empty")]
    #[schema(default = "Vec<GroupValues>::new")]
    pub group_values: Vec<GroupValues>,
    /// Translations for attribute name and attribute values.
    #[serde(default = "value_empty_vec", skip_serializing_if = "value_is_empty")]
    #[schema(default = "Vec<Language>::new")]
    pub translations: Vec<Language>,
}
