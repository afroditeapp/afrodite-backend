use api_client::{
    apis::{
        account_api::get_account_state,
        common_admin_api::{post_get_waiting_reports_page, post_process_reports},
        media_api::post_profile_content_report,
        profile_api::{get_my_profile, post_report_profile_name},
    },
    models::{
        ContentId, GetWaitingReportsPage, ProcessReport, ProcessReports, ReportType,
        UpdateProfileContentReport, UpdateProfileNameReport,
    },
};
use test_mode_test_utils::{Account, prelude::*};

fn set_spam_report_threshold_1(config: ServerConfigEditor) {
    config.server.limits = Some(config::file::LimitsConfig {
        common: Some(config::file::CommonLimitsConfig {
            auto_ban_spam_reporters_invalid_report_threshold: 1,
            auto_ban_spam_reporters_ban_duration:
                simple_backend_utils::time::DurationValue::from_days(1),
            ..Default::default()
        }),
        ..Default::default()
    });
}

async fn get_first_content_id(account: &Account) -> TestResult<ContentId> {
    let result = api_client::apis::media_api::get_profile_content_info(
        &account.media_api(),
        &account.account_id_string(),
        None,
        None,
    )
    .await?;
    let content = result
        .content
        .flatten()
        .and_then(|v| v.content.first().cloned())
        .ok_or(TestError::MissingValue.report())?;
    Ok(*content.cid)
}

async fn process_all_reports_as(context: &mut TestContext, valid: bool) -> TestResult {
    let admin = context.new_admin_and_moderate_initial_content().await?;

    let waiting_reports =
        post_get_waiting_reports_page(&admin.account().account_api(), GetWaitingReportsPage::new())
            .await?;
    assert_ne(waiting_reports.values.len(), 0)?;

    let process_reports: Vec<ProcessReport> = waiting_reports
        .values
        .iter()
        .map(|report| {
            ProcessReport::new(
                (*report.content).clone(),
                *report.info.creator.clone(),
                ReportType::new(report.info.report_type.n),
                *report.info.target.clone(),
                valid,
            )
        })
        .collect();

    post_process_reports(
        &admin.account().account_api(),
        ProcessReports::new(process_reports),
    )
    .await?;

    Ok(())
}

async fn simple_auto_ban_spam_reporters_test(
    mut context: TestContext,
    valid: bool,
    wanted_banned: bool,
) -> TestResult {
    let reporter = context.new_account().await?;
    let target = context.new_account().await?;

    let target_content_id = get_first_content_id(&target).await?;

    let report_result = post_profile_content_report(
        &reporter.account_api(),
        UpdateProfileContentReport::new(target_content_id, target.account_id()),
    )
    .await?;
    assert(!report_result.error.unwrap_or(false))?;

    process_all_reports_as(&mut context, valid).await?;

    let banned = get_account_state(&reporter.account_api())
        .await?
        .state
        .banned
        .unwrap_or_default();
    assert(banned == wanted_banned)?;

    Ok(())
}

#[server_test(modify_server_config_with = "set_spam_report_threshold_1")]
async fn auto_ban_spam_reporters_threshold_1_invalid_report_bans_reporter(
    context: TestContext,
) -> TestResult {
    simple_auto_ban_spam_reporters_test(context, false, true).await
}

#[server_test(modify_server_config_with = "set_spam_report_threshold_1")]
async fn auto_ban_spam_reporters_threshold_1_valid_report_does_not_ban_reporter(
    context: TestContext,
) -> TestResult {
    simple_auto_ban_spam_reporters_test(context, true, false).await
}

#[server_test(modify_server_config_with = "set_spam_report_threshold_1")]
async fn auto_ban_spam_reporters_threshold_1_valid_and_invalid_reports_do_not_ban(
    mut context: TestContext,
) -> TestResult {
    let reporter = context.new_account().await?;
    let target = context.new_account().await?;

    let target_content_id = get_first_content_id(&target).await?;
    let report_result = post_profile_content_report(
        &reporter.account_api(),
        UpdateProfileContentReport::new(target_content_id, target.account_id()),
    )
    .await?;
    assert(!report_result.error.unwrap_or(false))?;

    let target_profile = get_my_profile(&target.account_api()).await?;
    let report_result = post_report_profile_name(
        &reporter.account_api(),
        UpdateProfileNameReport::new(
            target_profile.profile.name.unwrap_or_default(),
            target.account_id(),
        ),
    )
    .await?;
    assert(!report_result.error.unwrap_or(false))?;

    let admin = context.new_admin_and_moderate_initial_content().await?;

    let waiting_reports =
        post_get_waiting_reports_page(&admin.account().account_api(), GetWaitingReportsPage::new())
            .await?;
    assert_eq(waiting_reports.values.len(), 2)?;

    let mut process_reports: Vec<ProcessReport> = Vec::new();
    for report in &waiting_reports.values {
        let is_valid = if report.info.creator.aid == reporter.account_id().aid
            && report.info.target.aid == target.account_id().aid
        {
            if report.info.report_type.n == 2 {
                // Profile content
                true
            } else if report.info.report_type.n == 0 {
                // Profile name
                false
            } else {
                return Err(TestError::InvalidValue.report().into());
            }
        } else {
            return Err(TestError::InvalidValue.report().into());
        };

        process_reports.push(ProcessReport::new(
            (*report.content).clone(),
            *report.info.creator.clone(),
            ReportType::new(report.info.report_type.n),
            *report.info.target.clone(),
            is_valid,
        ));
    }

    post_process_reports(
        &admin.account().account_api(),
        ProcessReports::new(process_reports),
    )
    .await?;

    let banned = get_account_state(&reporter.account_api())
        .await?
        .state
        .banned
        .unwrap_or_default();
    assert(!banned)?;

    Ok(())
}
