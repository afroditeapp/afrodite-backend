use std::{collections::HashMap, sync::Arc};

use database_profile::current::read::GetDbReadCommandsProfile;
use model::{AccountIdDb, LastSeenTime, LoginSessionInfoInternal};
use model_profile::{
    AccountExtraInfoSettings, AccountIdDbValue, AdminAccountLoginSessionInfo, ProfileIteratorPage,
    ProfileIteratorPageData, ProfileIteratorPageValue, ProfileIteratorSettings,
    ProfileStringIterationPage, ProfileStringModerationContentType,
};
use server_data::{
    DataError, IntoDataError, db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead,
    result::Result,
};
use simple_backend::maxmind_db::IpDb;
use simple_backend_model::{AppAttestationTypeNumber, UnixTime};

define_cmd_wrapper_read!(ReadCommandsProfileIterator);

struct ProfileIteratorDbPage {
    data: Vec<ProfileIteratorPageData>,
    has_more_pages: bool,
    email_addresses: HashMap<i64, String>,
    login_session_infos: HashMap<i64, LoginSessionInfoInternal>,
}

impl ReadCommandsProfileIterator<'_> {
    pub async fn get_latest_created_account_id_db(&self) -> Result<AccountIdDbValue, DataError> {
        self.db_read(move |mut cmds| cmds.profile_admin().iterator().get_latest_account_id_db())
            .await
            .into_error()
    }

    pub async fn get_profile_page(
        &self,
        settings: ProfileIteratorSettings,
        ip_db: Option<Arc<IpDb>>,
    ) -> Result<ProfileIteratorPage, DataError> {
        let extra_info = settings.extra_info();
        let filters = settings.filters();

        // Online status filtering happens after the database query,
        // so the iteration may have to go through multiple database
        // pages to find at least one value for the response page.
        let mut values = Vec::new();
        let mut next_db_page = settings.page;

        let mut has_more_db_pages;

        loop {
            let db_page = self.get_db_page(next_db_page, &settings).await?;

            has_more_db_pages = db_page.has_more_pages;

            self.add_page_values(
                &mut values,
                db_page,
                filters.clone(),
                extra_info,
                ip_db.as_deref(),
            )
            .await?;

            next_db_page += 1;

            if !values.is_empty() || !has_more_db_pages {
                // Returns a possibly partial page as soon as any value
                // is found. The next page number allows the client to
                // continue the iteration.
                break;
            }
        }

        let next_page = has_more_db_pages.then_some(next_db_page);

        Ok(ProfileIteratorPage { values, next_page })
    }

    /// Reads one database page with related extra info.
    async fn get_db_page(
        &self,
        page: i64,
        settings: &ProfileIteratorSettings,
    ) -> Result<ProfileIteratorDbPage, DataError> {
        let extra_info = settings.extra_info();
        let filters = settings.filters();

        let db_settings = ProfileIteratorSettings {
            start_position: settings.start_position,
            page,
            filters: settings.filters.clone(),
            extra_info: settings.extra_info,
        };

        let (data, has_more_pages, email_addresses, login_session_infos) = self
            .db_read(move |mut cmds| {
                let (data, has_more_pages) = cmds
                    .profile_admin()
                    .iterator()
                    .get_profile_page(db_settings)?;

                let account_db_ids: Vec<AccountIdDb> = data.iter().map(|v| v.id.id).collect();

                let email_addresses = if extra_info.email_address {
                    cmds.profile_admin()
                        .iterator()
                        .get_email_addresses(&account_db_ids)?
                } else {
                    vec![]
                };

                let login_session_infos =
                    if extra_info.any_login_session_info() || filters.any_login_session_filter() {
                        cmds.profile_admin()
                            .iterator()
                            .get_login_session_infos(&account_db_ids)?
                    } else {
                        vec![]
                    };

                Ok((data, has_more_pages, email_addresses, login_session_infos))
            })
            .await
            .into_error()?;

        let email_addresses: HashMap<i64, String> = email_addresses
            .into_iter()
            .filter_map(|(account_db_id, email)| Some((account_db_id.0, email?)))
            .collect();

        let login_session_infos: HashMap<i64, LoginSessionInfoInternal> = login_session_infos
            .into_iter()
            .map(|(account_db_id, info)| (account_db_id.0, info))
            .collect();

        Ok(ProfileIteratorDbPage {
            data,
            has_more_pages,
            email_addresses,
            login_session_infos,
        })
    }

    /// Collects response values from one database page. Accounts
    /// filtered out by the online status filtering are skipped.
    async fn add_page_values(
        &self,
        values: &mut Vec<ProfileIteratorPageValue>,
        db_page: ProfileIteratorDbPage,
        filters: model_profile::AccountFilters,
        extra_info: AccountExtraInfoSettings,
        ip_db: Option<&IpDb>,
    ) -> Result<(), DataError> {
        let online_filter = filters.account_online;

        for data in db_page.data {
            let account_db_id = data.id.id.0;

            let is_online = if online_filter.is_some() || extra_info.account_online {
                self.cache()
                    .read_cache(data.id.uuid, |e| {
                        Ok(e.profile.last_seen_time().last_seen_time_private()
                            == LastSeenTime::ONLINE)
                    })
                    .await
                    .into_error()?
            } else {
                false
            };

            if let Some(wanted_online) = online_filter
                && wanted_online != is_online
            {
                continue;
            }

            if let Some(not_country) = &filters.login_session_ip_country_is_not
                && login_session_ip_country_is(
                    db_page.login_session_infos.get(&account_db_id),
                    ip_db,
                    not_country,
                )
            {
                continue;
            }

            if let Some(wanted_attestation_validity) =
                filters.login_session_basic_app_attestation_validity
                && wanted_attestation_validity
                    != basic_app_attestation_valid(db_page.login_session_infos.get(&account_db_id))
                        .unwrap_or(false)
            {
                continue;
            }

            values.push(ProfileIteratorPageValue {
                account_id: data.id.uuid,
                age: data.profile.age,
                name: data.profile.profile_name,
                account_state: extra_info
                    .account_state
                    .then(|| data.state.state_container().account_state()),
                account_online: extra_info.account_online.then_some(is_online),
                profile_visibility: extra_info
                    .profile_visibility
                    .then_some(data.state.profile_visibility_state_number),
                email: extra_info
                    .email_address
                    .then(|| db_page.email_addresses.get(&account_db_id).cloned())
                    .flatten(),
                login_session: if extra_info.login_session_client_type
                    || extra_info.login_session_login_method
                    || extra_info.login_session_app_attestation_info
                {
                    db_page.login_session_infos.get(&account_db_id).map(|info| {
                        AdminAccountLoginSessionInfo {
                            client_type: extra_info
                                .login_session_client_type
                                .then_some(info.client_platform)
                                .flatten(),
                            login_method: extra_info
                                .login_session_login_method
                                .then_some(info.login_method_number),
                            app_attestation_type: extra_info
                                .login_session_app_attestation_info
                                .then_some(info.app_attestation_type_number)
                                .flatten(),
                            app_attestation_app_integrity: extra_info
                                .login_session_app_attestation_info
                                .then_some(info.app_attestation_app_integrity)
                                .flatten(),
                            app_attestation_device_integrity: extra_info
                                .login_session_app_attestation_info
                                .then_some(info.app_attestation_device_integrity)
                                .flatten(),
                            app_attestation_failed: extra_info
                                .login_session_app_attestation_info
                                .then_some(info.app_attestation_failed)
                                .flatten(),
                        }
                    })
                } else {
                    None
                },
                login_session_ip_country: if extra_info.login_session_ip_country {
                    db_page
                        .login_session_infos
                        .get(&account_db_id)
                        .map(|info| info.client_ip.to_ip_addr())
                        .and_then(|ip| ip_db.and_then(|db| db.get_country(ip)))
                        .map(|v| v.into_string())
                } else {
                    None
                },
            });
        }

        Ok(())
    }

    pub async fn profile_string_iteration_page(
        &self,
        content_type: ProfileStringModerationContentType,
        page: i64,
        origin: UnixTime,
    ) -> Result<ProfileStringIterationPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .iterator()
                .profile_string_iteration_page(content_type, page, origin)
        })
        .await
        .into_error()
    }
}

