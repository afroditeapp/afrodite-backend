
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
* Profile attributes (logical AND operation)
* Last seen time
* Unlimited chat requests enabled boolean

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

## Privacy

* End-to-end message encryption support
* Profile visibility setting

## Admin API

* Image moderation
* Profile name moderation (manual and allowlist)
* Profile text moderation
* Bot count configuration
* Server performance metrics
* Profile statistics
  * Private
  * History

### With app-manager

<https://github.com/jutuon/app-manager>

* Backend restart
* Backend reset
* Software update
* Server logs

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

* Profile text and image reporting
* Chat message reporting
* Account specific API usage statistics
* IP address history

## Data export

* Export account related data to ZIP archive

## Video calls

* Jitsi Meet integration

## Sweepstakes

Admin can create automated and optional sweepstakes
for users.

## Performance metrics

* Active websocket connections

## Backups

* Daily backups

## Backend management

Move secure storage management, log viewing, backend restarting and reseting to
backend binary so that app-manager is no longer needed when hosting on VPS.
Software building related features from app-manager are not needed as backend
updates are rare so manual updates are enough.

<https://github.com/jutuon/app-manager>

Consider also changing API obfuscation to happen from compile time to runtime
so that public binary releases would be possible. The internal API port should
host the non obfuscated API in this case to allow using the same generated API
bindings for bots for example.

# Possible future backend features

## Account

* Email and password login with two factor authentication
* EU digital wallet
