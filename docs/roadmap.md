
# Roadmap

If item does not have client/server specified then work for both is needed.

## Internal release 0.2

### Generic
- [ ] Server: email sending
- [ ] Missing push notifications and events
      (likes and image moderation status updates)
- [ ] Server: CPU, RAM, disk and i-node usage to perf history
- [ ] Server: Perf history saving to database
- [ ] Client: Perf history viewing improvements
      (show multiple items, time range)
- [ ] Data export
- [ ] Client: Client only data export/import
- [ ] Server: Mark account to be deleted automatically if specific
      time has passed since the current last seen date.
- [ ] Review new terms and conditions and if declined
      mark account to be removed.
- [ ] Server: Automatic data backups
- [ ] Server: NSFW upload prevention (API needs error for that)

### Account
- [ ] Profile reporting
- [ ] Account removing
- [ ] Account banning (with ban time)
- [ ] Admin view and edit capabilities
- [ ] Client: admin: open profile with account ID

### Chat
- [ ] End-to-end encryption
- [ ] Blocking functionality changes
      (block does not hide the profile from the blocked user's client but all
      interaction is blocked)
- [ ] Client: message manual resend if failure happens
- [ ] Client: possibility to remove messages where sending has failed
- [ ] Client: Unread messages support (with count?)
- [ ] Server: Limit pending messages count
- [ ] Server: Limit message size

### Profile
- [ ] Profile age change only to the valid age
- [ ] Account last seen value
- [ ] Limit likes to one per day (API which has limits for client?)
- [ ] Unlimited likes club
- [ ] Limit like undos to one per day
- [ ] Client: Favorite action should display snackbar
- [ ] Admin view all profiles

### Media
- [ ] Server max image count
- [ ] Remove old images

### Uncertain
- [ ] Client: do profile location edits using long press
      (remove the edit button). Perhaps update the location
      to server once back navigation is happened.
      On the other hand the previous screen also has the edit button.
- [ ] Client: Test swipe gestures to change main screen
- [ ] Client: Swipe gestures for profile image browsing
- [ ] Client: Change filters icon to perhaps settings
- [ ] Client: Long press message for message details
- [ ] Whitelist for profile first names and if no name whitelisted
      then fallback to name initials (2 letters)
