use std::fmt::Debug;

use api_client::{
    apis::media_api::put_moderation_request,
    manual_additions::put_image_to_moderation_slot_fixed,
    models::{ContentId, ModerationRequestContent},
};
use async_trait::async_trait;

use error_stack::Result;

use super::{super::super::client::TestError, BotAction};

use crate::{test::bot::utils::image::ImageProvider, utils::IntoReportExt};

use super::BotState;
