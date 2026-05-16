use model::{AccountIdInternal, AccountVerificationErrorFlags, EditVerificationValues};
use server_data::{DataError, result::WrappedContextExt, write_commands::WriteCommandRunnerHandle};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};

pub async fn edit_verification_values(
    write_command_runner: &WriteCommandRunnerHandle,
    moderator_id: AccountIdInternal,
    profile_owner_id: AccountIdInternal,
    values: EditVerificationValues,
) -> server_common::result::Result<(), DataError> {
    let flags = write_command_runner
        .write(move |cmds| async move {
            edit_verification_values_in_write_call(
                &cmds,
                moderator_id,
                profile_owner_id,
                values,
                false,
            )
            .await
        })
        .await?;

    if !flags.is_empty() {
        return Err(DataError::NotAllowed.report());
    }

    Ok(())
}

pub async fn edit_verification_values_in_write_call(
    cmds: &server_data::write_commands::WriteCmds,
    moderator_id: AccountIdInternal,
    profile_owner_id: AccountIdInternal,
    values: EditVerificationValues,
    reset_missing_values_to_null: bool,
) -> server_common::result::Result<AccountVerificationErrorFlags, DataError> {
    let EditVerificationValues {
        security_content,
        profile_age_range,
        profile_name,
    } = values;

    let mut flags = AccountVerificationErrorFlags::empty();
    let mut send_profile_changed_event = false;

    if let Some(profile_age_range) = profile_age_range {
        let current_profile_age_in_db = cmds
            .read()
            .profile()
            .profile(profile_owner_id)
            .await?
            .profile
            .age;

        if profile_age_range.current_profile_age != current_profile_age_in_db {
            flags |= AccountVerificationErrorFlags::PROFILE_AGE_RANGE_MISMATCH;
        } else {
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
    } else if reset_missing_values_to_null {
        let changed = cmds
            .profile_admin()
            .verification()
            .change_profile_age_range_verified_value(moderator_id, profile_owner_id, None)
            .await?;
        send_profile_changed_event |= changed;
    }

    if let Some(profile_name) = profile_name {
        let current_profile_name_in_db = cmds
            .read()
            .profile()
            .profile(profile_owner_id)
            .await?
            .profile
            .name;

        if profile_name.current_profile_name != current_profile_name_in_db {
            flags |= AccountVerificationErrorFlags::PROFILE_NAME_MISMATCH;
        } else {
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
    } else if reset_missing_values_to_null {
        let changed = cmds
            .profile_admin()
            .verification()
            .change_profile_name_verified_value(moderator_id, profile_owner_id, None)
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
            flags |= AccountVerificationErrorFlags::SECURITY_CONTENT_MISMATCH;
        } else {
            cmds.media_admin()
                .content()
                .change_security_content_verified_value(
                    moderator_id,
                    profile_owner_id,
                    verified_value,
                )
                .await?;
        }
    } else if reset_missing_values_to_null {
        cmds.media_admin()
            .content()
            .change_security_content_verified_value(moderator_id, profile_owner_id, None)
            .await?;
    }

    Ok(flags)
}
