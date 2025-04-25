//! Bots for fake clients

use std::{fmt::Debug, iter::Peekable, time::Instant};

use api_client::{
    apis::{
        account_api::get_account_state, chat_api::{
            get_latest_public_key_id, post_add_receiver_acknowledgement, post_add_sender_acknowledgement, post_get_next_received_likes_page, post_reset_received_likes_paging, post_send_like
        }, common_api::get_client_config, profile_api::{
            post_get_query_available_profile_attributes, post_profile, post_search_age_range, post_search_groups
        }
    },
    manual_additions::{get_pending_messages_fixed, get_public_key_fixed, post_add_public_key_fixed, post_send_message_fixed},
    models::{
        AccountId, AttributeMode, ClientId, ClientLocalId, MessageNumber, PendingMessageAcknowledgementList, PendingMessageId, ProfileAttributeQuery, ProfileAttributeValueUpdate, ProfileSearchAgeRange, ProfileUpdate, SearchGroups, SentMessageId, SentMessageIdList
    },
};
use async_trait::async_trait;
use config::bot_config_file::Gender;
use error_stack::{Result, ResultExt};
use simple_backend_utils::UuidBase64Url;
use tracing::warn;

use super::{
    actions::{
        account::{
            AccountState, AssertAccountState, Login, Register, SetAccountSetup, SetProfileVisibility, DEFAULT_AGE
        },
        media::SendImageToSlot,
        profile::{ChangeProfileText, GetProfile, ProfileText, UpdateLocationRandomOrConfigured},
        BotAction, RunActions, RunActionsIf,
    },
    BotState, BotStruct, TaskState,
};
use utils::encrypt::{encrypt_data, generate_keys, unwrap_signed_binary_message};
use crate::{
    action_array,
    bot::actions::{
        account::CompleteAccountSetup, admin::{content::AdminBotContentModerationLogic, profile_text::AdminBotProfileTextModerationLogic}, media::SetContent, profile::ChangeProfileTextDaily, ActionArray
    },
    client::TestError, state::BotEncryptionKeys,
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
        let iter = if state.is_bot_mode_admin_bot() {
            // Admin bot

            const SETUP: ActionArray =
                action_array![Register, Login, DoInitialSetupIfNeeded { admin: true },];
            const ACTION_LOOP: ActionArray = action_array![
                ActionsBeforeIteration,
                AdminBotContentModerationLogic,
                AdminBotProfileTextModerationLogic,
                ActionsAfterIteration,
            ];
            let iter = SETUP
                .iter()
                .copied()
                .chain(ACTION_LOOP.iter().copied().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        } else {
            // User bot

            const SETUP: ActionArray = action_array![
                Register,
                Login,
                DoInitialSetupIfNeeded { admin: false },
                UpdateLocationRandomOrConfigured::new(None),
                SetProfileVisibility(true),
                SendLikeIfNeeded,
            ];
            const ACTION_LOOP: ActionArray = action_array![
                ActionsBeforeIteration,
                GetProfile,
                RunActionsIf(action_array!(UpdateLocationRandomOrConfigured::new(None)), |s| {
                    s.get_bot_config().change_location() && rand::random::<f32>() < 0.2
                }),
                // TODO: Toggle the profile visiblity in the future?
                RunActionsIf(action_array!(SetProfileVisibility(true)), |s| {
                    s.get_bot_config().change_visibility() && rand::random::<f32>() < 0.5
                }),
                RunActionsIf(action_array!(SetProfileVisibility(false)), |s| {
                    s.get_bot_config().change_visibility() && rand::random::<f32>() < 0.1
                }),
                RunActionsIf(action_array!(ChangeProfileTextDaily), |s| {
                    s.get_bot_config().change_profile_text_daily()
                }),
                AcceptReceivedLikesAndSendMessage,
                AnswerReceivedMessages,
                ActionsAfterIteration,
            ];
            let iter = SETUP
                .iter()
                .copied()
                .chain(ACTION_LOOP.iter().copied().cycle());

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

        if !account_state.state.initial_setup_completed.unwrap_or(true) {
            if self.admin {
                SetAccountSetup::admin()
            } else {
                SetAccountSetup::new()
            }
            .excecute_impl_task_state(state, task_state)
            .await?;

            const ACTIONS1: ActionArray = action_array!(
                SendImageToSlot {
                    slot: 0,
                    copy_to_slot: None,
                    mark_copied_image: false,
                },
                SetContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(0),
                },
            );
            RunActions(ACTIONS1)
                .excecute_impl_task_state(state, task_state)
                .await?;
            ChangeBotAgeAndOtherSettings { admin: self.admin }
                .excecute_impl_task_state(state, task_state)
                .await?;
            const ACTIONS2: ActionArray = action_array!(
                SetBotPublicKey,
                CompleteAccountSetup,
                AssertAccountState::account(AccountState::Normal),
            );
            RunActions(ACTIONS2)
                .excecute_impl_task_state(state, task_state)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SetBotPublicKey;

impl SetBotPublicKey {
    async fn setup_bot_keys_if_needed(state: &mut BotState) -> Result<BotEncryptionKeys, TestError> {
        let account_id_string = state.account_id_string()?;
        let latest_public_key_id = get_latest_public_key_id(
            state.api.chat(),
            &account_id_string,
        )
            .await
            .change_context(TestError::ApiRequest)?
            .id
            .flatten()
            .map(|v| v.id);

        if let Some(keys) = state.chat.keys.clone() {
            if latest_public_key_id == Some(keys.public_key_id) {
                return Ok(keys);
            }
        }

        let keys = generate_keys(account_id_string)
            .change_context( TestError::MessageEncryptionError)?;
        let public_key_bytes = keys.public_key_bytes()
            .change_context( TestError::MessageEncryptionError)?;

        let r = post_add_public_key_fixed(
            state.api.chat(),
            public_key_bytes,
        )
        .await
        .change_context(TestError::ApiRequest)?;

        if r.error_too_many_public_keys {
            return Err(TestError::ApiRequest.report())
                .attach_printable("Too many public keys");
        }

        let Some(public_key_id) = r.key_id.flatten().map(|v| v.id) else {
            return Err(TestError::ApiRequest.report())
                .attach_printable("Public key ID not found");
        };

        let keys = BotEncryptionKeys {
            private: keys.private,
            public: keys.public,
            public_key_id,
        };
        state.chat.keys = Some(keys.clone());

        Ok(keys)
    }
}

#[async_trait]
impl BotAction for SetBotPublicKey {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        Self::setup_bot_keys_if_needed(state).await?;
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
        }
        .excecute_impl(state)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ChangeBotAgeAndOtherSettings {
    pub admin: bool,
}

#[async_trait]
impl BotAction for ChangeBotAgeAndOtherSettings {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let bot_config = state.get_bot_config();
        let age = bot_config.age.unwrap_or(DEFAULT_AGE);

        let groups = {
            let man = SearchGroups {
                man_for_man: Some(true),
                man_for_woman: Some(true),
                man_for_non_binary: Some(true),
                ..Default::default()
            };
            let woman = SearchGroups {
                woman_for_man: Some(true),
                woman_for_woman: Some(true),
                woman_for_non_binary: Some(true),
                ..Default::default()
            };
            let non_binary = SearchGroups {
                non_binary_for_man: Some(true),
                non_binary_for_woman: Some(true),
                non_binary_for_non_binary: Some(true),
                ..Default::default()
            };

            match bot_config.gender {
                Some(Gender::Man) => man,
                Some(Gender::Woman) => woman,
                None => match state.bot_id % 3 {
                    0 => man,
                    1 => woman,
                    _ => non_binary,
                },
            }
        };

        let available_attributes = get_client_config(state.api.profile())
            .await
            .change_context(TestError::ApiRequest)?
            .profile_attributes
            .flatten()
            .map(|v| v.attributes)
            .unwrap_or_default();

        let available_attributes = post_get_query_available_profile_attributes(
            state.api.profile(),
            ProfileAttributeQuery {
                values: available_attributes.iter().map(|v| v.id).collect(),
            },
        )
            .await
            .change_context(TestError::ApiRequest)?
            .values
            .into_iter()
            .map(|v| v.a);

        let mut attributes: Vec<ProfileAttributeValueUpdate> = vec![];
        for attribute in available_attributes {
            if attribute.required.unwrap_or_default()
                && attribute.mode == AttributeMode::SelectMultipleFilterMultiple
            {
                let mut select_all = 0;
                for value in attribute.values {
                    select_all |= value.id;
                }

                let update = ProfileAttributeValueUpdate {
                    id: attribute.id,
                    v: vec![select_all],
                };

                attributes.push(update);
            }
        }

        let name = if self.admin {
            format!("Admin bot {}", state.bot_id + 1)
        } else {
            state
                .get_bot_config()
                .name
                .clone()
                .unwrap_or("B".to_string())
        };

        let update = ProfileUpdate {
            name,
            age: age.into(),
            attributes,
            ptext: state
                .get_bot_config()
                .text
                .clone()
                .unwrap_or_default(),
        };

        post_profile(state.api.profile(), update)
            .await
            .change_context(TestError::ApiRequest)?;

        let age_range = ProfileSearchAgeRange { min: 18, max: 99 };

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
        let r = post_reset_received_likes_paging(state.api.chat())
            .await
            .change_context(TestError::ApiRequest)?;
        let session_id = *r.s;

        loop {
            let received_likes =
                post_get_next_received_likes_page(state.api.chat(), session_id.clone())
                    .await
                    .change_context(TestError::ApiRequest)?;

            if received_likes.p.is_empty() {
                break;
            }

            for like in received_likes.p {
                post_send_like(state.api.chat(), like.clone())
                    .await
                    .change_context(TestError::ApiRequest)?;

                let new_msg = "Hello!".to_string();
                send_message(state, like, new_msg).await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AnswerReceivedMessages;

#[async_trait]
impl BotAction for AnswerReceivedMessages {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let messages = get_pending_messages_fixed(state.api.chat())
            .await
            .change_context(TestError::ApiRequest)?;

        if messages.is_empty() {
            return Ok(());
        }

        fn parse_minimal_i64(d: &mut impl Iterator<Item=u8>) -> Option<i64> {
            let count = d.next()?;
            let number: i64 = if count == 1 {
                i8::from_le_bytes([d.next()?]).into()
            } else if count == 2 {
                i16::from_le_bytes([d.next()?, d.next()?]).into()
            } else if count == 4 {
                i32::from_le_bytes([
                    d.next()?,
                    d.next()?,
                    d.next()?,
                    d.next()?,
                ]).into()
            } else if count == 8 {
                i64::from_le_bytes([
                    d.next()?,
                    d.next()?,
                    d.next()?,
                    d.next()?,
                    d.next()?,
                    d.next()?,
                    d.next()?,
                    d.next()?,
                ])
            } else {
                return None;
            };

            Some(number)
        }

        fn parse_account_id(d: &mut impl Iterator<Item=u8>) -> Option<AccountId> {
            let id = d.by_ref().take(16).collect::<Vec<u8>>();
            let id = TryInto::<[u8; 16]>::try_into(id)
                .ok()?;
            let id = UuidBase64Url::from_bytes(id);
            let id = AccountId::new(id.to_string());
            Some(id)
        }

        fn parse_signed_message_data(data: Vec<u8>) -> Option<PendingMessageId> {
            let d = &mut data.iter().copied();
            let sender = parse_account_id(d)?;
            let _ = parse_account_id(d)?;
            let _ = parse_minimal_i64(d)?;
            let _ = parse_minimal_i64(d)?;
            let message_number = parse_minimal_i64(d)?;

            Some(PendingMessageId {
                sender: sender.into(),
                mn: MessageNumber::new(message_number).into(),
            })
        }

        fn parse_messages(messages: &[u8]) -> Option<Vec<PendingMessageId>> {
            let mut list_iterator = messages.iter().copied();
            let mut pending_messages: Vec<PendingMessageId> = vec![];
            while let Some(data_len) = parse_minimal_i64(&mut list_iterator) {
                let data_len = match TryInto::<usize>::try_into(data_len) {
                    Ok(len) => len,
                    Err(_) => break,
                };
                let data = list_iterator
                    .by_ref()
                    .take(data_len)
                    .collect::<Vec<u8>>();
                let data = unwrap_signed_binary_message(&data)
                    .ok()?;
                pending_messages.push(parse_signed_message_data(data)?);
            }

            Some(pending_messages)
        }

        let pending_messages = parse_messages(&messages).ok_or(TestError::MissingValue)?;

        let delete_list = PendingMessageAcknowledgementList { ids: pending_messages.clone() };

        post_add_receiver_acknowledgement(state.api.chat(), delete_list)
            .await
            .change_context(TestError::ApiRequest)?;

        for msg in pending_messages {
            let new_msg = "Hello!".to_string();
            send_message(state, *msg.sender, new_msg).await?;
        }

        Ok(())
    }
}

async fn send_message(
    state: &mut BotState,
    receiver: AccountId,
    msg: String,
) -> Result<(), TestError> {
    let latest_key_id = get_latest_public_key_id(state.api.chat(), &receiver.aid.to_string())
        .await
        .change_context(TestError::ApiRequest)?;

    let latest_key_id = match latest_key_id.id.flatten().map(|v| v.id) {
        Some(value) => value,
        None => {
            warn!("Receiver public key is missing");
            return Ok(());
        }
    };

    let public_key = get_public_key_fixed(state.api.chat(), &receiver.aid.to_string(), latest_key_id)
        .await
        .change_context(TestError::ApiRequest)?;

    let keys = SetBotPublicKey::setup_bot_keys_if_needed(state).await?;

    let mut message_bytes = vec![0]; // Text message
    let len_u16 = msg.len() as u16;
    message_bytes.extend_from_slice(&len_u16.to_le_bytes());
    message_bytes.extend_from_slice(msg.as_bytes());
    let encrypted_bytes = encrypt_data(
        &keys.private,
        public_key,
        &message_bytes,
    )
        .change_context( TestError::MessageEncryptionError)?;

    let mut type_number_and_message = vec![0]; // Message type PGP
    type_number_and_message.extend_from_slice(&encrypted_bytes);

    post_send_message_fixed(
        state.api.chat(),
        keys.public_key_id,
        &receiver.aid.to_string(),
        latest_key_id,
        0,
        0,
        type_number_and_message,
    )
    .await
    .change_context(TestError::ApiRequest)?;

    post_add_sender_acknowledgement(
        state.api.chat(),
        SentMessageIdList {
            ids: vec![SentMessageId {
                c: ClientId::new(0).into(),
                l: ClientLocalId::new(0).into(),
            }],
        },
    )
    .await
    .change_context(TestError::ApiRequest)?;

    Ok(())
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

#[derive(Debug)]
struct SendLikeIfNeeded;

#[async_trait]
impl BotAction for SendLikeIfNeeded {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if let Some(account_id) = state.get_bot_config().send_like_to_account_id {
            let account_id = AccountId::new(account_id.to_string());
            let r = post_send_like(state.api.chat(), account_id).await;
            if r.is_err() {
                warn!(
                    "Sending like failed. Task: {}, Bot: {}",
                    state.task_id, state.bot_id
                );
            }
        }
        Ok(())
    }
}
