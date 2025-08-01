use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper};
use utoipa::ToSchema;

use crate::{AccountIdDb, AccountIdInternal};

/// Message ID for identifying a message in a conversation.
///
/// The ID is conversation specific and it increments.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct MessageId {
    pub id: i64,
}

impl MessageId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(MessageId);

#[derive(Debug, Clone, Copy)]
pub enum AccountInteractionStateError {
    WrongStateNumber(i64),
    Transition {
        from: AccountInteractionState,
        to: AccountInteractionState,
    },
}
impl AccountInteractionStateError {
    pub fn wrong_state_number(number: i64) -> Self {
        Self::WrongStateNumber(number)
    }
    pub fn transition(from: AccountInteractionState, to: AccountInteractionState) -> Self {
        Self::Transition { from, to }
    }
}
impl std::fmt::Display for AccountInteractionStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountInteractionStateError::WrongStateNumber(number) => {
                write!(f, "Wrong state number: {number}")
            }
            AccountInteractionStateError::Transition { from, to } => {
                write!(f, "State transition from {from:?} to {to:?} is not allowed")
            }
        }
    }
}
impl std::error::Error for AccountInteractionStateError {}

/// Account interaction states
///
/// Possible state transitions:
/// - Empty -> Like -> Match
/// - Like -> Empty
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Deserialize,
    Serialize,
    ToSchema,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub enum AccountInteractionState {
    Empty = 0,
    Like = 1,
    Match = 2,
}

impl TryFrom<i64> for AccountInteractionState {
    type Error = AccountInteractionStateError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Like),
            2 => Ok(Self::Match),
            _ => Err(AccountInteractionStateError::WrongStateNumber(value)),
        }
    }
}

diesel_i64_try_from!(AccountInteractionState);

#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct MatchId {
    pub id: i64,
}

impl MatchId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    /// Return new incremented value using `saturated_add`.
    pub fn increment(&self) -> Self {
        Self {
            id: self.id.saturating_add(1),
        }
    }

    /// This returns -1 if ID is not incremented.
    pub fn next_id_to_latest_used_id(&self) -> Self {
        Self { id: self.id - 1 }
    }
}

diesel_i64_wrapper!(MatchId);

impl From<MatchId> for i64 {
    fn from(value: MatchId) -> Self {
        value.id
    }
}

#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct ReceivedLikeId {
    pub id: i64,
}

impl ReceivedLikeId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    /// Return new incremented value using `saturated_add`.
    pub fn increment(&self) -> Self {
        Self {
            id: self.id.saturating_add(1),
        }
    }

    /// This returns -1 if ID is not incremented.
    pub fn next_id_to_latest_used_id(&self) -> Self {
        Self { id: self.id - 1 }
    }
}

diesel_i64_wrapper!(ReceivedLikeId);

impl From<ReceivedLikeId> for i64 {
    fn from(value: ReceivedLikeId) -> Self {
        value.id
    }
}

/// Account specific conversation ID
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    PartialEq,
    FromSqlRow,
    AsExpression,
    ToSchema,
)]
#[diesel(sql_type = BigInt)]
pub struct ConversationId {
    pub id: i64,
}

impl ConversationId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    /// Return new incremented value using `saturated_add`.
    pub fn increment(&self) -> Self {
        Self {
            id: self.id.saturating_add(1),
        }
    }

    /// This returns -1 if ID is not incremented.
    pub fn next_id_to_latest_used_id(&self) -> Self {
        Self { id: self.id - 1 }
    }
}

diesel_i64_wrapper!(ConversationId);

