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
    attribute_order: AttributeOrderMode,
    attribute: Vec<AttributeInternal>,
}

impl AttributesFileInternal {
    fn validate_attributes(mut self) -> Result<(AttributeOrderMode, Vec<AttributeInternal>), String> {
        let mut keys = HashSet::new();
        let mut ids = HashSet::new();
        let mut order_numbers = HashSet::new();
        // Validate uniquenes of keys, IDs and order numbers.
        for attribute in &self.attribute {
            if keys.contains(&attribute.key) {
                return Err(format!("Duplicate key {}", attribute.key));
            }
            keys.insert(attribute.key.clone());

            if ids.contains(&attribute.id) {
                return Err(format!("Duplicate id {}", attribute.id));
            }
            ids.insert(attribute.id);

            if order_numbers.contains(&attribute.order_number) {
                return Err(format!("Duplicate order number {}", attribute.order_number));
            }
            order_numbers.insert(attribute.order_number);
        }

        // Check that correct IDs are used.
        for i in 0..self.attribute.len() {
            let i = i as u16;
            if ids.get(&i).is_none() {
                return Err(format!(
                    "ID {} is missing from attribute ID values, all numbers between 0 and {} should be used",
                    i,
                    self.attribute.len() - 1
                ));
            }
        }
        self.attribute.sort_by_key(|a| a.id);

        Ok((self.attribute_order, self.attribute))
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
    pub order_number: u16,
    pub value_order: AttributeValueOrderMode,
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
    current_id: Option<u16>,
}

impl ModeAndIdSequenceNumber {
    const LAST_INTEGER_ID: u16 = i16::MAX as u16;
    const FIRST_BITFLAG_ID: u16 = 1;
    const LAST_BITFLAG_ID: u16 = 0x40;

    fn new(mode: AttributeMode) -> Self {
        Self {
            mode,
            current_id: None,
        }
    }

    fn new_increment_only_mode() -> Self {
        Self::new(AttributeMode::SelectSingleFilterSingle)
    }

    fn set_value(&mut self, id: u16) -> Result<u16, String> {
        match self.mode {
            AttributeMode::SelectSingleFilterSingle => {
                Self::validate_integer_id(id)?;
                self.current_id = Some(id);
            }
            AttributeMode::SelectSingleFilterMultiple |
            AttributeMode::SelectMultipleFilterMultiple => {
                Self::validate_bitflag_id(id)?;
                self.current_id = Some(id);
            }
        }

        Ok(id)
    }

