use api_client::apis::account_admin_api::post_update_news_translation;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result =
        post_update_news_translation(&no_token_api, 999_999, "en", sample_translation()).await;
    assert_failure(result)
}

fn sample_translation() -> api_client::models::UpdateNewsTranslation {
    api_client::models::UpdateNewsTranslation::new(
        "body".to_string(),
        api_client::models::NewsTranslationVersion::default(),
        "title".to_string(),
    )
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result =
        post_update_news_translation(&account.api(), 999_999, "en", sample_translation()).await;
    assert_failure(result)
}

/// Translation of a non existing news item cannot be updated.
#[server_test]
async fn updating_translation_of_non_existing_news_item_fails(
    mut context: TestContext,
) -> TestResult {
    let admin = context.new_admin().await?;

    let result =
        post_update_news_translation(&admin.api(), 999_999, "en", sample_translation()).await;
    assert_failure(result)?;

    Ok(())
}