impl From<ConversationId> for i64 {
    fn from(value: ConversationId) -> Self {
        value.id
    }
}

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::account_interaction)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct AccountInteractionInternal {
    pub id: i64,
    pub state_number: AccountInteractionState,
    pub account_id_sender: Option<AccountIdDb>,
    pub account_id_receiver: Option<AccountIdDb>,
    pub account_id_block_sender: Option<AccountIdDb>,
    pub account_id_block_receiver: Option<AccountIdDb>,
    pub two_way_block: bool,
    /// Message counter for [Self::account_id_sender] which increments for each
    /// message. The counter does not reset. Zero means that no messages are
    /// sent.
    pub message_counter_sender: i64,
    /// Message counter for [Self::account_id_receiver] which increments for each
    /// message. The counter does not reset. Zero means that no messages are
    /// sent.
    pub message_counter_receiver: i64,
    pub included_in_received_new_likes_count: bool,
    pub received_like_id: Option<ReceivedLikeId>,
    pub match_id: Option<MatchId>,
    account_id_previous_like_deleter_slot_0: Option<AccountIdDb>,
    account_id_previous_like_deleter_slot_1: Option<AccountIdDb>,
    conversation_id_sender: Option<ConversationId>,
    conversation_id_receiver: Option<ConversationId>,
}

