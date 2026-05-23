use api_client::models::{ContentId, ContentProcessingState, ContentProcessingStateType};
pub use model::AdminBotConfigWarningFlags;
use model::{
    AdminBotNotificationTypes as InternalAdminBotNotificationTypes, ContentProcessingStateInternal,
    ContentProcessingStateType as InternalContentProcessingStateType, EventToClientInternal,
    ResponseNextProfilePageStatus as InternalResponseNextProfilePageStatus,
    ResponseResetProfilePagingStatus as InternalResponseResetProfilePagingStatus,
    ServerMessageType, parse_server_binary_message,
};

pub type EventType = ServerMessageType;

#[derive(Clone, Debug, PartialEq)]
pub struct EventToClient {
    pub admin_bot_notification: Option<AdminBotNotificationTypes>,
    pub content_processing_state_changed: Option<ContentProcessingStateChanged>,
    pub request_admin_bot_config_warnings: Option<RequestAdminBotConfigWarnings>,
    pub response_reset_profile_paging: Option<ResponseResetProfilePaging>,
    pub response_next_profile_page: Option<ResponseNextProfilePage>,
    pub event: EventType,
}

impl EventToClient {
    pub fn new(event: EventType) -> Self {
        Self {
            admin_bot_notification: None,
            content_processing_state_changed: None,
            request_admin_bot_config_warnings: None,
            response_reset_profile_paging: None,
            response_next_profile_page: None,
            event,
        }
    }

    pub fn should_be_forwarded_when_events_disabled(&self) -> bool {
        self.admin_bot_notification.is_some()
            || self.request_admin_bot_config_warnings.is_some()
            || self.response_reset_profile_paging.is_some()
            || self.response_next_profile_page.is_some()
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AdminBotNotificationTypes {
    pub moderate_initial_media_content_bot: Option<bool>,
    pub moderate_media_content_bot: Option<bool>,
    pub moderate_profile_names_bot: Option<bool>,
    pub moderate_profile_texts_bot: Option<bool>,
    pub verify_media_content_face_bot: Option<bool>,
    pub verify_account_bot: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContentProcessingStateChanged {
    pub processing_id_from_client: u8,
    pub new_state: ContentProcessingState,
}

impl ContentProcessingStateChanged {
    pub fn new(processing_id_from_client: u8, new_state: ContentProcessingState) -> Self {
        Self {
            processing_id_from_client,
            new_state,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ResponseNextProfilePage {
    pub request_id: u8,
    pub success: bool,
    pub profiles: Vec<String>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ResponseResetProfilePaging {
    pub request_id: u8,
    pub success: bool,
    pub iterator_session_id: Option<i64>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct RequestAdminBotConfigWarnings {
    pub request_id: u8,
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
                verify_media_content_face_bot: Some(
                    notification
                        .contains(InternalAdminBotNotificationTypes::VERIFY_MEDIA_CONTENT_FACE_BOT),
                ),
                verify_account_bot: Some(
                    notification.contains(InternalAdminBotNotificationTypes::VERIFY_ACCOUNT_BOT),
                ),
            };
            event.admin_bot_notification = Some(value);
            Some(event)
        }
        EventToClientInternal::ContentProcessingStateChanged(state_changed) => {
            let mut event = EventToClient::new(EventType::ContentProcessingStateChanged);
            let value = ContentProcessingStateChanged::new(
                state_changed.processing_id_from_client,
                convert_content_processing_state(state_changed.new_state),
            );
            event.content_processing_state_changed = Some(value);
            Some(event)
        }
        EventToClientInternal::RequestAdminBotConfigWarnings { request_id } => {
            let mut event = EventToClient::new(EventType::RequestAdminBotConfigWarnings);
            event.request_admin_bot_config_warnings =
                Some(RequestAdminBotConfigWarnings { request_id });
            Some(event)
        }
        EventToClientInternal::ResponseResetProfilePaging {
            request_id,
            status,
            iterator_session_id,
        } => {
            let mut event = EventToClient::new(EventType::ResponseResetProfilePaging);
            event.response_reset_profile_paging = Some(ResponseResetProfilePaging {
                request_id,
                success: matches!(status, InternalResponseResetProfilePagingStatus::Success),
                iterator_session_id,
            });
            Some(event)
        }
        EventToClientInternal::ResponseNextProfilePage {
            request_id,
            status,
            profiles,
        } => {
            let mut event = EventToClient::new(EventType::ResponseNextProfilePage);
            event.response_next_profile_page = Some(ResponseNextProfilePage {
                request_id,
                success: matches!(status, InternalResponseNextProfilePageStatus::Success),
                profiles: profiles
                    .into_iter()
                    .map(|profile| profile.account_id().to_string())
                    .collect(),
            });
            Some(event)
        }
        EventToClientInternal::AccountStateChanged
        | EventToClientInternal::NewMessageReceived
        | EventToClientInternal::PendingChatNotificationsChanged
        | EventToClientInternal::PendingAppNotificationsChanged
        | EventToClientInternal::WebSocketConnectionAttemptsRemaining { .. }
        | EventToClientInternal::AppUpdateAvailable
        | EventToClientInternal::ReceivedLikesChanged
        | EventToClientInternal::ClientConfigChanged
        | EventToClientInternal::ProfileChanged
        | EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging { .. }
        | EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage { .. }
        | EventToClientInternal::NewsChanged
        | EventToClientInternal::MediaContentChanged
        | EventToClientInternal::AccountVerificationQueuePositionChanged { .. }
        | EventToClientInternal::DailyLikesLeftChanged
        | EventToClientInternal::ScheduledMaintenanceStatus(_)
        | EventToClientInternal::PushNotificationInfoChanged
        | EventToClientInternal::TypingStart(_)
        | EventToClientInternal::TypingStop(_)
        | EventToClientInternal::OnlineStatusUpdated(_)
        | EventToClientInternal::MessageDeliveryInfoChanged
        | EventToClientInternal::LatestSeenMessageChanged => None,
    }
}

fn convert_content_processing_state(
    state: ContentProcessingStateInternal,
) -> ContentProcessingState {
    let mut converted = ContentProcessingState::new();
    converted.state = Some(Some(convert_content_processing_state_type(
        state.state_type(),
    )));

    match state {
        ContentProcessingStateInternal::InQueue {
            wait_queue_position,
        } => {
            converted.wait_queue_position = Some(Some(wait_queue_position));
        }
        ContentProcessingStateInternal::Completed {
            content_id,
            face_detected,
        } => {
            converted.cid = Some(Some(Box::new(ContentId::new(content_id.cid.to_string()))));
            converted.face_detected = Some(Some(face_detected));
        }
        ContentProcessingStateInternal::Processing
        | ContentProcessingStateInternal::Failed
        | ContentProcessingStateInternal::NsfwDetected => (),
    }

    converted
}

fn convert_content_processing_state_type(
    state_type: InternalContentProcessingStateType,
) -> ContentProcessingStateType {
    match state_type {
        InternalContentProcessingStateType::InQueue => ContentProcessingStateType::InQueue,
        InternalContentProcessingStateType::Processing => ContentProcessingStateType::Processing,
        InternalContentProcessingStateType::Completed => ContentProcessingStateType::Completed,
        InternalContentProcessingStateType::Failed => ContentProcessingStateType::Failed,
        InternalContentProcessingStateType::NsfwDetected => {
            ContentProcessingStateType::NsfwDetected
        }
    }
}
