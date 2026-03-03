use database::current::read::GetDbReadCommandsCommon;
use model::{Attribute, AttributeOrderMode};

use crate::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonProfileAttributes);

impl ReadCommandsCommonProfileAttributes<'_> {
    pub async fn all_attributes_from_db(
        &self,
    ) -> Result<(Vec<Attribute>, AttributeOrderMode), DataError> {
        let (attributes, order_mode) = self
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

        Ok((attributes, order))
    }
}
