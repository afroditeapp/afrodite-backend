use database_media::current::write::GetDbWriteCommandsMedia;
use model::ImageProcessingDynamicConfig;
use server_common::result::Result;
use server_data::{DataError, db_transaction, define_cmd_wrapper_write, write::DbTransaction};

define_cmd_wrapper_write!(WriteCommandsMediaAdminImageProcessingConfig);

impl WriteCommandsMediaAdminImageProcessingConfig<'_> {
    pub async fn upsert_image_processing_config(
        &self,
        config: &ImageProcessingDynamicConfig,
    ) -> Result<(), DataError> {
        let config = config.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.media_admin()
                .image_processing_config()
                .upsert_image_processing_config(&config)
        })
    }
}
