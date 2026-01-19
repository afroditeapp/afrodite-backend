use database::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::ImageProcessingDynamicConfig;
use simple_backend_utils::db::MyRunQueryDsl;

define_current_write_commands!(CurrentWriteMediaAdminImageProcessingConfig);

impl CurrentWriteMediaAdminImageProcessingConfig<'_> {
    pub fn upsert_image_processing_config(
        &mut self,
        config: &ImageProcessingDynamicConfig,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::image_processing_config::dsl::*;

        insert_into(image_processing_config)
            .values((
                row_type.eq(0),
                seetaface_threshold.eq(config.seetaface_threshold),
                nsfw_threshold_drawings.eq(config.nsfw_thresholds.drawings),
                nsfw_threshold_hentai.eq(config.nsfw_thresholds.hentai),
                nsfw_threshold_neutral.eq(config.nsfw_thresholds.neutral),
                nsfw_threshold_porn.eq(config.nsfw_thresholds.porn),
                nsfw_threshold_sexy.eq(config.nsfw_thresholds.sexy),
            ))
            .on_conflict(row_type)
            .do_update()
            .set((
                seetaface_threshold.eq(config.seetaface_threshold),
                nsfw_threshold_drawings.eq(config.nsfw_thresholds.drawings),
                nsfw_threshold_hentai.eq(config.nsfw_thresholds.hentai),
                nsfw_threshold_neutral.eq(config.nsfw_thresholds.neutral),
                nsfw_threshold_porn.eq(config.nsfw_thresholds.porn),
                nsfw_threshold_sexy.eq(config.nsfw_thresholds.sexy),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
