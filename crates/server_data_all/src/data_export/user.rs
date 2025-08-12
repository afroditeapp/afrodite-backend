use database::{DbReadMode, DieselDatabaseError, current::read::GetDbReadCommandsCommon};
use database_account::current::read::GetDbReadCommandsAccount;
use database_media::current::read::GetDbReadCommandsMedia;
use database_profile::current::read::GetDbReadCommandsProfile;
use model::{
    Account, AccountIdInternal, AdminNotification, AdminNotificationSettings,
    ClientConfigSyncVersion, ClientLanguage, ClientType, GetApiUsageStatisticsResult,
    GetApiUsageStatisticsSettings, GetIpAddressStatisticsResult, InitialSetupCompletedTime,
    LatestBirthdate, OtherSharedState, PushNotificationDbState, ReportId,
    ReportIteratorQueryInternal, ReportProcessingState, ReportTypeNumber, UnixTime,
};
use model_account::{AccountData, AccountEmailSendingStateRaw, AccountSetup, AccountStateTableRaw};
use model_chat::AccountAppNotificationSettings;
use model_media::ContentInfoDetailed;
use model_profile::GetMyProfileResult;
use serde::Serialize;
use server_data::data_export::SourceAccount;

// TODO(future): Add news to data export. This is low priority task as
//               only admins can create or edit news.

// TODO(prod): Add more data to data export JSON

#[derive(Serialize)]
pub struct UserDataExportJson {
    // Common
    id: AccountIdInternal,
    account: Account,
    shared_state: DataExportSharedState,
    sent_reports: Vec<DataExportReport>,
    common_state: DataExportCommonState,
    api_usage_statistics: GetApiUsageStatisticsResult,
    ip_address_statistics: GetIpAddressStatisticsResult,
    admin_notification_settings: AdminNotificationSettings,
    admin_notification_subscriptions: AdminNotification,

    // Account
    account_data: AccountData,
    account_setup: AccountSetup,
    email_sending_states: AccountEmailSendingStateRaw,
    account_state_table: AccountStateTableRaw,
    account_notification_settings: AccountAppNotificationSettings,

    // Profile
    my_profile: GetMyProfileResult,

    // Media
    pub content: Vec<ContentInfoDetailed>,

    // Other
    note: &'static str,
}

#[derive(Serialize)]
struct DataExportSharedState {
    latest_birthdate: LatestBirthdate,
    is_bot_account: bool,
    initial_setup_completed_unix_time: InitialSetupCompletedTime,
}

impl DataExportSharedState {
    fn new(state: OtherSharedState) -> Self {
        Self {
            latest_birthdate: state.latest_birthdate(),
            is_bot_account: state.is_bot_account,
            initial_setup_completed_unix_time: state.initial_setup_completed_unix_time,
        }
    }
}

#[derive(Serialize)]
struct DataExportReport {
    pub id: ReportId,
    pub processing_state: ReportProcessingState,
    pub report_type: ReportTypeNumber,
    pub creation_time: UnixTime,
}

impl DataExportReport {
    fn query(
        current: &mut DbReadMode,
        id: AccountIdInternal,
    ) -> error_stack::Result<Vec<Self>, DieselDatabaseError> {
        let mut data_export_reports = vec![];
        let mut query = ReportIteratorQueryInternal {
            start_position: UnixTime::default(),
            page: 0,
            aid: id,
            mode: model::ReportIteratorMode::Sent,
        };
        loop {
            let reports = current
                .common_admin()
                .report()
                .get_report_iterator_page_internal(query.clone())?;
            if reports.is_empty() {
                break;
            }
            for r in reports {
                data_export_reports.push(DataExportReport {
                    id: r.id.into(),
                    processing_state: r.info.processing_state,
                    report_type: r.info.report_type.into(),
                    creation_time: r.info.creation_time,
                });
            }
            query.page += 1;
        }
        Ok(data_export_reports)
    }
}

#[derive(Serialize)]
struct DataExportCommonState {
    pub client_config_sync_version: ClientConfigSyncVersion,
    pub push_notifications: PushNotificationDbState,
    pub client_login_session_platform: Option<ClientType>,
    pub client_language: ClientLanguage,
}

impl DataExportCommonState {
    fn query(
        current: &mut DbReadMode,
        id: AccountIdInternal,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        Ok(Self {
            client_config_sync_version: current
                .common()
                .client_config()
                .client_config_sync_version(id)?,
            push_notifications: current
                .common()
                .push_notification()
                .push_notification_db_state(id)?,
            client_login_session_platform: current
                .common()
                .client_config()
                .client_login_session_platform(id)?,
            client_language: current.common().client_config().client_language(id)?,
        })
    }
}

pub fn generate_user_data_export_json(
    current: &mut DbReadMode,
    id: SourceAccount,
) -> error_stack::Result<UserDataExportJson, DieselDatabaseError> {
    let id = id.0;
    let data = UserDataExportJson {
        id,
        account: current.common().account(id)?,
        shared_state: DataExportSharedState::new(current.common().state().other_shared_state(id)?),
        sent_reports: DataExportReport::query(current, id)?,
        common_state: DataExportCommonState::query(current, id)?,
        api_usage_statistics: current.common_admin().statistics().api_usage_statistics(
            id,
            GetApiUsageStatisticsSettings::get_all_statistics(id.uuid),
        )?,
        ip_address_statistics: current
            .common_admin()
            .statistics()
            .ip_address_statistics(id, None, None)?,
        admin_notification_settings: current
            .common_admin()
            .notification()
            .admin_notification_settings(id)?,
        admin_notification_subscriptions: current
            .common_admin()
            .notification()
            .admin_notification_subscriptions(id)?,
        account_data: current.account().data().account_data(id)?,
        account_setup: current.account().data().account_setup(id)?,
        email_sending_states: current.account().email().email_sending_states(id)?,
        account_state_table: current.account().data().account_state_table_raw(id)?,
        account_notification_settings: current
            .account()
            .notification()
            .app_notification_settings(id)?,
        my_profile: current.profile().data().my_profile(id, None)?,
        content: {
            let internal_current_media = current
                .media()
                .media_content()
                .get_account_media_content(id)?;
            internal_current_media
                .into_iter()
                .map(|m| m.into())
                .collect()
        },
        note: "If you created or edited news, that data is not currently included here.",
    };

    Ok(data)
}