    fn validate_integer_id(id: u16) -> Result<(), String> {
        if id > Self::LAST_INTEGER_ID {
            return Err(format!("Invalid ID {}, id > {}", id, Self::LAST_INTEGER_ID));
        }

        Ok(())
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
    fn increment_value(&mut self) -> Result<u16, String> {
        match self.mode {
            AttributeMode::SelectSingleFilterSingle => {
                let tmp = if let Some(current_id) = self.current_id {
                    current_id + 1
                } else {
                    0
                };
                Self::validate_integer_id(tmp)?;
                self.current_id = Some(tmp);
                Ok(tmp)
            }
            AttributeMode::SelectSingleFilterMultiple |
            AttributeMode::SelectMultipleFilterMultiple => {
                let tmp = if let Some(current_id) = self.current_id {
                    current_id << 1
                } else {
                    1
                };
                Self::validate_bitflag_id(tmp)?;
                self.current_id = Some(tmp);
                Ok(tmp)
            }
        }
    }
}

struct AttributeInfoValidated {
    values: Vec<AttributeValue>,
    translations: Vec<Language>,
}

fn english_text_to_key(s: &str) -> String {
    s.to_lowercase().replace(" ", "_")
}

impl AttributeInternal {
    fn validate(&self) -> Result<AttributeInfoValidated, String> {
        fn handle_attribute_value(
            v: toml::Value,
            all_ids: &mut HashSet<u16>,
            all_order_numbers: &mut HashSet<u16>,
            all_keys: &mut HashSet<String>,
            id_state: &mut ModeAndIdSequenceNumber,
            order_number_state: &mut ModeAndIdSequenceNumber,
        ) -> Result<AttributeValue, String> {
            match v {
                toml::Value::Table(t) => {
                    let value: AttributeValueInternal = t
                        .try_into()
                        .map_err(|e| format!("Attribute value error: {}", e))?;

                    let id = match value.id {
                        Some(id) => id_state.set_value(id)?,
                        None => id_state.increment_value()?,
                    };
                    if all_ids.contains(&id) {
                        return Err(format!("Duplicate id {}", id));
                    }
                    all_ids.insert(id);

                    let key = match value.key {
                        Some(key) => key,
                        None => english_text_to_key(&value.value),
                    };
                    if all_keys.contains(&key) {
                        return Err(format!("Duplicate key {}", key));
                    }
                    all_keys.insert(key.clone());

                    let order_number = match value.order_number {
                        Some(order_number) => order_number_state.set_value(order_number)?,
                        None => order_number_state.increment_value()?
                    };
                    if all_order_numbers.contains(&order_number) {
                        return Err(format!("Duplicate order number {}", order_number));
                    }
                    all_order_numbers.insert(order_number);

                    let value = AttributeValue {
                        key,
                        value: value.value,
                        id,
                        order_number,
                        editable: value.editable,
                        visible: value.visible,
                        icon: value.icon,
                        group_values: None,
                    };
                    Ok(value)
                }
                toml::Value::String(s) => {
                    let value = AttributeValue {
                        key: english_text_to_key(&s),
                        value: s,
                        id: id_state.increment_value()?,
                        order_number: order_number_state.increment_value()?,
                        editable: true,
                        visible: true,
                        icon: None,
                        group_values: None,
                    };

                    if all_ids.contains(&value.id) {
                        return Err(format!("Duplicate id {}", value.id));
                    }
                    all_ids.insert(value.id);

                    if all_keys.contains(&value.key) {
                        return Err(format!("Duplicate key {}", value.key));
                    }
                    all_keys.insert(value.key.clone());

                    if all_order_numbers.contains(&value.order_number) {
                        return Err(format!("Duplicate order number {}", value.order_number));
                    }
                    all_order_numbers.insert(value.order_number);

                    Ok(value)
                }
                _ => return Err(format!("Invalid value type: {:?}", v)),
            }
        }

        let mut keys = HashSet::new();
        keys.insert(self.key.clone());

        let mut top_level_ids = HashSet::new();
        let mut top_level_order_numbers = HashSet::new();
        let mut current_top_level_id = ModeAndIdSequenceNumber::new(self.mode);
        let mut current_top_level_count_number = ModeAndIdSequenceNumber::new_increment_only_mode();
        let mut values = Vec::new();

        for v in  self
            .values
            .clone()
            {
                values.push(
                    handle_attribute_value(
                        v,
                        &mut top_level_ids,
                        &mut top_level_order_numbers,
                        &mut keys,
                        &mut current_top_level_id,
                        &mut current_top_level_count_number,
                    )?
                );
            }

        if values.is_empty() {
            return Err(format!("Attribute {} must have at least one value", self.key));
        }

        // Check that correct IDs are used.
        if self.mode.is_bitflag_mode() {
            let mut current = 1;
            for _ in 0..values.len() {
                if top_level_ids.get(&current).is_none() {
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
            for i in 0..self.values.len() {
                let i = i as u16;
                if top_level_ids.get(&i).is_none() {
                    return Err(format!(
                        "ID {} is missing from attribute value IDs for attribute {}, all numbers between 0 and {} should be used",
                        i,
                        self.key,
                        self.values.len() - 1
                    ));
                }
            }
        }
        values.sort_by(|a, b| a.id.cmp(&b.id));

        let mut group_values = Vec::new();

        for g in self
            .group_values
            .clone() {
                if !keys.contains(&g.key) {
                    return Err(format!("Missing attribute value definition for key {}", g.key));
                }

                let mut sub_level_ids = HashSet::new();
                let mut sub_level_order_numbers = HashSet::new();
                let mut current_sub_level_id = ModeAndIdSequenceNumber::new(self.mode);
                let mut current_sub_level_count_number = ModeAndIdSequenceNumber::new_increment_only_mode();
                let mut values = Vec::new();

                for v in g.values {
                    let value = handle_attribute_value(
                        v,
                        &mut sub_level_ids,
                        &mut sub_level_order_numbers,
                        &mut keys,
                        &mut current_sub_level_id,
                        &mut current_sub_level_count_number,
                    )?;
                    values.push(value);
                }

                if values.is_empty() {
                    return Err(format!("Value group {} must have at least one value", g.key));
                }

                // Check that correct IDs are used.
                for i in 0..values.len() {
                    let i = i as u16;
                    if sub_level_ids.get(&i).is_none() {
                        return Err(format!(
                            "ID {} is missing from value IDs for value group {}, all numbers between 0 and {} should be used",
                            i,
                            g.key,
                            values.len() - 1
                        ));
                    }
                }
                values.sort_by(|a, b| a.id.cmp(&b.id));

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

        for g in group_values.into_iter() {
            if let Some(v) = values.iter_mut().find(|v| v.key == g.key) {
                v.group_values = Some(g);
            }
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
    /// Values for this group.
    ///
    /// Values are sorted by AttributeValue ID related to this group
    /// and ID can be used to index this list.
    pub values: Vec<AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeValueInternal {
    pub key: Option<String>,
    pub value: String,
    pub id: Option<u16>,
    pub order_number: Option<u16>,
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
    /// Order number for client to determine in what order the
    /// values should be displayed.
    pub order_number: u16,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub editable: bool,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub visible: bool,
    pub icon: Option<IconResource>,
    /// Sub level values for this attribute value.
    pub group_values: Option<GroupValues>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum AttributeMode {
    SelectSingleFilterSingle,
    SelectSingleFilterMultiple,
    SelectMultipleFilterMultiple,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum AttributeOrderMode {
    OrderNumber,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum AttributeValueOrderMode {
    AlphabethicalKey,
    AlphabethicalValue,
    OrderNumber,
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
    pub attribute_order: AttributeOrderMode,
    /// List of attributes.
    ///
    /// Attributes are sorted by Attribute ID and ID can be used to
    /// index this list.
    pub attributes: Vec<Attribute>,
}

impl ProfileAttributes {
    pub fn from_file(file: AttributesFileInternal) -> Result<Self, String> {
        let (attribute_order, internal_attributes) = file.validate_attributes()?;

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
                order_number: a.order_number,
                value_order: a.value_order,
                values: info.values,
                translations: info.translations,
            };
            attributes.push(a);
        }
        Ok(Self {
            attribute_order,
            attributes,
        })
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
    #[schema(default = "Vec<Language>::new")]
    pub translations: Vec<Language>,
}
