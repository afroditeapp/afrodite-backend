//! Bots for fake clients

use std::{
    fmt::Debug,
    iter::Peekable,
    time::{Duration, Instant},
};

use api_client::{
    apis::{account_api::get_account_state, chat_api::{delete_pending_messages, get_pending_messages, get_received_likes, post_send_like, post_send_message}, profile_api::{get_available_profile_attributes, post_profile, post_search_age_range, post_search_groups}},
    models::{AccountState, AttributeMode, PendingMessageDeleteList, ProfileAttributeValueUpdate, ProfileSearchAgeRange, ProfileUpdate, SearchGroups, SendMessageToAccount},
};
use async_trait::async_trait;
use config::bot_config_file::Gender;
use error_stack::{Result, ResultExt};

use super::{
    actions::{
        account::{AssertAccountState, Login, Register, SetAccountSetup, SetProfileVisibility},
        media::SendImageToSlot,
        profile::{ChangeProfileText, GetProfile, ProfileText, UpdateLocationRandom},
        BotAction, RunActions, RunActionsIf,
    },
    BotState, BotStruct, TaskState,
};
use crate::{
    action_array,
    bot::actions::{
        account::CompleteAccountSetup, admin::ModerateMediaModerationRequest,
        media::{MakeModerationRequest, SetPendingContent}, ActionArray,
    },
    client::TestError,
};

pub struct ClientBot {
    state: BotState,
    actions: Peekable<Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>>,
}

impl Debug for ClientBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ClientBot").finish()
    }
}

impl ClientBot {
    pub fn new(state: BotState) -> Self {
        let iter = if state.is_admin_bot() {
            // Admin bot

            let setup = [
                &Register as &dyn BotAction,
                &Login,
                &DoInitialSetupIfNeeded { admin: true },
            ];
            const MODERATE_INITIAL: ModerateMediaModerationRequest =
                ModerateMediaModerationRequest::moderate_initial_content();
            const MODERATE_ADDITIONAL: ModerateMediaModerationRequest =
                ModerateMediaModerationRequest::moderate_additional_content();
            let action_loop = [
                &ActionsBeforeIteration as &dyn BotAction,
                &MODERATE_INITIAL,
                &MODERATE_ADDITIONAL,
                &ActionsAfterIteration,
            ];
            let iter = setup.into_iter().chain(action_loop.into_iter().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        } else {
            // User bot

            let setup = [
                &Register as &dyn BotAction,
                &Login,
                &DoInitialSetupIfNeeded { admin: false },
                &UpdateLocationRandom(None),
                &SetProfileVisibility(true),
            ];
            let action_loop = [
                &ActionsBeforeIteration as &dyn BotAction,
                &GetProfile,
                &RunActionsIf(
                    action_array!(UpdateLocationRandom(None)),
                    || rand::random::<f32>() < 0.2,
                ),
                // TODO: Toggle the profile visiblity in the future?
                &RunActionsIf(action_array!(SetProfileVisibility(true)), || {
                    rand::random::<f32>() < 0.5
                }),
                &RunActionsIf(action_array!(SetProfileVisibility(false)), || {
                    rand::random::<f32>() < 0.1
                }),
                &AcceptReceivedLikesAndSendMessage,
                &AnswerReceivedMessages,
                &ActionsAfterIteration,
            ];
            let iter = setup.into_iter().chain(action_loop.into_iter().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        };

        Self {
            state,
            actions: iter.peekable(),
        }
    }
}

#[async_trait]
impl BotStruct for ClientBot {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState) {
        (self.actions.peek().copied(), &mut self.state)
    }
    fn next_action(&mut self) {
        self.actions.next();
    }
    fn state(&self) -> &BotState {
        &self.state
    }
}

#[derive(Debug)]
pub struct DoInitialSetupIfNeeded {
    admin: bool,
}

#[async_trait]
impl BotAction for DoInitialSetupIfNeeded {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        let account_state = get_account_state(state.api.account())
            .await
            .change_context(TestError::ApiRequest)?;

        if account_state.state == AccountState::InitialSetup {
            let email = format!("bot{}@example.com", state.bot_id);
            if self.admin {
                SetAccountSetup::admin()
            } else {
                SetAccountSetup {
                    email: Some(&email),
                }
            }
            .excecute_impl_task_state(state, task_state)
            .await?;

            const ACTIONS: ActionArray = action_array!(
                SendImageToSlot {
                    slot: 1,
                    random_if_not_defined_in_config: true,
                    copy_to_slot: Some(0),
                    mark_copied_image: true,
                },
                SetPendingContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(1),
                },
                MakeModerationRequest { slot_0_secure_capture: true },
                ChangeBotAgeAndOtherSettings,
                CompleteAccountSetup,
                AssertAccountState(AccountState::Normal),
            );
            RunActions(ACTIONS)
                .excecute_impl_task_state(state, task_state)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ChangeBotProfileText;

#[async_trait]
impl BotAction for ChangeBotProfileText {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let text = format!(
            "Hello! My location is\n{:#?}",
            state.previous_value.location()
        );

