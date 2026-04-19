use utils::minimal_i64;

use crate::{
    AccountId, ContentProcessingStateChanged, ContentProcessingStateInternal,
    ContentProcessingStateType, EventToClientInternal, LastSeenTime, OnlineStatusUpdate,
    ProfileContentVersion, ProfileLink, ProfileVersion, ResponseNextProfilePageStatus,
    ResponseResetProfilePagingStatus, ScheduledMaintenanceStatus, ServerMessageType, UnixTime,
};

pub fn parse_server_binary_message(message: &[u8]) -> Result<EventToClientInternal, String> {
    let mut message_iter = message.iter().copied();
    let message_type_u8 = message_iter
        .next()
        .ok_or_else(|| "missing server message type byte".to_owned())?;

    let message_type = ServerMessageType::try_from(message_type_u8)
        .map_err(|_| format!("unsupported server message type {message_type_u8}"))?;

    let event = match message_type {
        ServerMessageType::PendingAppNotificationsChanged => {
            EventToClientInternal::PendingAppNotificationsChanged
        }
        ServerMessageType::ClientConfigChanged => EventToClientInternal::ClientConfigChanged,
        ServerMessageType::NewsCountChanged => EventToClientInternal::NewsChanged,
        ServerMessageType::ScheduledMaintenanceStatus => {
            EventToClientInternal::ScheduledMaintenanceStatus(
                parse_scheduled_maintenance_status_payload(&mut message_iter)?,
            )
        }
        ServerMessageType::AdminBotNotification => {
            let bits = next_payload_byte(&mut message_iter, "admin bot notification")?;
            EventToClientInternal::AdminBotNotification(
                crate::AdminBotNotificationTypes::from_bits_truncate(bits),
            )
        }
        ServerMessageType::RequestAdminBotConfigWarnings => {
            let request_id = next_payload_byte(
                &mut message_iter,
                "request admin bot config warnings request id",
            )?;
            EventToClientInternal::RequestAdminBotConfigWarnings { request_id }
        }
        ServerMessageType::WebSocketConnectionAttemptsRemaining => {
            let remaining =
                next_payload_byte(&mut message_iter, "websocket connection attempts remaining")?;
            EventToClientInternal::WebSocketConnectionAttemptsRemaining { remaining }
        }
        ServerMessageType::PushNotificationInfoChanged => {
            EventToClientInternal::PushNotificationInfoChanged
        }
        ServerMessageType::AccountStateChanged => EventToClientInternal::AccountStateChanged,
        ServerMessageType::ProfileChanged => EventToClientInternal::ProfileChanged,
        ServerMessageType::ResponseResetProfilePaging => {
            let (request_id, status, iterator_session_id) =
                parse_response_reset_profile_paging_payload(&mut message_iter)?;
            EventToClientInternal::ResponseResetProfilePaging {
                request_id,
                status,
                iterator_session_id,
            }
        }
        ServerMessageType::ResponseNextProfilePage => {
            let (request_id, status, profiles) =
                parse_response_next_profile_page_payload(&mut message_iter)?;
            EventToClientInternal::ResponseNextProfilePage {
                request_id,
                status,
                profiles,
            }
        }
        ServerMessageType::ResponseAutomaticProfileSearchResetProfilePaging => {
            let (request_id, status, iterator_session_id) =
                parse_response_reset_profile_paging_payload(&mut message_iter)?;
            EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
                request_id,
                status,
                iterator_session_id,
            }
        }
        ServerMessageType::ResponseAutomaticProfileSearchNextProfilePage => {
            let (request_id, status, profiles) =
                parse_response_next_profile_page_payload(&mut message_iter)?;
            EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
                request_id,
                status,
                profiles,
            }
        }
        ServerMessageType::ContentProcessingStateChanged => {
            EventToClientInternal::ContentProcessingStateChanged(
                parse_content_processing_state_changed_payload(&mut message_iter)?,
            )
        }
        ServerMessageType::MediaContentChanged => EventToClientInternal::MediaContentChanged,
        ServerMessageType::NewMessageReceived => EventToClientInternal::NewMessageReceived,
        ServerMessageType::PendingChatNotificationsChanged => {
            EventToClientInternal::PendingChatNotificationsChanged
        }
        ServerMessageType::ReceivedLikesChanged => EventToClientInternal::ReceivedLikesChanged,
        ServerMessageType::DailyLikesLeftChanged => EventToClientInternal::DailyLikesLeftChanged,
        ServerMessageType::TypingStart => {
            EventToClientInternal::TypingStart(parse_account_id_payload(&mut message_iter)?)
        }
        ServerMessageType::TypingStop => {
            EventToClientInternal::TypingStop(parse_account_id_payload(&mut message_iter)?)
        }
        ServerMessageType::OnlineStatusUpdated => EventToClientInternal::OnlineStatusUpdated(
            parse_online_status_updated_payload(&mut message_iter)?,
        ),
        ServerMessageType::MessageDeliveryInfoChanged => {
            EventToClientInternal::MessageDeliveryInfoChanged
        }
        ServerMessageType::LatestSeenMessageChanged => {
            EventToClientInternal::LatestSeenMessageChanged
        }
    };

    ensure_payload_fully_consumed(&mut message_iter, message_type)?;

    Ok(event)
}

