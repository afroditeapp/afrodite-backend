# Account states

When account is created the first state is 'initial setup'. That happens when
user logins using Apple/Google single sign on.

Account has several capablities for example granting admin related features.
Admin can give some capablities or also user actions can grant some of those.

Initial admin must be set from the server settings. This admin has
'admin_modify_capablities' and 'admin_setup_possible' capablities.

Terms of Service updates are notified early using email, so no need to save
accepted version to the server.

## Initial setup

When user account is in this state the app launches to the account setup screen.
In this screen it is possible to move account to 'pending deletion' state. Also
if admin gives user some admin capabilities it is possible to partly skip
the initial setup.

Possible state transfers:
* Normal
* Banned
* Pending deletion

### Capablities

* 'admin_setup_possible' - User can select if complete initial setup or minimal
admin setup should be done when doing initial setup.

## Normal

Initial setup is now completed.

Possible state transfers:
* Banned
* Pending deletion

### Capabilities

Admin:

* 'admin_modify_capablities' - Add and remove capablities exept this one.
* 'admin_moderate_profiles' - View and moderate all user flagged profiles.
* 'admin_moderate_images' - Account image moderation is now possible.
* 'admin_view_private_info' - View private account info.
* 'admin_view_profile_history' - View public and private info changes that
accounts has.
* 'admin_view_all_profiles' - View all public and private profiles. Also goto to
to specific profile by email is enabled.
* 'admin_ban_profile' - Banning some profile is now possible.

Normal:

* 'view_public_profiles' - View public profiles. This is added if user sets
it's profile to public. Removed if profile is private.

## Banned

Account is banned temporarely or permanently. It is possible to move to
'pending deletion' state.

Possible state transfers:
* Pending deletion
* Normal

### Capabilities

* 'banned_edit_profile' - Edit profile and then send it to moderation again.

## Pending deletion

Account will be deleted after for example 3 months. Account which is not yet
deleted can be restored to previous state.

Possible state transfers:
* Initial setup
* Normal
* Banned

# Profile

Profile can have three states:
* Hidden
* Visible only to confirmed profiles
* Visible to all


# TODO

Think about the case where chat could be encrypted locally when either chat
participant blocks the user.
