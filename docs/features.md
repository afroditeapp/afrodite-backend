
# Backend features

## Account

* Sign in with Apple
* Sign in with Google
* Demo mode accounts for developers (access multiple normal accounts)

## Notifications

* Email
  * Email notifying that account was created
* Push notifications (Firebase)
  * Messages
  * Likes
  * Image moderation completed
  * Profile text moderation completed
  * Automatic profile search results available
  * News
  * Admin notification (moderator work available)
* WebSocket
  * Used for event sending instead of push notifications if connected

## Profile

* Age (18-99 and updates automatically without birthdate)
* Name (max 100 bytes)
* Text (max 2000 bytes)
* Images (max 6 images and the first must be a face image)
* First image crop info (for displaying thumbnail image for the profile)
* Unlimited chat requests enabled boolean
  * Automatic daily disabling (server config)
* Last seen time
* Location (exact coordinates are not public)
* Gender (indirectly public)

### Profile attributes

Profile attributes are predefined questions and answers which are visible on
user's profile. The attributes are configurable from server side. Check
[profile_attributes_spec.md](./profile_attributes_spec.md) for details.

For attributes with groups an CSV file can be used to load the attribute
values.

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

* Min and max distance
* Min and max profile text length
* Profile attributes
  * Wanted values (logical OR or logical AND)
  * Unwanted values (logical AND)
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

### Automatic profile search

Backend searches for updated profiles on every weekday by default.
User sees an notification if profiles are found. User can configure
the search with these options:

* Search only new profiles
* Use current max distance filter
* Use current profile attribute filters
* Search on specific weekdays

## Chat

* One-to-one conversations

## Chat security

* Messages are removed from server when sending and delivery is confirmed by
  clients
* End-to-end encrypted messages (OpenPGP)
* Authenticy verification for reported messages

## User interaction

* Chat requests (likes)
  * Optional daily limit for chat requests
    * When sending a chat request to someone who has unlimited chat requests
      enabled, the available chat requests does not decrease.
  * Undo once per user

### User interaction security

* Blocking
  * Message sending is prevented with error
  * Sent chat request is invisible for blocker

## News

Simple content management system which for example can be used for informing
users about app version changelogs and terms of service updates.

## Statistics

* [Profile statistics](#profile-statistics)
* WebSocket connection statistics
  * Hourly data for previous 24 hours
  * Min, max and average connection counts
  * All connections and profile gender specific connections
  * Bots are excluded
* Account count (bots excluded)
* Online accounts count (bots excluded)

## Images

* Server image storage size restrictions (max 20 images by default)
* JPEG image processing

### Image security

* Face detection for images ([rustface library](https://github.com/atomashpolskiy/rustface))
* Face image for moderators (security selfie)
* Image removal wait time (90 days by default)
* NSFW detection ([nsfw library](https://github.com/Fyko/nsfw))

## Security

* [Chat security](#chat-security)
* [User interaction security](#user-interaction-security)
* [Image security](#image-security)
* Account banning
* Account removing wait time (90 days by default)
* Inactive account automatic logout (365 days by default)
* Account specific API usage statistics
* IP address history
  * IP country info (MaxMind GeoLite2 database support)
  * Configure manual IP range and network lists and
    see is the IP on some of the lists
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
* Automatic actions (for example send a message if a message is received)

### Admin bots

* Neural network based image moderation ([nsfw library](https://github.com/Fyko/nsfw))
* Large language model (LLM) based moderation (OpenAI API compatible)
  * Profile texts
  * Profile images

## Analytics

* Client version tracking

## Video calls

* Jitsi Meet integration (JWT token and meeting room name generation)

## Other

* Configurable minimum client version
* Configurable API path obfuscation

# Missing backend features

## Account

* Possibility to enable email and password login after account creation
  * Email login code sending and weekly sending limit
* Profile age, name and picture verification using
  EU digital wallet
* Login method management

## Email

* Email address verification
* Email address changing
* Notification emails for chat requests and messages
* Email language selection

## Data export

* Export account related data to ZIP archive

# Possible future backend features

## Account

* EU digital wallet login method
* Subscription management

## Sweepstakes

Admin can create automated and optional sweepstakes
for users.
