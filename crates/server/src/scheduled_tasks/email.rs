use model::{AccountIdInternal, EmailMessages, UnixTime};
use server_api::{
    DataError,
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::result::Result;
use server_data::read::GetReadCommandsCommon;
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_state::S;

pub async fn handle_email_notifications(state: &S, id: AccountIdInternal) -> Result<(), DataError> {
    let email_settings = state
        .read()
        .chat()
        .notification()
        .chat_email_notification_settings(id)
        .await?;

    if email_settings.messages {
        handle_messages_email_notification(state, id).await?;
    }

    if email_settings.likes {
        handle_likes_email_notification(state, id).await?;
    }

    handle_account_deletion_email_notification(state, id).await?;

    handle_email_change(state, id).await?;

    remove_expired_email_verification_token(state, id).await?;

    cancel_email_change_if_needed(state, id).await?;

    remove_expired_email_login_token(state, id).await?;

    Ok(())
}

async fn handle_messages_email_notification(
    state: &S,
    id: AccountIdInternal,
) -> Result<(), DataError> {
    let push_notification_device_token_exists = state
        .read()
        .common()
        .push_notification()
        .push_notification_device_token_exists(id)
        .await?;
    let wait_time = if push_notification_device_token_exists {
        state
            .config()
            .limits_chat()
            .new_message_email_with_push_notification_device_token
    } else {
        state
            .config()
            .limits_chat()
            .new_message_email_without_push_notification_device_token
    };

    let messages = state
        .read()
        .chat()
        .notification()
        .messages_without_sent_email_notification(id)
        .await?;
    let mut send_notification = false;

    for m in &messages {
        if m.time.duration_value_elapsed(wait_time) {
            send_notification = true;
        }
    }

    if send_notification {
        let messages = messages.iter().map(|v| v.id.clone()).collect();
        db_write_raw!(state, move |cmds| {
            cmds.chat()
                .notification()
                .mark_message_email_notification_sent(messages)
                .await?;
            cmds.account()
                .email()
                .send_email_if_sending_is_not_in_progress(id, EmailMessages::NewMessage)
                .await?;
            Ok(())
        })
        .await?;
    }

    Ok(())
}

async fn handle_likes_email_notification(
    state: &S,
    id: AccountIdInternal,
) -> Result<(), DataError> {
    let push_notification_device_token_exists = state
        .read()
        .common()
        .push_notification()
        .push_notification_device_token_exists(id)
        .await?;

    let wait_time = if push_notification_device_token_exists {
        state
            .config()
            .limits_chat()
            .new_like_email_with_push_notification_device_token
    } else {
        state
            .config()
            .limits_chat()
            .new_like_email_without_push_notification_device_token
    };

    let likes_with_time = state
        .read()
        .chat()
        .notification()
        .unviewed_received_likes_without_sent_email_notification(id)
        .await?;

    let mut send_notification = false;
    for (_, received_time) in &likes_with_time {
        if received_time.duration_value_elapsed(wait_time) {
            send_notification = true;
        }
    }

    if send_notification {
        db_write_raw!(state, move |cmds| {
            cmds.chat()
                .notification()
                .mark_like_email_notification_sent(
                    id,
                    likes_with_time.into_iter().map(|v| v.0).collect(),
                )
                .await?;
            cmds.account()
                .email()
                .send_email_if_sending_is_not_in_progress(id, EmailMessages::NewLike)
                .await?;
            Ok(())
        })
        .await?;
    }

    Ok(())
}

async fn handle_account_deletion_email_notification(
    state: &S,
    id: AccountIdInternal,
) -> Result<(), DataError> {
    let deletion_state = state
        .read()
        .account()
        .delete()
        .account_deletion_state(id)
        .await?;

    let deletion_time = match deletion_state.automatic_deletion_allowed {
        Some(deletion_time) => deletion_time,
        None => return Ok(()),
    };

    let current_time = UnixTime::current_time();
    if current_time >= deletion_time {
        return Ok(());
    }

    let time_until_deletion = deletion_time.ut - current_time.ut;

    const SECONDS_PER_DAY: i64 = 60 * 60 * 24;
    const SECOND_EMAIL_DAYS_REMAINING: i64 = 30;
    const THIRD_EMAIL_DAYS_REMAINING: i64 = 7;

    let second_email_threshold = SECOND_EMAIL_DAYS_REMAINING * SECONDS_PER_DAY;
    let third_email_threshold = THIRD_EMAIL_DAYS_REMAINING * SECONDS_PER_DAY;

    // Determine which email to send based on time remaining
    // We only send the most recent applicable email to avoid sending outdated ones
    let email_to_send = if time_until_deletion <= third_email_threshold {
        // 7 days or less remaining - send third email
        EmailMessages::AccountDeletionRemainderThird
    } else if time_until_deletion <= second_email_threshold {
        // Between 7 and 30 days remaining - send second email
        EmailMessages::AccountDeletionRemainderSecond
    } else {
        EmailMessages::AccountDeletionRemainderFirst
    };

    db_write_raw!(state, move |cmds| {
        cmds.account()
            .email()
            .send_email_if_not_already_sent(id, email_to_send)
            .await?;
        Ok(())
    })
    .await?;

    Ok(())
}

async fn handle_email_change(state: &S, id: AccountIdInternal) -> Result<(), DataError> {
    let internal = state
        .read()
        .account()
        .email_address_state_internal(id)
        .await?;

    let change_time = match internal.email_change_unix_time {
        Some(time) => time,
        None => return Ok(()),
    };

    let new_email = match internal.email_change {
        Some(email) => email,
        None => return Ok(()),
    };

    if !internal.email_change_verified {
        return Ok(());
    }

    let min_wait_duration = state
        .config()
        .limits_account()
        .email_change_min_wait_duration;

    if !change_time.duration_value_elapsed(min_wait_duration) {
        return Ok(());
    }

    db_write_raw!(state, move |cmds| {
        cmds.account()
            .email()
            .complete_email_change(id, new_email)
            .await?;
        Ok(())
    })
    .await?;

    Ok(())
}

async fn remove_expired_email_verification_token(
    state: &S,
    id: AccountIdInternal,
) -> Result<(), DataError> {
    let internal = state
        .read()
        .account()
        .email_address_state_internal(id)
        .await?;

    let email_verification_token_validity = state
        .config()
        .limits_account()
        .email_verification_token_validity_duration;

    if let Some(token_time) = internal.email_verification_token_unix_time {
        if token_time.duration_value_elapsed(email_verification_token_validity) {
            db_write_raw!(state, move |cmds| {
                cmds.account()
                    .email()
                    .clear_email_verification_token(id)
                    .await?;
                Ok(())
            })
            .await?;
        }
    }

    Ok(())
}

async fn cancel_email_change_if_needed(state: &S, id: AccountIdInternal) -> Result<(), DataError> {
    let internal = state
        .read()
        .account()
        .email_address_state_internal(id)
        .await?;

    let email_change_token_validity = state
        .config()
        .limits_account()
        .email_change_min_wait_duration;

    if let Some(change_time) = internal.email_change_unix_time {
        if change_time.duration_value_elapsed(email_change_token_validity) {
            db_write_raw!(state, move |cmds| {
                cmds.account().email().cancel_email_change(id).await?;
                Ok(())
            })
            .await?;
        }
    }

    Ok(())
}

async fn remove_expired_email_login_token(
    state: &S,
    id: AccountIdInternal,
) -> Result<(), DataError> {
    let internal = state
        .read()
        .account()
        .email_address_state_internal(id)
        .await?;

    let email_login_token_validity = state
        .config()
        .limits_account()
        .email_login_token_validity_duration;

    if let Some(token_time) = internal.email_login_token_unix_time {
        if token_time.duration_value_elapsed(email_login_token_validity) {
            db_write_raw!(state, move |cmds| {
                cmds.account().email().clear_email_login_token(id).await?;
                Ok(())
            })
            .await?;
        }
    }

    Ok(())
}
