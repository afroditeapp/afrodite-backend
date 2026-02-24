use database::current::write::GetDbWriteCommandsCommon;
use model_profile::{
    Attribute, AttributeValue, ProfileAttributesInternal, UpdateProfileAttributesSchema,
};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write,
    read::GetReadCommandsCommon,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsProfileAdminAttributeSchema);

async fn load_schema(
    handle: &WriteCommandsProfileAdminAttributeSchema<'_>,
) -> Result<ProfileAttributesInternal, DataError> {
    let (attributes, order) = handle
        .handle()
        .read()
        .common()
        .profile_attributes()
        .all_attributes_from_db()
        .await?;

    ProfileAttributesInternal::from_db_data(attributes, order).map_err(|e| {
        DataError::NotAllowed
            .report()
            .attach_printable(format!("Profile attributes validation error: {e}"))
    })
}

fn value_content_changed(old: &AttributeValue, new: &AttributeValue) -> Result<bool, DataError> {
    if old.name != new.name || old.icon != new.icon {
        return Ok(true);
    }

    let old_group: std::collections::HashMap<u16, &AttributeValue> =
        old.group_values.iter().map(|v| (v.id, v)).collect();
    let new_group: std::collections::HashMap<u16, &AttributeValue> =
        new.group_values.iter().map(|v| (v.id, v)).collect();

    for (id, old_value) in old_group {
        if let Some(new_value) = new_group.get(&id)
            && value_content_changed(old_value, new_value)?
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn attribute_content_changed(old: &Attribute, new: &Attribute) -> Result<bool, DataError> {
    if old.name != new.name || old.icon != new.icon || old.translations != new.translations {
        return Ok(true);
    }

    let old_values: std::collections::HashMap<u16, &AttributeValue> =
        old.values.iter().map(|v| (v.id, v)).collect();
    let new_values: std::collections::HashMap<u16, &AttributeValue> =
        new.values.iter().map(|v| (v.id, v)).collect();

    for (id, old_value) in old_values {
        if let Some(new_value) = new_values.get(&id)
            && value_content_changed(old_value, new_value)?
        {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Key and mode are immutable
fn ensure_values_not_removed_or_immutable_changed(
    old: &Attribute,
    new: &Attribute,
) -> Result<(), DataError> {
    if old.key != new.key || old.mode != new.mode {
        return Err(DataError::NotAllowed.report());
    }

    let new_top_values: std::collections::HashMap<u16, &AttributeValue> =
        new.values.iter().map(|v| (v.id, v)).collect();

    for old_top_value in &old.values {
        let new_top_value = new_top_values
            .get(&old_top_value.id)
            .ok_or_else(|| DataError::NotAllowed.report())?;

        if old_top_value.key != new_top_value.key {
            return Err(DataError::NotAllowed.report());
        }

        let new_group_values: std::collections::HashMap<u16, &AttributeValue> = new_top_value
            .group_values
            .iter()
            .map(|v| (v.id, v))
            .collect();

        for old_group_value in &old_top_value.group_values {
            let new_group_value = new_group_values
                .get(&old_group_value.id)
                .ok_or_else(|| DataError::NotAllowed.report())?;

            if old_group_value.key != new_group_value.key {
                return Err(DataError::NotAllowed.report());
            }
        }
    }

    Ok(())
}

fn validate_new_state(
    current_state: &ProfileAttributesInternal,
    new_state: &ProfileAttributesInternal,
    has_content_permission: bool,
) -> Result<(), DataError> {
    let current_by_id = current_state
        .attributes()
        .iter()
        .map(|v| (v.attribute().id, v))
        .collect::<std::collections::HashMap<_, _>>();

    let mut content_changed = false;

    for new in new_state.attributes().iter().map(|v| v.attribute()) {
        if let Some(current) = current_by_id.get(&new.id).map(|v| v.attribute()) {
            ensure_values_not_removed_or_immutable_changed(current, new)?;
            if attribute_content_changed(current, new)? {
                content_changed = true;
            }
        }
    }

    if content_changed && !has_content_permission {
        return Err(DataError::NotAllowed.report());
    }

    Ok(())
}

impl WriteCommandsProfileAdminAttributeSchema<'_> {
    pub async fn update_schema(
        &self,
        request: UpdateProfileAttributesSchema,
        has_content_permission: bool,
    ) -> Result<(), DataError> {
        let current_state = request.current_state.validate().map_err(|e| {
            DataError::NotAllowed
                .report()
                .attach_printable(format!("current_state schema validation error: {e}"))
        })?;
        let new_state = request.new_state.validate().map_err(|e| {
            DataError::NotAllowed
                .report()
                .attach_printable(format!("new_state schema validation error: {e}"))
        })?;

        let db_schema = load_schema(self).await?;

        if current_state.attribute_order() != db_schema.attribute_order()
            || current_state.attributes() != db_schema.attributes()
        {
            return Err(DataError::NotAllowed.report());
        }

        validate_new_state(&current_state, &new_state, has_content_permission)?;

        db_transaction!(self, move |mut cmds| {
            for attr in new_state.attributes() {
                cmds.common()
                    .profile_attributes()
                    .upsert_profile_attribute(attr.attribute())?;
            }
            cmds.common()
                .profile_attributes()
                .upsert_profile_attributes_order_mode(new_state.attribute_order())
        })?;

        Ok(())
    }
}