fn ensure_payload_fully_consumed(
    payload_iter: &mut impl Iterator<Item = u8>,
    message_type: ServerMessageType,
) -> Result<(), String> {
    if payload_iter.next().is_some() {
        return Err(format!("unexpected trailing payload for {message_type:?}"));
    }

    Ok(())
}

fn next_payload_byte(
    payload_iter: &mut impl Iterator<Item = u8>,
    payload_name: &'static str,
) -> Result<u8, String> {
    payload_iter
        .next()
        .ok_or_else(|| format!("missing {payload_name} payload"))
}

fn parse_uuid_base64_url_from_iter(
    payload_iter: &mut impl Iterator<Item = u8>,
    payload_name: &'static str,
) -> Result<simple_backend_utils::UuidBase64Url, String> {
    let first_byte = next_payload_byte(payload_iter, payload_name)?;
    parse_uuid_base64_url_from_iter_with_first_byte(first_byte, payload_iter, payload_name)
}

fn parse_uuid_base64_url_from_iter_with_first_byte(
    first_byte: u8,
    payload_iter: &mut impl Iterator<Item = u8>,
    payload_name: &'static str,
) -> Result<simple_backend_utils::UuidBase64Url, String> {
    let mut bytes = [0u8; 16];
    bytes[0] = first_byte;
    for byte in bytes.iter_mut().skip(1) {
        *byte = payload_iter
            .next()
            .ok_or_else(|| format!("truncated {payload_name} payload"))?;
    }

    Ok(simple_backend_utils::UuidBase64Url::from_bytes(bytes))
}

fn parse_account_id_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<AccountId, String> {
    let account_id = parse_uuid_base64_url_from_iter(payload_iter, "account id")?;

    Ok(AccountId::new_base_64_url(account_id))
}

fn parse_scheduled_maintenance_status_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<ScheduledMaintenanceStatus, String> {
    let admin_bot_offline_raw =
        next_payload_byte(payload_iter, "scheduled maintenance admin_bot_offline")?;

    let start = parse_optional_minimal_i64_value(payload_iter)?;
    let end = if start.is_some() {
        parse_optional_minimal_i64_value(payload_iter)?
    } else {
        None
    };

    let mut status = ScheduledMaintenanceStatus::default();
    status.set_admin_bot_offline(admin_bot_offline_raw != 0);
    status.set_maintenance_time(start.map(UnixTime::new), end.map(UnixTime::new));

    Ok(status)
}

