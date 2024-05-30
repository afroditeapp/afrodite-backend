use model::{AccountId, AccountIdInternal, EmailAddress, SignInWithInfo};
use server_api::{
    app::{StateBase, WriteData},
    db_write_raw,
    utils::StatusCode,
};
use server_data::write::WriteCommandsProvider;
use server_data_all::register::RegisterAccount;

pub async fn register_impl<S: StateBase + WriteData>(
    state: &S,
    sign_in_with: SignInWithInfo,
    email: Option<EmailAddress>,
) -> Result<AccountIdInternal, StatusCode> {
    // New unique UUID is generated every time so no special handling needed
    // to avoid database collisions.
    let id = AccountId::new(uuid::Uuid::new_v4());

    let id = db_write_raw!(state, move |cmds| {
        RegisterAccount::new(cmds.write_cmds())
            .register(id, sign_in_with, email)
            .await
    })
    .await?;

    Ok(id)
}