/// Returns true if the latest login session IP country is the provided
/// country. Accounts without a login session or an unknown IP country
/// are treated as not matching the provided country.
fn login_session_ip_country_is(
    info: Option<&LoginSessionInfoInternal>,
    ip_db: Option<&IpDb>,
    country: &str,
) -> bool {
    let resolved = info
        .map(|info| info.client_ip.to_ip_addr())
        .and_then(|ip| ip_db.and_then(|db| db.get_country(ip)))
        .map(|v| v.into_string());

    match resolved {
        Some(resolved_country) => resolved_country.eq_ignore_ascii_case(country),
        // No login session or unknown IP country.
        None => false,
    }
}

/// Returns Some(true) if the latest login session has valid basic app
/// attestation, Some(false) if it does not, and None if the account
/// does not have a login session.
///
/// Valid basic app attestation means a valid iOS app attestation token
/// or valid Android app and device integrity values.
fn basic_app_attestation_valid(info: Option<&LoginSessionInfoInternal>) -> Option<bool> {
    let info = info?;

    match info.app_attestation_type_number {
        Some(AppAttestationTypeNumber::AppleAppAttest) => {
            // iOS: valid app attest token and no errors.
            Some(!info.app_attestation_failed.unwrap_or(true))
        }
        Some(AppAttestationTypeNumber::PlayIntegrity) => {
            // Android: both app and device integrity valid and no errors.
            Some(
                info.app_attestation_app_integrity.unwrap_or(false)
                    && info.app_attestation_device_integrity.unwrap_or(false)
                    && !info.app_attestation_failed.unwrap_or(true),
            )
        }
        // Web and bot clients do not provide app attestation.
        None => Some(false),
    }
}
