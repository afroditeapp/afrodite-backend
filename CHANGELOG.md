# Changelog

## [Unreleased]

### Changed

- Make Linux release builds reproducible

## [0.27.0] - 2026-11-23

### Added

- Save Google Play Integrity API stronger verdict info to login session
  info: whether the device recognition verdict contained
  `MEETS_STRONG_INTEGRITY` and the Android SDK version from the verdict
  token device attributes.
- New query parameter `is_received_like` for APIs `get_profile`,
  `get_profile_binary`, `get_profile_content_info`,
  `get_profile_content_info_binary` and `get_content`. When set to `true`,
  access to the requested data is allowed if the requested profile has sent
  a like to current account. Blocking can be used to cancel the access.
- New APIs `POST /chat_api/sent_likes` and
  `GET /chat_api/sent_likes/initial_state` for listing likes sent by
  the current account, paged similarly to matches listing. The listing
  uses new `sent_like_id` column in `account_interaction` which is
  cleared when a like transitions into a match.

### Changed

- Profile statistics API `get_profile_statistics` now returns private
  profile statistics in new optional field `private_age_counts`
  alongside public statistics.
- Profile statistics now exclude bot accounts from age counts and
  profile counts. This affects also profile statistics history which
  is saved daily.
- Deprecated the `profile_visibility` query parameter of API
  `get_profile_statistics`.
- The first account deletion reminder email is now sent only when 30 or
  fewer days remain before deletion, instead of on the first scheduled
  tasks run after deletion was scheduled.
- Account deletion reminder emails are sent only when the deletion was
  initiated automatically for an inactive account.
- Deleting the current security content no longer resets face verified
  values from the account's media content or security content verified
  values from the account's current media content. Verification status
  stays until new security content is set or verified values are edited
  by an admin.

### Fixed

- Received likes listing now skips profiles which are blocked by either
  account.

## [0.26.1] - 2026-11-09

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
  account is currently online and the last seen time of the account.

### Changed

- Breaking change: Admin API `get_admin_profile_iterator_page` is now
  a POST endpoint which accepts its settings as JSON request body.
  Breaks the previous GET version which did not support account filters
  or extra info.

## [0.26.0] - 2026-11-05

### Changed

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
