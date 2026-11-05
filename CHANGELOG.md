# Changelog

## [Unreleased]

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
