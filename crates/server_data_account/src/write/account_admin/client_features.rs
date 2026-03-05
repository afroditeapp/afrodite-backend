use model::{InfoBanner, InfoBannersConfig};
use model_account::SaveInfoBanners;
use server_data::{
    DataError, db_manager::InternalWriting, define_cmd_wrapper_write, result::Result,
    write::GetWriteCommandsCommon,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveInfoBannersResult {
    Updated,
    NotModified,
    ErrorCurrentStateChanged,
}

fn banner_content_changed(current: &InfoBanner, new: &mut InfoBanner) -> bool {
    // Ignore version field
    new.version = current.version;
    current != new
}

fn merge_info_banners(current: &InfoBannersConfig, new: InfoBannersConfig) -> InfoBannersConfig {
    let mut merged = current.clone();

    for (key, mut new_banner) in new.banners.into_iter() {
        if let Some(current_banner) = current.banners.get(&key) {
            if banner_content_changed(current_banner, &mut new_banner) {
                new_banner.version = current_banner.version.wrapping_add(1);
            } else {
                new_banner.version = current_banner.version;
            }
        } else {
            new_banner.version = 0;
        }

        merged.banners.insert(key, new_banner);
    }

    merged
}

define_cmd_wrapper_write!(WriteCommandsAccountClientFeaturesAdmin);

impl WriteCommandsAccountClientFeaturesAdmin<'_> {
    pub async fn save_info_banners(
        &self,
        request: SaveInfoBanners,
    ) -> Result<SaveInfoBannersResult, DataError> {
        let mut dynamic_client_features = self
            .handle()
            .dynamic_client_features()
            .dynamic_client_features()
            .await
            .map(|value| value.config)
            .unwrap_or_default();

        let current_banners = dynamic_client_features.info_banners.unwrap_or_default();

        if request.current.unwrap_or_default() != current_banners {
            return Ok(SaveInfoBannersResult::ErrorCurrentStateChanged);
        }

        let merged = merge_info_banners(&current_banners, request.new);

        if current_banners == merged {
            return Ok(SaveInfoBannersResult::NotModified);
        }

        dynamic_client_features.info_banners = Some(merged);

        self.handle()
            .common()
            .client_config()
            .upsert_dynamic_client_features_config(dynamic_client_features)
            .await?;

        Ok(SaveInfoBannersResult::Updated)
    }
}
