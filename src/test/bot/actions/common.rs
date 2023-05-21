use std::{fmt::Debug, collections::HashSet};

use api_client::{apis::profile_api::{post_profile, self}, models::{ProfileUpdate, Location}};
use async_trait::async_trait;
use error_stack::Result;

use super::{super::super::client::TestError, BotAction, PreviousValue};

use crate::{utils::IntoReportExt, config::file::LocationConfig};

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
