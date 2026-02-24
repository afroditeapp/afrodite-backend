use database::current::read::GetDbReadCommandsCommon;
use model::{Attribute, AttributeOrderMode};

use crate::{
    DataError, IntoDataError, define_cmd_wrapper_read,
    read::DbRead,
    result::{Result, WrappedContextExt},
};

define_cmd_wrapper_read!(ReadCommandsCommonProfileAttributes);

impl ReadCommandsCommonProfileAttributes<'_> {
    pub async fn profile_attributes_hash(&self) -> Result<Option<String>, DataError> {
        self.db_read(|mut cmds| cmds.common().profile_attributes().profile_attributes_hash())
            .await
            .into_error()
    }

    pub async fn all_attributes_from_db(
        &self,
    ) -> Result<(Vec<Attribute>, AttributeOrderMode), DataError> {
        let (raw, order_mode) = self
            .db_read(|mut cmds| {
                let attrs = cmds
                    .common()
                    .profile_attributes()
                    .all_profile_attributes()?;
                let order = cmds.common().profile_attributes().attribute_order_mode()?;
                Ok((attrs, order))
            })
            .await
            .into_error()?;

        let order = order_mode.unwrap_or_default();

        let mut attributes = Vec::with_capacity(raw.len());
        for (attr_id, json) in raw {
            let attr: Attribute = serde_json::from_str(&json).map_err(|e| {
                tracing::error!("Failed to deserialize attribute {attr_id}: {e}");
                DataError::Diesel.report()
            })?;
            attributes.push(attr);
        }

        Ok((attributes, order))
    }
}
