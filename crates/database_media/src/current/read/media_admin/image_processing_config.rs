use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::ImageProcessingDynamicConfig;

define_current_read_commands!(CurrentReadMediaAdminImageProcessingConfig);

impl CurrentReadMediaAdminImageProcessingConfig<'_> {
    pub fn image_processing_config(
        &mut self,
    ) -> Result<Option<ImageProcessingDynamicConfig>, DieselDatabaseError> {
        use model::schema::image_processing_config::dsl::*;

        #[allow(clippy::type_complexity)]
        image_processing_config
            .filter(row_type.eq(0))
            .select((
                seetaface_threshold,
                nsfw_threshold_drawings,
                nsfw_threshold_hentai,
                nsfw_threshold_neutral,
                nsfw_threshold_porn,
                nsfw_threshold_sexy,
            ))
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
            .map(|opt| {
                opt.map(
                    |(seetaface, drawings, hentai, neutral, porn, sexy): (
                        Option<f64>,
                        Option<f64>,
                        Option<f64>,
                        Option<f64>,
                        Option<f64>,
                        Option<f64>,
                    )| ImageProcessingDynamicConfig {
                        seetaface_threshold: seetaface,
                        nsfw_thresholds: simple_backend_model::NsfwDetectionThresholds {
                            drawings,
                            hentai,
                            neutral,
                            porn,
                            sexy,
                        },
                    },
                )
            })
    }
}
