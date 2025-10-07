use model::{AccountIdInternal, EmailMessages};
use server_api::{
    DataError,
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::result::Result;
use server_data::read::GetReadCommandsCommon;
use server_data_account::write::GetWriteCommandsAccount;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_state::S;

pub async fn handle_email_notifications(state: &S, id: AccountIdInternal) -> Result<(), DataError> {
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
