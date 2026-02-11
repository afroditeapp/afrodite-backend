use database::current::write::GetDbWriteCommandsCommon;

use crate::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonProfileAttributes);

impl WriteCommandsCommonProfileAttributes<'_> {
    pub async fn upsert_profile_attributes_hash(&self, hash: &str) -> Result<(), DataError> {
        let hash = hash.to_string();
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .profile_attributes()
                .upsert_profile_attributes_hash(&hash)
        })
    }
}