        ChangeProfileText {
            mode: ProfileText::String(text),
        }.excecute_impl(state).await?;

        Ok(())
    }
}

const DEFAULT_AGE: u8 = 30;

#[derive(Debug)]
pub struct ChangeBotAgeAndOtherSettings;

#[async_trait]
impl BotAction for ChangeBotAgeAndOtherSettings {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let (age, groups) = if let Some(bot_config) = state.get_bot_config() {
            (
                bot_config.age.unwrap_or(DEFAULT_AGE),
                match bot_config.img_dir_gender() {
                    Gender::Man => SearchGroups {
                        man_for_man: Some(true),
                        man_for_woman: Some(true),
                        man_for_non_binary: Some(true),
                        ..Default::default()
                    },
                    Gender::Woman => SearchGroups {
                        woman_for_man: Some(true),
                        woman_for_woman: Some(true),
                        woman_for_non_binary: Some(true),
                        ..Default::default()
                    },
                }
            )
        } else {
            (
                DEFAULT_AGE,
                match state.bot_id % 3 {
                    0 => SearchGroups {
                        man_for_man: Some(true),
                        man_for_woman: Some(true),
                        man_for_non_binary: Some(true),
                        ..Default::default()
                    },
                    1 => SearchGroups {
                        woman_for_man: Some(true),
                        woman_for_woman: Some(true),
                        woman_for_non_binary: Some(true),
                        ..Default::default()
                    },
                    _ => SearchGroups {
                        non_binary_for_man: Some(true),
                        non_binary_for_woman: Some(true),
                        non_binary_for_non_binary: Some(true),
                        ..Default::default()
                    },
                }
            )
        };

        let available_attributes = get_available_profile_attributes(state.api.profile())
            .await
            .change_context(TestError::ApiRequest)?
            .info
            .flatten()
            .map(|v| v.attributes)
            .unwrap_or_default();

        let mut attributes: Vec<ProfileAttributeValueUpdate> = vec![];
        for attribute in available_attributes {
            if attribute.required.unwrap_or_default() && attribute.mode == AttributeMode::SelectMultipleFilterMultiple {
                let mut select_all = 0;
                for value in attribute.values {
                    select_all |= value.id;
                }

                let update = ProfileAttributeValueUpdate {
                    id: attribute.id,
                    value_part1: Some(Some(select_all)),
                    value_part2: None,
                };

                attributes.push(update);
            }
        }
        let update = ProfileUpdate {
            name: state.get_bot_config()
                .and_then(|v| v.name.clone())
                .unwrap_or("B".to_string()),
            age: age.into(),
            attributes,
            ..Default::default()
        };

        post_profile(state.api.profile(), update)
            .await
            .change_context(TestError::ApiRequest)?;

        let age_range = ProfileSearchAgeRange {
            min: 18,
            max: 99,
        };

        post_search_age_range(state.api.profile(), age_range)
            .await
            .change_context(TestError::ApiRequest)?;

        post_search_groups(state.api.profile(), groups)
            .await
            .change_context(TestError::ApiRequest)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct AcceptReceivedLikesAndSendMessage;

#[async_trait]
impl BotAction for AcceptReceivedLikesAndSendMessage {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let received_likes = get_received_likes(state.api.chat())
            .await
            .change_context(TestError::ApiRequest)?;

        for like in received_likes.profiles {
            post_send_like(state.api.chat(), like)
                .await
                .change_context(TestError::ApiRequest)?;

            let new_msg = "Hello!".to_string();

            let send_msg = SendMessageToAccount {
                receiver: Box::new(like),
                message: new_msg,
            };

            post_send_message(state.api.chat(), send_msg)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AnswerReceivedMessages;

#[async_trait]
impl BotAction for AnswerReceivedMessages {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let messages = get_pending_messages(state.api.chat())
            .await
            .change_context(TestError::ApiRequest)?;

        if messages.messages.is_empty() {
            return Ok(());
        }

        let messages_ids = messages.messages
            .iter()
            .map(|msg| msg.id.as_ref().clone())
            .collect::<Vec<_>>();

        let delete_list = PendingMessageDeleteList {
            messages_ids,
        };

        delete_pending_messages(state.api.chat(), delete_list)
            .await
            .change_context(TestError::ApiRequest)?;

        for msg in messages.messages {
            let new_msg = "Hello!".to_string();

            let send_msg = SendMessageToAccount {
                receiver: msg.id.account_id_sender,
                message: new_msg,
            };

            post_send_message(state.api.chat(), send_msg)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }
}


#[derive(Debug)]
struct ActionsBeforeIteration;

#[async_trait]
impl BotAction for ActionsBeforeIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        state.benchmark.action_duration = Instant::now();

        Ok(())
    }
}

#[derive(Debug)]
struct ActionsAfterIteration;

#[async_trait]
impl BotAction for ActionsAfterIteration {
    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
}
