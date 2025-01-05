//! Bots for fake clients

use std::{fmt::Debug, iter::Peekable, time::Instant};

use api_client::{
    apis::{
        account_api::get_account_state,
        chat_api::{
            get_public_key, post_add_receiver_acknowledgement, post_add_sender_acknowledgement,
            post_get_next_received_likes_page, post_public_key, post_reset_received_likes_paging,
            post_send_like,
        },
        profile_api::{
            get_available_profile_attributes, post_get_query_available_profile_attributes, post_profile, post_search_age_range, post_search_groups
        },
    },
    manual_additions::{get_pending_messages_fixed, post_send_message_fixed},
    models::{
        AccountId, AttributeMode, ClientId, ClientLocalId, PendingMessage, PendingMessageAcknowledgementList, ProfileAttributeQuery, ProfileAttributeValueUpdate, ProfileSearchAgeRange, ProfileUpdate, PublicKeyData, PublicKeyVersion, SearchGroups, SentMessageId, SentMessageIdList, SetPublicKey
    },
};
use async_trait::async_trait;
use config::bot_config_file::Gender;
use error_stack::{Result, ResultExt};
use tracing::warn;

use super::{
    actions::{
        account::{
            AssertAccountState, Login, Register, SetAccountSetup, SetProfileVisibility, DEFAULT_AGE, AccountState
        },
        media::SendImageToSlot,
        profile::{ChangeProfileText, GetProfile, ProfileText, UpdateLocationRandomOrConfigured},
        BotAction, RunActions, RunActionsIf,
    },
    utils::encrypt::encrypt_data,
    BotState, BotStruct, TaskState,
};
use crate::{
    action_array,
    bot::actions::{
        account::CompleteAccountSetup,
        admin::{profile_text::AdminBotProfileTextModerationLogic, content::AdminBotContentModerationLogic},
        media::SetContent,
        ActionArray,
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

/*
The key is generated with pgp library using following settings:

let mut key_params = SecretKeyParamsBuilder::default();
key_params
    .key_type(KeyType::ECDSA(ECCCurve::P256))
    .can_encrypt(false)
    .can_certify(false)
    .can_sign(true)
    .primary_user_id("User ID".to_string())
    .preferred_symmetric_algorithms(smallvec![
        SymmetricKeyAlgorithm::AES128,
    ])
    .preferred_hash_algorithms(smallvec![
        HashAlgorithm::SHA2_256,
    ])
    .preferred_compression_algorithms(smallvec![])
    .subkey(
        SubkeyParamsBuilder::default()
            .key_type(KeyType::ECDH(ECCCurve::P256))
            .can_authenticate(false)
            .can_certify(false)
            .can_encrypt(true)
            .can_sign(false)
            .build()
            .unwrap()
    );
*/
const BOT_PUBLIC_KEY: &str = "
-----BEGIN PGP PUBLIC KEY BLOCK-----

xlIEZrUithMIKoZIzj0DAQcCAwQPyfhOjBpuNHTfc3RLX2jkK6kPD2awvT1M32Ye
WNb2TnlS/GQkMPiO8FzIM4HeOhH8gCFPF2Zdx4sKPJmliNsyzQdVc2VyIElEwoME
EBMIACsCGQEFAma1IrYCGwICCwcCFQgBFhYhBHZlmH4QZ4iaqMhnaWXPEpaBFVLg
AAoJEGXPEpaBFVLgAQcBALaroQGcjCGJagYl394YnDLgLrU4x65vrMBTkUWJPTlF
AQCMJXAIcJzdAE8granlxSUyECfAOxdav8N0ZEkFY15BMs5WBGa1IrYSCCqGSM49
AwEHAgMEiuP4c3Y99j9iA8KsVGY5a/g1PFCDJCTOi/ISjY4bg5Y3Qt0ZildT8gyo
5h8QUadvRciIFEPe1/5/uaMTuPfD1gMBCAfCeAQYEwgAIAUCZrUitgIbDBYhBHZl
mH4QZ4iaqMhnaWXPEpaBFVLgAAoJEGXPEpaBFVLgZRoA/2h6zKOrtMoqdg07d+yI
pLFJWGK6aPpk4axuljBPjHxSAQCoggKkU+Bf4vFqbwJQuVbh/G+tJG8w0YtF/Jfp
qzmprA==
=xgPw
-----END PGP PUBLIC KEY BLOCK-----
";

const BOT_PRIVATE_KEY: &str = "
-----BEGIN PGP PRIVATE KEY BLOCK-----

xXcEZrUithMIKoZIzj0DAQcCAwQPyfhOjBpuNHTfc3RLX2jkK6kPD2awvT1M32Ye
WNb2TnlS/GQkMPiO8FzIM4HeOhH8gCFPF2Zdx4sKPJmliNsyAAEAhfnLqgKe8T/V
YxvmviGU5dh7r1kUdNpO1f82f4d9pnsSDs0HVXNlciBJRMKDBBATCAArAhkBBQJm
tSK2AhsCAgsHAhUIARYWIQR2ZZh+EGeImqjIZ2llzxKWgRVS4AAKCRBlzxKWgRVS
4AEHAQC2q6EBnIwhiWoGJd/eGJwy4C61OMeub6zAU5FFiT05RQEAjCVwCHCc3QBP
IK2p5cUlMhAnwDsXWr/DdGRJBWNeQTLHewRmtSK2EggqhkjOPQMBBwIDBIrj+HN2
PfY/YgPCrFRmOWv4NTxQgyQkzovyEo2OG4OWN0LdGYpXU/IMqOYfEFGnb0XIiBRD
3tf+f7mjE7j3w9YDAQgHAAEAoIh5mtDadAnwyL/2ZHTYPHwBbGQACc9eqFu3JKOV
hg0P6MJ4BBgTCAAgBQJmtSK2AhsMFiEEdmWYfhBniJqoyGdpZc8SloEVUuAACgkQ
Zc8SloEVUuBlGgD/aHrMo6u0yip2DTt37IiksUlYYrpo+mThrG6WME+MfFIBAKiC
AqRT4F/i8WpvAlC5VuH8b60kbzDRi0X8l+mrOams
=JeHT
-----END PGP PRIVATE KEY BLOCK-----
";

#[derive(Debug)]
pub struct SetBotPublicKey;

#[async_trait]
impl BotAction for SetBotPublicKey {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        post_public_key(
            state.api.chat(),
            SetPublicKey {
                version: PublicKeyVersion::new(1).into(),
                data: PublicKeyData::new(BOT_PUBLIC_KEY.to_string()).into(),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?;

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

        let available_attributes = get_available_profile_attributes(state.api.profile())
            .await
            .change_context(TestError::ApiRequest)?
            .info
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
            ..Default::default()
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

        fn parse_messages(messages: &[u8]) -> Option<Vec<PendingMessage>> {
            let mut list_iterator = messages.iter().cloned();
            let mut pending_messages: Vec<PendingMessage> = vec![];
            loop {
                let pending_message_json_len = [
                    match list_iterator.next() {
                        Some(v) => v,
                        None => break,
                    },
                    list_iterator.next()?,
                ];
                let pending_message_json_len = u16::from_le_bytes(pending_message_json_len);
                let pending_message_json = list_iterator
                    .by_ref()
                    .take(pending_message_json_len.into())
                    .collect::<Vec<u8>>();
                let pending_message: PendingMessage =
                    serde_json::from_slice(&pending_message_json).ok()?;
                pending_messages.push(pending_message);
                let data_len = [list_iterator.next()?, list_iterator.next()?];
                let data_len = u16::from_le_bytes(data_len);
                list_iterator.by_ref().skip(data_len.into()).for_each(drop);
            }

            Some(pending_messages)
        }

        let pending_messages = parse_messages(&messages).ok_or(TestError::MissingValue)?;

        let messages_ids = pending_messages
            .iter()
            .map(|msg| msg.id.as_ref().clone())
            .collect::<Vec<_>>();

        let delete_list = PendingMessageAcknowledgementList { ids: messages_ids };

        post_add_receiver_acknowledgement(state.api.chat(), delete_list)
            .await
            .change_context(TestError::ApiRequest)?;

        for msg in pending_messages {
            let new_msg = "Hello!".to_string();
            send_message(state, *msg.id.sender, new_msg).await?;
        }

        Ok(())
    }
}

async fn send_message(
    state: &mut BotState,
    receiver: AccountId,
    msg: String,
) -> Result<(), TestError> {
    let public_key = get_public_key(state.api.chat(), &receiver.aid.to_string(), 1)
        .await
        .change_context(TestError::ApiRequest)?;

    if let Some(receiver_public_key) = public_key.key.flatten() {
        let mut message_bytes = vec![0]; // Text message
        let len_u16 = msg.len() as u16;
        message_bytes.extend_from_slice(&len_u16.to_le_bytes());
        message_bytes.extend_from_slice(msg.as_bytes());
        let encrypted_bytes = encrypt_data(
            BOT_PRIVATE_KEY,
            &receiver_public_key.data.data,
            &message_bytes,
        )
        .map_err(|e| TestError::MessageEncryptionError(e).report())?;

        let mut type_number_and_message = vec![0]; // Message type PGP
        type_number_and_message.extend_from_slice(&encrypted_bytes);

        post_send_message_fixed(
            state.api.chat(),
            &receiver.aid.to_string(),
            receiver_public_key.id.id,
            receiver_public_key.version.version,
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
    } else {
        warn!("Receiver public key is missing");
    }

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
