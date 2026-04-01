use utils::minimal_i64;

use crate::{
    AccountId, ContentProcessingStateChanged, ContentProcessingStateInternal,
    ContentProcessingStateType, EventToClientInternal, LastSeenTime, ProfileContentVersion,
    ProfileLink, ProfileVersion, ResponseCheckOnlineStatus, ResponseNextProfilePageStatus,
    ScheduledMaintenanceStatus, ServerMessageType, UnixTime,
};

pub fn parse_server_binary_message(message: &[u8]) -> Result<EventToClientInternal, String> {
    let (message_type_u8, payload) = message
        .split_first()
        .ok_or_else(|| "missing server message type byte".to_owned())?;

    let message_type = ServerMessageType::try_from(*message_type_u8)
        .map_err(|_| format!("unsupported server message type {message_type_u8}"))?;

    let event = match message_type {
        ServerMessageType::PendingAppNotificationsChanged => {
            EventToClientInternal::PendingAppNotificationsChanged
        }
        ServerMessageType::ClientConfigChanged => EventToClientInternal::ClientConfigChanged,
        ServerMessageType::NewsCountChanged => EventToClientInternal::NewsChanged,
        ServerMessageType::ScheduledMaintenanceStatus => {
            EventToClientInternal::ScheduledMaintenanceStatus(
                parse_scheduled_maintenance_status_payload(payload)?,
            )
        }
        ServerMessageType::AdminBotNotification => {
            let bits = payload
                .first()
                .copied()
                .ok_or_else(|| "missing admin bot notification payload".to_owned())?;
            EventToClientInternal::AdminBotNotification(
                crate::AdminBotNotificationTypes::from_bits_truncate(bits),
            )
        }
        ServerMessageType::PushNotificationInfoChanged => {
            EventToClientInternal::PushNotificationInfoChanged
        }
        ServerMessageType::AccountStateChanged => EventToClientInternal::AccountStateChanged,
        ServerMessageType::ProfileChanged => EventToClientInternal::ProfileChanged,
        ServerMessageType::ResponseNextProfilePage => {
            let (status, profiles) = parse_response_next_profile_page_payload(payload)?;
            EventToClientInternal::ResponseNextProfilePage { status, profiles }
        }
        ServerMessageType::ContentProcessingStateChanged => {
            EventToClientInternal::ContentProcessingStateChanged(
                parse_content_processing_state_changed_payload(payload)?,
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
            EventToClientInternal::TypingStart(parse_account_id_payload(payload)?)
        }
        ServerMessageType::TypingStop => {
            EventToClientInternal::TypingStop(parse_account_id_payload(payload)?)
        }
        ServerMessageType::ResponseCheckOnlineStatus => {
            EventToClientInternal::ResponseCheckOnlineStatus(
                parse_check_online_status_response_payload(payload)?,
            )
        }
        ServerMessageType::MessageDeliveryInfoChanged => {
            EventToClientInternal::MessageDeliveryInfoChanged
        }
        ServerMessageType::LatestSeenMessageChanged => {
            EventToClientInternal::LatestSeenMessageChanged
        }
    };

    Ok(event)
}

fn parse_account_id_payload(payload: &[u8]) -> Result<AccountId, String> {
    let bytes: [u8; 16] = payload
        .try_into()
        .map_err(|_| "invalid account id payload size".to_owned())?;

    Ok(AccountId::new_base_64_url(
        simple_backend_utils::UuidBase64Url::from_bytes(bytes),
    ))
}

fn parse_scheduled_maintenance_status_payload(
    payload: &[u8],
) -> Result<ScheduledMaintenanceStatus, String> {
    let (admin_bot_offline_raw, remaining_payload) = payload
        .split_first()
        .ok_or_else(|| "missing scheduled maintenance admin_bot_offline payload".to_owned())?;

    let mut payload_iter = remaining_payload.iter().copied();
    let start = if remaining_payload.is_empty() {
        None
    } else {
        Some(parse_minimal_i64_value(&mut payload_iter)?)
    };

    let end = if payload_iter.clone().next().is_some() {
        Some(parse_minimal_i64_value(&mut payload_iter)?)
    } else {
        None
    };

    let mut status = ScheduledMaintenanceStatus::default();
    status.set_admin_bot_offline(*admin_bot_offline_raw != 0);
    status.set_maintenance_time(start.map(UnixTime::new), end.map(UnixTime::new));

    Ok(status)
}

fn parse_minimal_i64_value(payload_iter: &mut impl Iterator<Item = u8>) -> Result<i64, String> {
    minimal_i64::parse_minimal_i64_from_iter(payload_iter)
        .ok_or_else(|| "invalid or truncated minimal i64 payload".to_owned())
}

fn parse_response_next_profile_page_payload(
    payload: &[u8],
) -> Result<(ResponseNextProfilePageStatus, Vec<ProfileLink>), String> {
    let (status_raw, mut tail) = payload
        .split_first()
        .ok_or_else(|| "missing response next profile page status payload".to_owned())?;
    let status = ResponseNextProfilePageStatus::try_from(*status_raw)
        .map_err(|_| format!("unsupported next profile page status value {status_raw}"))?;

    if !matches!(status, ResponseNextProfilePageStatus::Success) {
        if !tail.is_empty() {
            return Err(
                "unexpected profile payload for non-success next profile page response".to_owned(),
            );
        }
        return Ok((status, Vec::new()));
    }

    let mut profiles = Vec::new();
    while !tail.is_empty() {
        if tail.len() < 48 {
            return Err("truncated response next profile page profile payload".to_owned());
        }

        let account_id = parse_account_id_payload(&tail[..16])?;
        let profile_version = parse_profile_version_payload(&tail[16..32])?;
        let profile_content_version = parse_profile_content_version_payload(&tail[32..48])?;
        tail = &tail[48..];

        let (last_seen, consumed) = parse_optional_last_seen_time_payload(tail)?;
        tail = &tail[consumed..];

        profiles.push(ProfileLink::new(
            account_id,
            profile_version,
            profile_content_version,
            last_seen,
        ));
    }

    Ok((status, profiles))
}

fn parse_profile_version_payload(payload: &[u8]) -> Result<ProfileVersion, String> {
    let bytes: [u8; 16] = payload
        .try_into()
        .map_err(|_| "invalid profile version payload size".to_owned())?;
    Ok(ProfileVersion::new_base_64_url(
        simple_backend_utils::UuidBase64Url::from_bytes(bytes),
    ))
}

fn parse_profile_content_version_payload(payload: &[u8]) -> Result<ProfileContentVersion, String> {
    let bytes: [u8; 16] = payload
        .try_into()
        .map_err(|_| "invalid profile content version payload size".to_owned())?;
    Ok(ProfileContentVersion::new_base_64_url(
        simple_backend_utils::UuidBase64Url::from_bytes(bytes),
    ))
}

fn parse_optional_last_seen_time_payload(
    payload: &[u8],
) -> Result<(Option<LastSeenTime>, usize), String> {
    let marker = *payload
        .first()
        .ok_or_else(|| "missing next profile page last seen marker".to_owned())?;

    if marker == 0 {
        return Ok((None, 1));
    }

    let byte_len = match marker {
        1 => 1,
        2 => 2,
        4 => 4,
        8 => 8,
        _ => {
            return Err(format!(
                "unsupported next profile page last seen marker {marker}"
            ));
        }
    };

    if payload.len() < 1 + byte_len {
        return Err("truncated next profile page last seen payload".to_owned());
    }

    let value_payload = &payload[1..1 + byte_len];
    let value = match marker {
        1 => i8::from_le_bytes([value_payload[0]]) as i64,
        2 => i16::from_le_bytes([value_payload[0], value_payload[1]]) as i64,
        4 => i32::from_le_bytes([
            value_payload[0],
            value_payload[1],
            value_payload[2],
            value_payload[3],
        ]) as i64,
        8 => i64::from_le_bytes([
            value_payload[0],
            value_payload[1],
            value_payload[2],
            value_payload[3],
            value_payload[4],
            value_payload[5],
            value_payload[6],
            value_payload[7],
        ]),
        _ => unreachable!(),
    };

    Ok((Some(LastSeenTime::new(value)), 1 + byte_len))
}

fn parse_content_processing_state_changed_payload(
    payload: &[u8],
) -> Result<ContentProcessingStateChanged, String> {
    let mut payload_iter = payload.iter().copied();

    let id = parse_minimal_i64_value(&mut payload_iter)?;
    let state_raw = payload_iter
        .next()
        .ok_or_else(|| "missing content processing state payload".to_owned())?;
    let state = ContentProcessingStateType::try_from(state_raw)
        .map_err(|_| format!("unsupported content processing state value {state_raw}"))?;

    let new_state = match state {
        ContentProcessingStateType::InQueue => ContentProcessingStateInternal::InQueue {
            wait_queue_position: parse_minimal_i64_value(&mut payload_iter)?,
        },
        ContentProcessingStateType::Completed => {
            let mut cid_bytes = [0u8; 16];
            for byte in cid_bytes.iter_mut() {
                *byte = payload_iter
                    .next()
                    .ok_or_else(|| "missing content id payload".to_owned())?;
            }
            let fd_byte = payload_iter
                .next()
                .ok_or_else(|| "missing face detection payload".to_owned())?;
            ContentProcessingStateInternal::Completed {
                content_id: crate::ContentId {
                    cid: simple_backend_utils::UuidBase64Url::from_bytes(cid_bytes),
                },
                fd: fd_byte != 0,
            }
        }
        ContentProcessingStateType::Empty => ContentProcessingStateInternal::Empty,
        ContentProcessingStateType::Processing => ContentProcessingStateInternal::Processing,
        ContentProcessingStateType::Failed => ContentProcessingStateInternal::Failed,
        ContentProcessingStateType::NsfwDetected => ContentProcessingStateInternal::NsfwDetected,
    };

    Ok(ContentProcessingStateChanged { id, new_state })
}

fn parse_check_online_status_response_payload(
    payload: &[u8],
) -> Result<ResponseCheckOnlineStatus, String> {
    if payload.len() != 17 && payload.len() != 25 {
        return Err("invalid check online status payload size".to_owned());
    }

    let (account_id_payload, has_last_seen_and_tail) = payload.split_at(16);
    let account_id = parse_account_id_payload(account_id_payload)?;
    let has_last_seen = has_last_seen_and_tail.first().copied().unwrap_or_default() != 0;

    let last_seen = if has_last_seen {
        let last_seen_tail = has_last_seen_and_tail
            .get(1..)
            .ok_or_else(|| "missing last seen payload".to_owned())?;
        let raw_bytes: [u8; 8] = last_seen_tail
            .try_into()
            .map_err(|_| "invalid last seen payload size".to_owned())?;
        Some(LastSeenTime::new(i64::from_be_bytes(raw_bytes)))
    } else {
        None
    };

    Ok(ResponseCheckOnlineStatus {
        a: account_id,
        l: last_seen,
    })
}
