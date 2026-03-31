use api_client::models::{
    ContentId, ContentProcessingState, ContentProcessingStateType, ServerMessageType,
};
use model::{
    AdminBotNotificationTypes as InternalAdminBotNotificationTypes, ContentProcessingStateInternal,
    ContentProcessingStateType as InternalContentProcessingStateType, EventToClientInternal,
    parse_server_binary_message,
};

pub type EventType = ServerMessageType;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct EventToClient {
    pub admin_bot_notification: Option<AdminBotNotificationTypes>,
    pub content_processing_state_changed: Option<ContentProcessingStateChanged>,
    pub event: EventType,
}

impl EventToClient {
    pub fn new(event: EventType) -> Self {
        Self {
            admin_bot_notification: None,
            content_processing_state_changed: None,
            event,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AdminBotNotificationTypes {
    pub moderate_initial_media_content_bot: Option<bool>,
    pub moderate_media_content_bot: Option<bool>,
    pub moderate_profile_names_bot: Option<bool>,
    pub moderate_profile_texts_bot: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContentProcessingStateChanged {
    pub id: i64,
    pub new_state: ContentProcessingState,
}

impl ContentProcessingStateChanged {
    pub fn new(id: i64, new_state: ContentProcessingState) -> Self {
        Self { id, new_state }
    }
}

pub fn parse_server_event_to_client_for_test_mode(
    data: &[u8],
) -> std::result::Result<Option<EventToClient>, String> {
    let event = parse_server_binary_message(data)?;
    Ok(convert_server_event_to_client_for_test_mode(event))
}

fn convert_server_event_to_client_for_test_mode(
    event: EventToClientInternal,
) -> Option<EventToClient> {
    match event {
        EventToClientInternal::AdminBotNotification(notification) => {
            let mut event = EventToClient::new(EventType::AdminBotNotification);
            let value = AdminBotNotificationTypes {
                moderate_initial_media_content_bot: Some(notification.contains(
                    InternalAdminBotNotificationTypes::MODERATE_INITIAL_MEDIA_CONTENT_BOT,
                )),
                moderate_media_content_bot: Some(
                    notification
                        .contains(InternalAdminBotNotificationTypes::MODERATE_MEDIA_CONTENT_BOT),
                ),
                moderate_profile_names_bot: Some(
                    notification
                        .contains(InternalAdminBotNotificationTypes::MODERATE_PROFILE_NAMES_BOT),
                ),
                moderate_profile_texts_bot: Some(
                    notification
                        .contains(InternalAdminBotNotificationTypes::MODERATE_PROFILE_TEXTS_BOT),
                ),
            };
            event.admin_bot_notification = Some(value);
            Some(event)
        }
        EventToClientInternal::ContentProcessingStateChanged(state_changed) => {
            let mut event = EventToClient::new(EventType::ContentProcessingStateChanged);
            let value = ContentProcessingStateChanged::new(
                state_changed.id,
                convert_content_processing_state(state_changed.new_state),
            );
            event.content_processing_state_changed = Some(value);
            Some(event)
        }
        EventToClientInternal::AccountStateChanged
        | EventToClientInternal::NewMessageReceived
        | EventToClientInternal::PendingChatNotificationsChanged
        | EventToClientInternal::PendingAppNotificationsChanged
        | EventToClientInternal::ReceivedLikesChanged
        | EventToClientInternal::ClientConfigChanged
        | EventToClientInternal::ProfileChanged
        | EventToClientInternal::ResponseNextProfilePage { .. }
        | EventToClientInternal::NewsChanged
        | EventToClientInternal::MediaContentChanged
        | EventToClientInternal::DailyLikesLeftChanged
        | EventToClientInternal::ScheduledMaintenanceStatus(_)
        | EventToClientInternal::PushNotificationInfoChanged
        | EventToClientInternal::TypingStart(_)
        | EventToClientInternal::TypingStop(_)
        | EventToClientInternal::CheckOnlineStatusResponse(_)
        | EventToClientInternal::MessageDeliveryInfoChanged
        | EventToClientInternal::LatestSeenMessageChanged => None,
    }
}

fn convert_content_processing_state(
    state: ContentProcessingStateInternal,
) -> ContentProcessingState {
    let mut converted =
        ContentProcessingState::new(convert_content_processing_state_type(state.state_type()));

    match state {
        ContentProcessingStateInternal::InQueue {
            wait_queue_position,
        } => {
            converted.wait_queue_position = Some(Some(wait_queue_position));
        }
        ContentProcessingStateInternal::Completed { content_id, fd } => {
            converted.cid = Some(Some(Box::new(ContentId::new(content_id.cid.to_string()))));
            converted.fd = Some(Some(fd));
        }
        ContentProcessingStateInternal::Empty
        | ContentProcessingStateInternal::Processing
        | ContentProcessingStateInternal::Failed
        | ContentProcessingStateInternal::NsfwDetected => (),
    }

    converted
}

fn convert_content_processing_state_type(
    state_type: InternalContentProcessingStateType,
) -> ContentProcessingStateType {
    match state_type {
        InternalContentProcessingStateType::Empty => ContentProcessingStateType::Empty,
        InternalContentProcessingStateType::InQueue => ContentProcessingStateType::InQueue,
        InternalContentProcessingStateType::Processing => ContentProcessingStateType::Processing,
        InternalContentProcessingStateType::Completed => ContentProcessingStateType::Completed,
        InternalContentProcessingStateType::Failed => ContentProcessingStateType::Failed,
        InternalContentProcessingStateType::NsfwDetected => {
            ContentProcessingStateType::NsfwDetected
        }
    }
}
