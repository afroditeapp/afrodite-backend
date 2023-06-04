use std::{collections::HashSet, fmt::Debug};

use api_client::{
    apis::profile_api::{self, post_profile},
    models::{Location, ProfileUpdate},
};
use async_trait::async_trait;
use error_stack::Result;

use super::{super::super::client::TestError, BotAction, PreviousValue};

use crate::{config::file::LocationConfig, utils::IntoReportExt};

use super::BotState;

#[derive(Debug)]
pub struct TestWebSocket;

#[async_trait]
impl BotAction for TestWebSocket {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        // TODO: get new refresh token and API key
        Ok(())
    }
}