impl AccountInteractionInternal {
    pub fn try_into_like(
        self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
        received_like_id: ReceivedLikeId,
    ) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Like;
        let state = self.state_number;
        match state {
            AccountInteractionState::Empty => Ok(Self {
                state_number: target,
                account_id_sender: Some(id_like_sender.into_db_id()),
                account_id_receiver: Some(id_like_receiver.into_db_id()),
                included_in_received_new_likes_count: !self.is_blocked(),
                received_like_id: Some(received_like_id),
                ..self
            }),
            AccountInteractionState::Like => Ok(self),
            AccountInteractionState::Match => {
                Err(AccountInteractionStateError::transition(state, target))
            }
        }
    }

    pub fn try_into_match(
        self,
        match_id: MatchId,
        (account, conversation_id1): (AccountIdInternal, ConversationId),
        conversation_id2: ConversationId,
    ) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Match;
        let state = self.state_number;
        match state {
            AccountInteractionState::Like => {
                let (sender, receiver) = if self.account_id_sender == Some(account.into_db_id()) {
                    (conversation_id1, conversation_id2)
                } else {
                    (conversation_id2, conversation_id1)
                };

                Ok(Self {
                    state_number: target,
                    included_in_received_new_likes_count: false,
                    received_like_id: None,
                    match_id: Some(match_id),
                    conversation_id_sender: Some(sender),
                    conversation_id_receiver: Some(receiver),
                    ..self
                })
            }
            AccountInteractionState::Match => Ok(self),
            AccountInteractionState::Empty => {
                Err(AccountInteractionStateError::transition(state, target))
            }
        }
    }

    pub fn try_into_empty(self) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Empty;
        let state = self.state_number;
        match state {
            AccountInteractionState::Like => Ok(Self {
                state_number: target,
                account_id_sender: None,
                account_id_receiver: None,
                included_in_received_new_likes_count: false,
                received_like_id: None,
                ..self
            }),
            AccountInteractionState::Empty => Ok(self),
            AccountInteractionState::Match => {
                Err(AccountInteractionStateError::transition(state, target))
            }
        }
    }

    #[allow(clippy::if_same_then_else)]
    pub fn add_block(
        self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Self {
        if self.account_id_block_sender == Some(id_block_sender.into_db_id())
            && self.account_id_block_receiver == Some(id_block_receiver.into_db_id())
        {
            // Already blocked
            self
        } else if self.account_id_block_sender == Some(id_block_receiver.into_db_id())
            && self.account_id_block_receiver == Some(id_block_sender.into_db_id())
            && self.two_way_block
        {
            // Already blocked
            self
        } else if self.account_id_block_sender == Some(id_block_receiver.into_db_id())
            && self.account_id_block_receiver == Some(id_block_sender.into_db_id())
        {
            Self {
                two_way_block: true,
                ..self
            }
        } else {
            Self {
                account_id_block_sender: Some(id_block_sender.into_db_id()),
                account_id_block_receiver: Some(id_block_receiver.into_db_id()),
                ..self
            }
        }
    }

    pub fn delete_block(
        self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Self {
        if self.account_id_block_sender == Some(id_block_sender.into_db_id())
            && self.account_id_block_receiver == Some(id_block_receiver.into_db_id())
        {
            // Block detected
            if self.two_way_block {
                Self {
                    account_id_block_sender: Some(id_block_receiver.into_db_id()),
                    account_id_block_receiver: Some(id_block_sender.into_db_id()),
                    two_way_block: false,
                    ..self
                }
            } else {
                Self {
                    account_id_block_sender: None,
                    account_id_block_receiver: None,
                    ..self
                }
            }
        } else if self.account_id_block_sender == Some(id_block_receiver.into_db_id())
            && self.account_id_block_receiver == Some(id_block_sender.into_db_id())
            && self.two_way_block
        {
            // Block detected
            Self {
                two_way_block: false,
                ..self
            }
        } else {
            // No block
            self
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state_number == AccountInteractionState::Empty
    }

    pub fn is_like(&self) -> bool {
        self.state_number == AccountInteractionState::Like
    }

    pub fn is_match(&self) -> bool {
        self.state_number == AccountInteractionState::Match
    }

    /// Return true if another or both have blocked each other
    pub fn is_blocked(&self) -> bool {
        self.account_id_block_sender.is_some()
    }

    #[allow(clippy::if_same_then_else)]
    pub fn is_direction_blocked(
        &self,
        id_block_sender: impl Into<AccountIdDb> + Copy,
        id_block_receiver: impl Into<AccountIdDb> + Copy,
    ) -> bool {
        if self.account_id_block_sender == Some(id_block_sender.into())
            && self.account_id_block_receiver == Some(id_block_receiver.into())
        {
            // Already blocked
            true
        } else if self.account_id_block_sender == Some(id_block_receiver.into())
            && self.account_id_block_receiver == Some(id_block_sender.into())
            && self.two_way_block
        {
            // Already blocked
            true
        } else {
            false
        }
    }

    pub fn is_direction_liked(
        &self,
        id_like_sender: impl Into<AccountIdDb> + Copy,
        id_like_receiver: impl Into<AccountIdDb> + Copy,
    ) -> bool {
        if self.is_match() {
            true
        } else {
            self.is_like()
                && self.account_id_sender == Some(id_like_sender.into())
                && self.account_id_receiver == Some(id_like_receiver.into())
        }
    }

    pub fn set_previous_like_deleter_if_slot_available(
        &mut self,
        id_like_deleter: AccountIdInternal,
    ) {
        if self.account_already_deleted_like(id_like_deleter) {
            // Skip
        } else if self.account_id_previous_like_deleter_slot_0.is_none() {
            self.account_id_previous_like_deleter_slot_0 = Some(id_like_deleter.into_db_id());
        } else if self.account_id_previous_like_deleter_slot_1.is_none() {
            self.account_id_previous_like_deleter_slot_1 = Some(id_like_deleter.into_db_id());
        }
    }

    pub fn account_already_deleted_like(&self, id_like_deleter: AccountIdInternal) -> bool {
        self.account_id_previous_like_deleter_slot_0 == Some(id_like_deleter.into_db_id())
            || self.account_id_previous_like_deleter_slot_1 == Some(id_like_deleter.into_db_id())
    }

    /// Total sent messages for [Self::message_counter_sender] and
    /// [Self::message_counter_receiver].
    pub fn message_counter(&self) -> i64 {
        self.message_counter_receiver
            .saturating_add(self.message_counter_sender)
    }

    /// Skip message ID 0 to make possible to use that as initial value
    /// for latest viewed message.
    pub fn next_message_id(&self) -> MessageId {
        MessageId::new(self.message_counter().saturating_add(1))
    }

    pub fn message_count_for_account(&self, account: impl Into<AccountIdDb>) -> i64 {
        let account = account.into();
        if self.account_id_sender == Some(account) {
            self.message_counter_sender
        } else if self.account_id_receiver == Some(account) {
            self.message_counter_receiver
        } else {
            0
        }
    }

    pub fn conversation_id_for_account(
        &self,
        account: impl Into<AccountIdDb>,
    ) -> Option<ConversationId> {
        let account = account.into();
        if self.account_id_sender == Some(account) {
            self.conversation_id_sender
        } else if self.account_id_receiver == Some(account) {
            self.conversation_id_receiver
        } else {
            None
        }
    }
}

#[derive(Default, Serialize, ToSchema)]
pub struct GetConversationId {
    pub value: Option<ConversationId>,
}
