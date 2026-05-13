use utils::minimal_i64;

use crate::{
    AttributeMode, GetProfileResultInternal, LastSeenTime, Profile, ProfileAttributeValue,
    ProfileAttributesInternal,
};

const RESULT_VARIANT_EMPTY: u8 = 0;
const RESULT_VARIANT_VERSION_ONLY: u8 = 1;
const RESULT_VARIANT_PROFILE_WITH_VERSION: u8 = 2;

const ATTRIBUTE_ID_U32_VALUES_FLAG: u16 = 0x8000;

impl GetProfileResultInternal {
    pub fn to_binary_with_schema(&self, schema: &ProfileAttributesInternal) -> Vec<u8> {
        let mut buffer = Vec::new();

        match self {
            Self::Empty => {
                buffer.push(RESULT_VARIANT_EMPTY);
            }
            Self::VersionOnly {
                version,
                last_seen_time,
            } => {
                buffer.push(RESULT_VARIANT_VERSION_ONLY);
                buffer.extend_from_slice(version.as_ref().as_bytes());
                append_last_seen_time(&mut buffer, *last_seen_time);
            }
            Self::ProfileWithVersion(info) => {
                buffer.push(RESULT_VARIANT_PROFILE_WITH_VERSION);
                buffer.extend_from_slice(info.version.as_ref().as_bytes());
                append_last_seen_time(&mut buffer, info.last_seen_time);
                append_profile(&mut buffer, &info.profile, schema);
            }
        }

        buffer
    }
}

fn append_last_seen_time(buffer: &mut Vec<u8>, value: Option<LastSeenTime>) {
    match value {
        Some(last_seen_time) => minimal_i64::add_minimal_i64(buffer, last_seen_time.raw()),
        None => buffer.push(0),
    }
}

fn append_profile(buffer: &mut Vec<u8>, profile: &Profile, schema: &ProfileAttributesInternal) {
    append_optional_string_u8(buffer, profile.name.as_ref().map(|v| v.as_str()));
    append_optional_string_u16(buffer, profile.ptext.as_ref().map(|v| v.as_str()));
    buffer.push(profile.age.value());

    let attribute_count = profile.attributes_count_u16();
    buffer.extend_from_slice(&attribute_count.to_le_bytes());
    for attribute in &profile.attributes {
        append_attribute_with_values(buffer, attribute, schema);
    }

    let mut flags = 0u8;
    flags |= u8::from(profile.unlimited_likes());
    flags |= u8::from(profile.name_accepted()) << 1;
    flags |= u8::from(profile.ptext_accepted()) << 2;
    buffer.push(flags);

    buffer.extend_from_slice(&profile.verification_status().v.to_le_bytes());
}

fn append_attribute_with_values(
    buffer: &mut Vec<u8>,
    attribute: &ProfileAttributeValue,
    schema: &ProfileAttributesInternal,
) {
    let use_u32_values = schema
        .get_attribute(attribute.id())
        .map(|value| value.mode)
        .unwrap_or_else(|| AttributeMode::TwoLevel)
        == AttributeMode::TwoLevel;

    let attribute_id = attribute.id().to_u16();
    let encoded_attribute_id = if use_u32_values {
        attribute_id | ATTRIBUTE_ID_U32_VALUES_FLAG
    } else {
        attribute_id
    };

    buffer.extend_from_slice(&encoded_attribute_id.to_le_bytes());
    buffer.push(attribute.raw_values_count_u8());

    if use_u32_values {
        for value in attribute.raw_values() {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
    } else {
        for value in attribute.raw_values() {
            let value_u16 = *value as u16;
            buffer.extend_from_slice(&value_u16.to_le_bytes());
        }
    }
}

fn append_optional_string_u8(buffer: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(text) => {
            let bytes = text.as_bytes();
            if let Ok(len) = u8::try_from(bytes.len()) {
                buffer.push(len);
                buffer.extend_from_slice(bytes);
            } else {
                buffer.push(0);
            }
        }
        None => buffer.push(0),
    }
}

fn append_optional_string_u16(buffer: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(text) => {
            let bytes = text.as_bytes();
            if let Ok(len) = u16::try_from(bytes.len()) {
                buffer.extend_from_slice(&len.to_le_bytes());
                buffer.extend_from_slice(bytes);
            } else {
                buffer.extend_from_slice(&0u16.to_le_bytes());
            }
        }
        None => buffer.extend_from_slice(&0u16.to_le_bytes()),
    }
}
