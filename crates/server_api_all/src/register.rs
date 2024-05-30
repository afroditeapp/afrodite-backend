pub async fn register_impl<S: WriteData>(
    state: &S,
    sign_in_with: SignInWithInfo,
    email: Option<EmailAddress>,
) -> Result<AccountIdInternal, StatusCode> {
    // New unique UUID is generated every time so no special handling needed
    // to avoid database collisions.
    let id = AccountId::new(uuid::Uuid::new_v4());

    let id = state
        .write(move |cmds| async move { cmds.register(id, sign_in_with, email).await })
        .await?;

    Ok(id)
}
