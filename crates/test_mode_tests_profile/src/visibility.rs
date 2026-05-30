use api_client::{
    apis::{
        account_api::get_account_state,
        common_api::{post_get_next_profile_page, post_reset_profile_paging},
        profile_api::{post_search_age_range, post_search_groups},
    },
    models::{ProfileVisibility, SearchAgeRange, SearchGroups},
};
use test_mode_bot::actions::account::SetProfileVisibility;
use test_mode_tests::prelude::*;

#[server_test]
async fn visiblity_updates_in_initial_setup_state(mut context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
    assert_eq(
        ProfileVisibility::Private,
        get_account_state(&account.account_api()).await?.visibility,
    )?;
    account.run(SetProfileVisibility(true)).await?;
    assert_eq(
        ProfileVisibility::Public,
        get_account_state(&account.account_api()).await?.visibility,
    )?;
    account.run(SetProfileVisibility(false)).await?;
    assert_eq(
        ProfileVisibility::Private,
        get_account_state(&account.account_api()).await?.visibility,
    )
}

#[server_test]
async fn visiblity_updates_in_normal_state(mut context: TestContext) -> TestResult {
    let mut account = context.new_account().await?;
    assert_eq(
        ProfileVisibility::Private,
        get_account_state(&account.account_api()).await?.visibility,
    )?;
    account.run(SetProfileVisibility(true)).await?;
    assert_eq(
        ProfileVisibility::Public,
        get_account_state(&account.account_api()).await?.visibility,
    )?;
    account.run(SetProfileVisibility(false)).await?;
    assert_eq(
        ProfileVisibility::Private,
        get_account_state(&account.account_api()).await?.visibility,
    )
}

#[server_test]
async fn initial_setup_visibility_change_does_not_change_available_profiles(
    mut context: TestContext,
) -> TestResult {
    let mut account1 = context.new_account_in_initial_setup_state().await?;
    post_search_groups(
        &account1.profile_api(),
        SearchGroups {
            man_for_man: Some(true),
            ..SearchGroups::default()
        },
    )
    .await?;
    post_search_age_range(&account1.profile_api(), SearchAgeRange { min: 18, max: 99 }).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        0,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )?;
    account1.run(SetProfileVisibility(true)).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        0,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )
}

#[server_test]
async fn your_own_profile_can_be_returned_if_filters_allow(mut context: TestContext) -> TestResult {
    let mut account1 = context.new_account().await?;
    post_search_groups(
        &account1.profile_api(),
        SearchGroups {
            man_for_man: Some(true),
            ..SearchGroups::default()
        },
    )
    .await?;
    post_search_age_range(&account1.profile_api(), SearchAgeRange { min: 18, max: 99 }).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        0,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )?;
    account1.run(SetProfileVisibility(true)).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        1,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )
}

#[server_test]
async fn updates_changes_available_profiles(mut context: TestContext) -> TestResult {
    let mut account1 = context.new_man_18_years().await?;
    let mut account2 = context.new_woman_18_years().await?;
    context.new_admin_and_moderate_initial_content().await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        0,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )?;
    account1.run(SetProfileVisibility(true)).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        0,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )?;
    account2.run(SetProfileVisibility(true)).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        1,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )?;
    account1.run(SetProfileVisibility(false)).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        1,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )?;
    account2.run(SetProfileVisibility(false)).await?;
    let iterator_id = post_reset_profile_paging(&account1.profile_api()).await?;
    assert_eq(
        0,
        post_get_next_profile_page(&account1.profile_api(), iterator_id)
            .await?
            .profiles
            .len(),
    )
}
