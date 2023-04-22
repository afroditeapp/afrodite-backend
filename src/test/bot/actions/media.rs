
use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login}, profile_api::{post_profile, get_profile, get_default_profile}, media_api::put_image_to_moderation_slot}, models::Profile};
use async_trait::async_trait;
use nalgebra::U8;

use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn};

use super::{super::super::client::{ApiClient, TestError}, BotAction};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt, server::database::file::file::ImageSlot,
};

use super::BotState;




#[derive(Debug)]
pub struct SendImageToSlot(pub u32);

#[async_trait]
impl BotAction for SendImageToSlot {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
       // put_image_to_moderation_slot(state.api.media(), self.0, body)
        Ok(())
    }
}
