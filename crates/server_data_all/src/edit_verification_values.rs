use model::{AccountIdInternal, EditVerificationValues};
use server_data::{DataError, result::WrappedContextExt, write_commands::WriteCommandRunnerHandle};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use server_data_profile::write::GetWriteCommandsProfile;

pub async fn edit_verification_values(
    write_command_runner: &WriteCommandRunnerHandle,
    moderator_id: AccountIdInternal,
    values: EditVerificationValues,
) -> server_common::result::Result<(), DataError> {
    let EditVerificationValues {
        profile_owner_id,
        security_content,
        profile_age_range,
        profile_name,
    } = values;

    write_command_runner
        .write(move |cmds| async move {
            let mut send_profile_changed_event = false;

            if let Some(profile_age_range) = profile_age_range {
                let changed = cmds
                    .profile_admin()
                    .verification()
                    .change_profile_age_range_verified_value(
                        moderator_id,
                        profile_owner_id,
                        profile_age_range.verified_value,
                    )
                    .await?;
                send_profile_changed_event |= changed;
            }

            if let Some(profile_name) = profile_name {
                let changed = cmds
                    .profile_admin()
                    .verification()
                    .change_profile_name_verified_value(
                        moderator_id,
                        profile_owner_id,
                        profile_name.verified_value,
                    )
                    .await?;
                send_profile_changed_event |= changed;
            }

            if send_profile_changed_event {
                cmds.profile_admin()
                    .verification()
                    .send_profile_changed_event(profile_owner_id)
                    .await?;
            }

            if let Some(security_content) = security_content {
                let verified_value = security_content.verified_value;

                let current_security_content = cmds
                    .read()
                    .media()
                    .current_account_media(profile_owner_id)
                    .await?
                    .security_content_id
                    .map(|v| v.content_id());

                if current_security_content != Some(security_content.security_content) {
                    return Err(DataError::NotAllowed.report());
                }

                cmds.media_admin()
                    .content()
                    .change_security_content_verified_value(
                        moderator_id,
                        profile_owner_id,
                        verified_value,
                    )
                    .await?;
            }

            Ok(())
        })
        .await
}
