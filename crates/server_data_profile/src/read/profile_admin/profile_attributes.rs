use model_profile::{ProfileAttributesInternal, ProfileAttributesSchemaExport};
use server_data::{
    DataError, define_cmd_wrapper_read,
    read::GetReadCommandsCommon,
    result::{Result, WrappedContextExt},
};

define_cmd_wrapper_read!(ReadCommandsProfileAdminAttributeSchema);

impl ReadCommandsProfileAdminAttributeSchema<'_> {
    pub async fn get_schema(&self) -> Result<ProfileAttributesSchemaExport, DataError> {
        let (attributes, order) = self
            .0
            .common()
            .profile_attributes()
            .all_attributes_from_db()
            .await?;

        let schema = ProfileAttributesInternal::from_db_data(attributes, order).map_err(|e| {
            DataError::NotAllowed
                .report()
                .attach_printable(format!("Profile attributes validation error: {e}"))
        })?;

        Ok(schema.export())
    }
}
