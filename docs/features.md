
# Backend features

## Account

* Sign in with Google
* Demo mode accounts for developers (access multiple normal accounts)

## Notifications

* Email
  * Email notifying that account was created
* Push notifications (Firebase)
* WebSocket

## Profile

* Age (18-99 and updates automatically without birthdate)
* Name (max 100 bytes)
* Text (max 2000 bytes)
* Images (max 6 images and the first must be a face image)
* First image crop info (for displaying thumbnail image for the profile)
* Unlimited chat requests enabled boolean
* Last seen time
* Location (exact coordinates are not public)
* Gender (indirectly public)

### Profile attributes

Profile attributes are predefined questions and answers which are visible on
user's profile. The attributes are configurable from server side. Check
[profile_attributes_spec.md](./profile_attributes_spec.md) for details.

### Profile iterator

Server side profile iterator is location based.
Also profile order can be randomized partially (random iterator
starting position).

#### Profile iterator privacy

If a profile is returned from the iterator, the profile owner and the profile
viewer are a match when checking each other's profile age, age range and gender
settings.

##### Profile iterator filtering

Also optional filters can be set.

* Max distance
* Profile attributes
* Last seen time
* Profile created time
* Profile edited time
* Unlimited chat requests enabled boolean

All enabled filters are chained together using logical AND operation.

### Favorite profiles

User can mark an profile as a favorite so that it can be found later for
example if daily chat request is already used.

### Profile statistics

Age and gender statistics for public profiles. Admins can also access
statistics for private profiles and profile statistics history (updated daily).

## Chat

* One-to-one conversations

## Chat security

* Messages are removed from server when sending and delivery is confirmed by
  clients
* Public key management (allows client to implement end-to-end encryption)

## User interaction

* Chat requests (likes)
  * One chat request per day
  * Unlimited chat requests per day
  * Undo once per user

### User interaction security

* Blocking
  * Message sending is prevented with error
  * Sent chat request is invisible for blocker

## News

Simple content management system which for example can be used for informing
users about app version changelogs and terms of service updates.

## Images

* Server image storage size restrictions (max 20 images by default)
* JPEG image processing

### Image security

* Face detection for images ([rustface library](https://github.com/atomashpolskiy/rustface))
* Face image for moderators (security selfie)
* Image removal wait time (90 days by default)

## Security

* [Chat security](#chat-security)
* [User interaction security](#user-interaction-security)
* [Image security](#image-security)
* Account banning
* Account removing wait time (90 days by default)
* Reporting
  * Profile name
  * Profile text
  * Profile images
  * Chat messages
  * Custom reports (configured like profile attributes)

## Privacy

* End-to-end message encryption support
* Profile visibility setting

## Admin API

* Image moderation
* Profile name moderation (manual and allowlist)
* Profile text moderation
* Bot count configuration
* Server performance metrics
  * API usage
  * WebSocket connection count
  * CPU and RAM usage
* Profile statistics
  * Private
  * History

### With manager mode

* Backend data reset (for development only)
* Backend restart (manual and scheduled)
* System reboot (manual and scheduled)
* Update backend binary from GitHub (manual)
* Server logs

## Manager mode

The backend binary can be started in manager mode which for example
can start the backend in server mode.

* [Admin API related manager mode features](#with-manager-mode)
* Maintenance break notifications
* Automatic system reboot scheduling for Ubuntu
* Secure storage management
* Daily backend data backups
  * Media content (image files) syncing
  * Database file backups with retention period

## Bots

User bots for development and debugging. Admin bots for text and image
moderation.

### User bots

* Partially configurable profile
* Random profile images (random color or random image from directory)
* End-to-end message encryption (OpenPGP)

### Admin bots

* Skin color based image moderation ([nude library](https://github.com/kpcyrd/nude-rs))
* Neural network based image moderation ([nsfw library](https://github.com/Fyko/nsfw))
* Large language model (LLM) based text moderation (OpenAI API compatible)

## Other

* Configurable minimum client version
* Configurable API path obfuscation

# Missing backend features

## Account

* Sign in with Apple
* Subscription management

## Email

* Email address confirmation
* Notification emails for chat requests and messages

## Chat

* Server message signing (to make sure that server assigned metadata is valid
  when message is reported)

## Security

* Validity check for reported chat messages
* Account specific API usage statistics
* IP address history

## Data export

* Export account related data to ZIP archive

## Video calls

* Jitsi Meet integration

## Sweepstakes

Admin can create automated and optional sweepstakes
for users.

## Analytics

* Client version tracking

# Possible future backend features

## Account

* Email and password login with two factor authentication
* EU digital wallet
