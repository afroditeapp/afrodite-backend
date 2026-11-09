# Changelog

## [Unreleased]

### Added

- Send email when account deletion is initiated automatically.
- Init account deletion for accounts which are in initial setup state and
  have been inactive for
  `init_deletion_for_initial_setup_inactive_accounts_wait_duration`
  (default 14 days). The deletion initiated email is sent as well.
- Cleanup `used_account_ids` rows for deleted accounts which never
  completed the initial setup. Cleanup happens during daily scheduled
  tasks one day after account deletion.
- New admin API route `get_profile_string_iteration_page` for iterating
  all profile name/text values in descending profile edited time order.
  Iteration page number and origin edited time value can be set.
- New admin API route `get_media_content_iteration_page` for iterating
  all media content items in descending content upload time order. The
  `only_security_content` query parameter can be used to iterate only
  security content items. Iteration page number and origin upload time
  value can be set.
- Admin API `get_account_login_session_info` now returns also whether the
  account is currently online.

## [0.26.0] - 2026-11-05

## Changed

- Add `initial_setup_completed` and `account_deleted_unix_time` columns to
  `used_account_ids` database table. The initial setup flag survives account
  deletion and the deletion time is set when account data is removed.
- Allow '+' character in local part of email address.
- Update profile and profile content edit times only on user visible changes.
  This avoids showing profiles without any visible changes in automatic
  profile search results.

## [0.25.5] - 2026-11-02

### Fixed

- Add missing 1 second wait time to post_remote_get_bots API.
- Breaking change: Fix admin API post_set_news_publicity HTTP method from
  DELETE to POST.

## [0.25.4] - 2026-10-28

- DB stability
- User API stability