fn parse_minimal_i64_value(payload_iter: &mut impl Iterator<Item = u8>) -> Result<i64, String> {
    parse_minimal_i64_value_with_context(payload_iter, "invalid or truncated minimal i64 payload")
}

fn parse_minimal_i64_value_with_context(
    payload_iter: &mut impl Iterator<Item = u8>,
    error_context: &'static str,
) -> Result<i64, String> {
    minimal_i64::parse_minimal_i64_from_iter(payload_iter).ok_or_else(|| error_context.to_owned())
}

fn parse_minimal_i64_value_from_marker(
    marker: u8,
    payload_iter: &mut impl Iterator<Item = u8>,
    error_context: &'static str,
) -> Result<i64, String> {
    let mut iter_with_marker = std::iter::once(marker).chain(payload_iter.by_ref());
    parse_minimal_i64_value_with_context(&mut iter_with_marker, error_context)
}

fn parse_optional_minimal_i64_value(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<Option<i64>, String> {
    let Some(marker) = payload_iter.next() else {
        return Ok(None);
    };

    parse_minimal_i64_value_from_marker(
        marker,
        payload_iter,
        "invalid or truncated minimal i64 payload",
    )
    .map(Some)
}

fn parse_response_reset_profile_paging_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<(u8, ResponseResetProfilePagingStatus, Option<i64>), String> {
    let request_id = next_payload_byte(payload_iter, "response reset profile paging request id")?;
    let status_raw = next_payload_byte(payload_iter, "response reset profile paging status")?;
    let status = ResponseResetProfilePagingStatus::try_from(status_raw)
        .map_err(|_| format!("unsupported reset profile paging status value {status_raw}"))?;

    if !matches!(status, ResponseResetProfilePagingStatus::Success) {
        if payload_iter.next().is_some() {
            return Err(
                "unexpected payload for non-success reset profile paging response".to_owned(),
            );
        }
        return Ok((request_id, status, None));
    }

    let iterator_session_id = parse_minimal_i64_value_with_context(
        payload_iter,
        "invalid or missing reset profile paging iterator session id payload",
    )?;

    Ok((request_id, status, Some(iterator_session_id)))
}

fn parse_response_next_profile_page_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<(u8, ResponseNextProfilePageStatus, Vec<ProfileLink>), String> {
    let request_id = next_payload_byte(payload_iter, "response next profile page request id")?;
    let status_raw = next_payload_byte(payload_iter, "response next profile page status")?;
    let status = ResponseNextProfilePageStatus::try_from(status_raw)
        .map_err(|_| format!("unsupported next profile page status value {status_raw}"))?;

    if !matches!(status, ResponseNextProfilePageStatus::Success) {
        if payload_iter.next().is_some() {
            return Err(
                "unexpected profile payload for non-success next profile page response".to_owned(),
            );
        }
        return Ok((request_id, status, Vec::new()));
    }

    let mut profiles = Vec::new();
    while let Some(account_id) = parse_optional_account_id_payload(payload_iter)? {
        let profile_version = parse_profile_version_payload(payload_iter)?;
        let profile_content_version = parse_profile_content_version_payload(payload_iter)?;
        let last_seen = parse_optional_last_seen_time_payload(payload_iter)?;

        profiles.push(ProfileLink::new(
            account_id,
            profile_version,
            profile_content_version,
            last_seen,
        ));
    }

    Ok((request_id, status, profiles))
}

fn parse_optional_account_id_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<Option<AccountId>, String> {
    let Some(first_byte) = payload_iter.next() else {
        return Ok(None);
    };

    let account_id =
        parse_uuid_base64_url_from_iter_with_first_byte(first_byte, payload_iter, "account id")?;
    Ok(Some(AccountId::new_base_64_url(account_id)))
}

fn parse_profile_version_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<ProfileVersion, String> {
    let profile_version = parse_uuid_base64_url_from_iter(payload_iter, "profile version")?;
    Ok(ProfileVersion::new_base_64_url(profile_version))
}

fn parse_profile_content_version_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<ProfileContentVersion, String> {
    let profile_content_version =
        parse_uuid_base64_url_from_iter(payload_iter, "profile content version")?;
    Ok(ProfileContentVersion::new_base_64_url(
        profile_content_version,
    ))
}

fn parse_optional_last_seen_time_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<Option<LastSeenTime>, String> {
    let marker = next_payload_byte(payload_iter, "next profile page last seen marker")?;

    if marker == 0 {
        return Ok(None);
    }

    let value = parse_minimal_i64_value_from_marker(
        marker,
        payload_iter,
        "invalid or truncated next profile page last seen payload",
    )?;

    Ok(Some(LastSeenTime::new(value)))
}

fn parse_content_processing_state_changed_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<ContentProcessingStateChanged, String> {
    let id = parse_minimal_i64_value(payload_iter)?;
    let state_raw = next_payload_byte(payload_iter, "content processing state")?;
    let state = ContentProcessingStateType::try_from(state_raw)
        .map_err(|_| format!("unsupported content processing state value {state_raw}"))?;

    let new_state = match state {
        ContentProcessingStateType::InQueue => ContentProcessingStateInternal::InQueue {
            wait_queue_position: parse_minimal_i64_value(payload_iter)?,
        },
        ContentProcessingStateType::Completed => {
            let content_id = parse_content_id_payload(payload_iter)?;
            let face_detected_byte = next_payload_byte(payload_iter, "face detection")?;
            ContentProcessingStateInternal::Completed {
                content_id,
                face_detected: face_detected_byte != 0,
            }
        }
        ContentProcessingStateType::Empty => ContentProcessingStateInternal::Empty,
        ContentProcessingStateType::Processing => ContentProcessingStateInternal::Processing,
        ContentProcessingStateType::Failed => ContentProcessingStateInternal::Failed,
        ContentProcessingStateType::NsfwDetected => ContentProcessingStateInternal::NsfwDetected,
    };

    Ok(ContentProcessingStateChanged { id, new_state })
}

fn parse_content_id_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<crate::ContentId, String> {
    let content_id = parse_uuid_base64_url_from_iter(payload_iter, "content id")?;
    Ok(crate::ContentId { cid: content_id })
}

fn parse_online_status_updated_payload(
    payload_iter: &mut impl Iterator<Item = u8>,
) -> Result<OnlineStatusUpdate, String> {
    let account_id = parse_account_id_payload(payload_iter)?;
    let marker = next_payload_byte(payload_iter, "check online status last seen marker")?;

    let last_seen = if marker == 0 {
        None
    } else {
        Some(LastSeenTime::new(parse_minimal_i64_value_from_marker(
            marker,
            payload_iter,
            "invalid or truncated check online status last seen payload",
        )?))
    };

    Ok(OnlineStatusUpdate {
        a: account_id,
        l: last_seen,
    })
}

#[cfg(test)]
mod tests {
    use simple_backend_model::ScheduledMaintenanceStatus;

    use super::parse_server_binary_message;
    use crate::{
        AccountId, ContentProcessingStateChanged, ContentProcessingStateInternal,
        EventToClientInternal, LastSeenTime, OnlineStatusUpdate, ProfileContentVersion,
        ProfileLink, ProfileVersion, ResponseNextProfilePageStatus,
        ResponseResetProfilePagingStatus, UnixTime,
        common::websocket::server::create_server_binary_message,
    };

    fn test_uuid(value: u8) -> simple_backend_utils::UuidBase64Url {
        simple_backend_utils::UuidBase64Url::from_bytes([value; 16])
    }

    fn test_account_id(value: u8) -> AccountId {
        AccountId::new_base_64_url(test_uuid(value))
    }

    fn test_profile_version(value: u8) -> ProfileVersion {
        ProfileVersion::new_base_64_url(test_uuid(value))
    }

    fn test_profile_content_version(value: u8) -> ProfileContentVersion {
        ProfileContentVersion::new_base_64_url(test_uuid(value))
    }

    macro_rules! assert_roundtrip_without_payload {
        ($name:ident, $event:expr, $pattern:pat) => {
            #[test]
            fn $name() {
                let message = create_server_binary_message(&$event);
                let parsed = parse_server_binary_message(&message)
                    .expect("parsing message without payload should succeed");

                assert!(matches!(parsed, $pattern));
            }
        };
    }

    assert_roundtrip_without_payload!(
        roundtrip_pending_app_notifications_changed_message,
        EventToClientInternal::PendingAppNotificationsChanged,
        EventToClientInternal::PendingAppNotificationsChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_client_config_changed_message,
        EventToClientInternal::ClientConfigChanged,
        EventToClientInternal::ClientConfigChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_news_changed_message,
        EventToClientInternal::NewsChanged,
        EventToClientInternal::NewsChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_push_notification_info_changed_message,
        EventToClientInternal::PushNotificationInfoChanged,
        EventToClientInternal::PushNotificationInfoChanged
    );

    #[test]
    fn roundtrip_websocket_connection_attempts_remaining_message() {
        let remaining = 50;
        let message = create_server_binary_message(
            &EventToClientInternal::WebSocketConnectionAttemptsRemaining { remaining },
        );
        let parsed = parse_server_binary_message(&message)
            .expect("websocket connection attempts remaining should parse");

        match parsed {
            EventToClientInternal::WebSocketConnectionAttemptsRemaining {
                remaining: parsed_remaining,
            } => {
                assert_eq!(parsed_remaining, remaining);
            }
            _ => panic!("unexpected event parsed"),
        }
    }
    assert_roundtrip_without_payload!(
        roundtrip_account_state_changed_message,
        EventToClientInternal::AccountStateChanged,
        EventToClientInternal::AccountStateChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_profile_changed_message,
        EventToClientInternal::ProfileChanged,
        EventToClientInternal::ProfileChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_media_content_changed_message,
        EventToClientInternal::MediaContentChanged,
        EventToClientInternal::MediaContentChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_new_message_received_message,
        EventToClientInternal::NewMessageReceived,
        EventToClientInternal::NewMessageReceived
    );
    assert_roundtrip_without_payload!(
        roundtrip_pending_chat_notifications_changed_message,
        EventToClientInternal::PendingChatNotificationsChanged,
        EventToClientInternal::PendingChatNotificationsChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_received_likes_changed_message,
        EventToClientInternal::ReceivedLikesChanged,
        EventToClientInternal::ReceivedLikesChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_daily_likes_left_changed_message,
        EventToClientInternal::DailyLikesLeftChanged,
        EventToClientInternal::DailyLikesLeftChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_message_delivery_info_changed_message,
        EventToClientInternal::MessageDeliveryInfoChanged,
        EventToClientInternal::MessageDeliveryInfoChanged
    );
    assert_roundtrip_without_payload!(
        roundtrip_latest_seen_message_changed_message,
        EventToClientInternal::LatestSeenMessageChanged,
        EventToClientInternal::LatestSeenMessageChanged
    );

    #[test]
    fn roundtrip_scheduled_maintenance_status_message() {
        let mut status = ScheduledMaintenanceStatus::default();
        status.set_admin_bot_offline(true);
        status.set_maintenance_time(Some(UnixTime::new(1234)), Some(UnixTime::new(5678)));

        let message = create_server_binary_message(
            &EventToClientInternal::ScheduledMaintenanceStatus(status.clone()),
        );
        let parsed =
            parse_server_binary_message(&message).expect("scheduled maintenance should parse");

        match parsed {
            EventToClientInternal::ScheduledMaintenanceStatus(parsed_status) => {
                assert_eq!(
                    parsed_status.admin_bot_offline(),
                    status.admin_bot_offline()
                );
                assert_eq!(parsed_status.start(), status.start());
                assert_eq!(parsed_status.end(), status.end());
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_admin_bot_notification_message() {
        let value = crate::AdminBotNotificationTypes::MODERATE_INITIAL_MEDIA_CONTENT_BOT
            | crate::AdminBotNotificationTypes::MODERATE_PROFILE_TEXTS_BOT;

        let message =
            create_server_binary_message(&EventToClientInternal::AdminBotNotification(value));
        let parsed =
            parse_server_binary_message(&message).expect("admin bot notification should parse");

        match parsed {
            EventToClientInternal::AdminBotNotification(parsed_value) => {
                assert_eq!(parsed_value.bits(), value.bits());
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_response_next_profile_page_message() {
        let profiles = vec![
            ProfileLink::new(
                test_account_id(1),
                test_profile_version(2),
                test_profile_content_version(3),
                Some(LastSeenTime::new(-1)),
            ),
            ProfileLink::new(
                test_account_id(4),
                test_profile_version(5),
                test_profile_content_version(6),
                None,
            ),
        ];
        let request_id = 7;
        let status = ResponseNextProfilePageStatus::Success;

        let message =
            create_server_binary_message(&EventToClientInternal::ResponseNextProfilePage {
                request_id,
                status,
                profiles: profiles.clone(),
            });
        let parsed = parse_server_binary_message(&message).expect("next profile page should parse");

        match parsed {
            EventToClientInternal::ResponseNextProfilePage {
                request_id: parsed_request_id,
                status: parsed_status,
                profiles: parsed_profiles,
            } => {
                assert_eq!(parsed_request_id, request_id);
                assert_eq!(parsed_status, status);
                assert_eq!(parsed_profiles, profiles);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_response_reset_profile_paging_message() {
        let request_id = 9;
        let status = ResponseResetProfilePagingStatus::Success;
        let iterator_session_id = Some(42);

        let message =
            create_server_binary_message(&EventToClientInternal::ResponseResetProfilePaging {
                request_id,
                status,
                iterator_session_id,
            });
        let parsed =
            parse_server_binary_message(&message).expect("reset profile paging should parse");

        match parsed {
            EventToClientInternal::ResponseResetProfilePaging {
                request_id: parsed_request_id,
                status: parsed_status,
                iterator_session_id: parsed_iterator_session_id,
            } => {
                assert_eq!(parsed_request_id, request_id);
                assert_eq!(parsed_status, status);
                assert_eq!(parsed_iterator_session_id, iterator_session_id);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_response_reset_profile_paging_rate_limited_message() {
        let request_id = 10;
        let status = ResponseResetProfilePagingStatus::RateLimited;
        let iterator_session_id = None;

        let message =
            create_server_binary_message(&EventToClientInternal::ResponseResetProfilePaging {
                request_id,
                status,
                iterator_session_id,
            });
        let parsed = parse_server_binary_message(&message)
            .expect("rate limited reset profile paging should parse");

        match parsed {
            EventToClientInternal::ResponseResetProfilePaging {
                request_id: parsed_request_id,
                status: parsed_status,
                iterator_session_id: parsed_iterator_session_id,
            } => {
                assert_eq!(parsed_request_id, request_id);
                assert_eq!(parsed_status, status);
                assert_eq!(parsed_iterator_session_id, iterator_session_id);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_response_automatic_profile_search_reset_profile_paging_message() {
        let request_id = 11;
        let status = ResponseResetProfilePagingStatus::Success;
        let iterator_session_id = Some(55);

        let message = create_server_binary_message(
            &EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
                request_id,
                status,
                iterator_session_id,
            },
        );
        let parsed = parse_server_binary_message(&message)
            .expect("automatic profile search reset profile paging should parse");

        match parsed {
            EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
                request_id: parsed_request_id,
                status: parsed_status,
                iterator_session_id: parsed_iterator_session_id,
            } => {
                assert_eq!(parsed_request_id, request_id);
                assert_eq!(parsed_status, status);
                assert_eq!(parsed_iterator_session_id, iterator_session_id);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_response_automatic_profile_search_next_profile_page_message() {
        let profiles = vec![
            ProfileLink::new(
                test_account_id(14),
                test_profile_version(15),
                test_profile_content_version(16),
                Some(LastSeenTime::new(123)),
            ),
            ProfileLink::new(
                test_account_id(17),
                test_profile_version(18),
                test_profile_content_version(19),
                None,
            ),
        ];
        let request_id = 12;
        let status = ResponseNextProfilePageStatus::Success;

        let message = create_server_binary_message(
            &EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
                request_id,
                status,
                profiles: profiles.clone(),
            },
        );
        let parsed = parse_server_binary_message(&message)
            .expect("automatic profile search next profile page should parse");

        match parsed {
            EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
                request_id: parsed_request_id,
                status: parsed_status,
                profiles: parsed_profiles,
            } => {
                assert_eq!(parsed_request_id, request_id);
                assert_eq!(parsed_status, status);
                assert_eq!(parsed_profiles, profiles);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_content_processing_state_changed_message() {
        let expected = ContentProcessingStateChanged {
            id: 42,
            new_state: ContentProcessingStateInternal::Completed {
                content_id: crate::ContentId { cid: test_uuid(9) },
                face_detected: true,
            },
        };

        let message = create_server_binary_message(
            &EventToClientInternal::ContentProcessingStateChanged(expected.clone()),
        );
        let parsed = parse_server_binary_message(&message)
            .expect("content processing state changed should parse");

        match parsed {
            EventToClientInternal::ContentProcessingStateChanged(parsed_value) => {
                assert_eq!(parsed_value.id, expected.id);
                assert_eq!(parsed_value.new_state, expected.new_state);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_typing_start_message() {
        let account_id = test_account_id(10);

        let message = create_server_binary_message(&EventToClientInternal::TypingStart(account_id));
        let parsed = parse_server_binary_message(&message).expect("typing start should parse");

        match parsed {
            EventToClientInternal::TypingStart(parsed_account_id) => {
                assert_eq!(parsed_account_id, account_id);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_typing_stop_message() {
        let account_id = test_account_id(11);

        let message = create_server_binary_message(&EventToClientInternal::TypingStop(account_id));
        let parsed = parse_server_binary_message(&message).expect("typing stop should parse");

        match parsed {
            EventToClientInternal::TypingStop(parsed_account_id) => {
                assert_eq!(parsed_account_id, account_id);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_online_status_updated_message() {
        let expected = OnlineStatusUpdate {
            a: test_account_id(12),
            l: Some(LastSeenTime::new(123_456_789)),
        };

        let message = create_server_binary_message(&EventToClientInternal::OnlineStatusUpdated(
            expected.clone(),
        ));
        let parsed = parse_server_binary_message(&message)
            .expect("online status updated event should parse");

        match parsed {
            EventToClientInternal::OnlineStatusUpdated(parsed_value) => {
                assert_eq!(parsed_value.a, expected.a);
                assert_eq!(parsed_value.l, expected.l);
            }
            _ => panic!("unexpected event parsed"),
        }
    }

    #[test]
    fn roundtrip_online_status_updated_without_last_seen_message() {
        let expected = OnlineStatusUpdate {
            a: test_account_id(13),
            l: None,
        };

        let message = create_server_binary_message(&EventToClientInternal::OnlineStatusUpdated(
            expected.clone(),
        ));
        let parsed = parse_server_binary_message(&message)
            .expect("online status updated event without last seen should parse");

        match parsed {
            EventToClientInternal::OnlineStatusUpdated(parsed_value) => {
                assert_eq!(parsed_value.a, expected.a);
                assert_eq!(parsed_value.l, expected.l);
            }
            _ => panic!("unexpected event parsed"),
        }
    }
}
