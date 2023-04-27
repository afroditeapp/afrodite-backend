# API usage

Minimum viable product API

## Account creation

App opens to login screen with Google and Apple single sign on buttons.

`/account/register` is used for now and it just creates a new account.

## Initial setup

The created account is in `initial setup` state. The client will ask
user all questions and fill in user details.

All textual data will be sent with `/account/setup` that path will only be
used when account state is in initial setup.

Client initial setup will create new image moderation request with one flagged
as real camera image and one other image. Check section 'image moderation request'
to see how to do that.

The client initial setup will then request state transfer to `normal` state
using path `/account_api/complete_setup`. Account server will check that all
required information is set to the account and then also check is there really
an moderation request created using internal media server API.

TODO: Remove capablity 'admin_setup_possible' from another document.

## Normal state

Client now gets the account state again using `/account/state` and updates the
client UI state accordingly.

After initial setup the client will go to the profile grid view. Initial image
moderation request currently in moderation. After client receives or asks info
that the first moderation request is moderated, the client will start setup the
profile on the profile server. First the client will update current profile text
and image content IDs using HTTP POST to `/profile_api/profile`. The server will
validate the content IDs using internal API call. After successful HTTP response
the client can update the profile visiblity value using HTTP PUT to
`/account_api/settings/profile_visibility`. The server will do internal API
request to both media and profile servers to update current visiblity status of
account's profile content.

Setting profile visiblity also updates the location index, so that profile is
removed or added depending on the visiblity. By default the profile location is
at (0,0) of the location index. When client changes the location using HTTP PUT
to `/profile_api/location` coordinates are converted to location index key. The
client updates the location before changing the profile visibility.

Client queries about one time events like rejected image moderation requests are
handled using `/media/events`

### Profile grid view

Client will use previously get account state and check if capablity
'view_public_profiles' is visible. If that capability is not visible then client
will check `/media/moderation/request` to see is there a currently ongoing
moderation request. That info will also include current position in the
moderation queue. Client will show moderation info if images are pending
moderation. If not then client will show text "Profile is not set as public".

If capablility 'view_public_profiles' is set then update location with
`/profile/location` and start profile paging `/profile/page/next`.
Refresh is possible when using `/profile/page/reset`.

Paging info will include AccountIds and profile images. Profile images will be
downloaded on the fly using `/media/images/IMG`.

### Opened profile view

When profile is opened from the grid then it's information is get with
`/profile/profiles/ACCOUNT_ID`

### Settings

You can set profile visibility in the grid using `/account_api/settings/profile_visibility`.

### Profile editing

#### Image moderation request

Send or update current image moderation using HTTP PUT to
`/media_api/moderation/request`. HTTP GET to that same address will
get current moderation request status.

Before sending new moderation request images must be loaded to moderation
slots using HTTP PUT `/media_api/moderation/request/slot/{image_slot}`. Image
slots `camera` and `image1` are available.
HTTP GET to that address will
return the image in the slot if there is one.

##### Implementation details
###### Uploading images (and other file related things)

Server contains directory structure like this:

- account1
    - image
        - content_id.l.jpg
        - content_id.h.jpg
    - tmp
        - content_id.raw.jpg (user uploaded)
    - export
        - export1.zip
- account2
    - ...

A user can upload an image to the server using image slot pahts. When server
gets new image upload from the user it creates new content ID (UUID) for the image.
The image will be saved to the user's `tmp` directory.

After upload completes, HTTP response is not yet sent. Server starts a new
process for image processing. The image will be decoded with a Rust image
decoder. After that new images with different qualities are encoded. The new
image files will be saved to the `tmp` directory. The image process quits at this
point. Server will move the new files to the user's `image` directory.
After that HTTP response with new content ID is returned to the user.

Images have different states. If image is in `InSlot` state, another upload to
that same slot will delete the previous image. When moderation request for
a image is started the image's state will go to `InModeration` state. After
moderation completes the image state turns to `Moderated`. SQLite will be used
for storing the image state information. Images also have a boolean value for
preventing other normal users accessing images other than those on the current
profile. Also hidden profiles have additional access check for images - only
matches can download the current profile images.

Directory named `export` is used for saving data export zip-files.

###### Moderation request

After all images are uploaded user will make a new media moderation
request to the server. If user does not have a queue number, then it
will be assigned to the user. Queue ID is valid until it is removed (by user
removing the request or
by admin handling the moderation request).

User will have a possiblity to remove the moderation request or update it.
Updating the moderation request can change the contents of the request
(image UUIDs included), but not the queue number.

When admin starts handling the request the request is copied so that it is
immutable. This prevents the user updating the request anymore.
The Queue ID is removed from active Queue IDs table.
After that the admin can mark the request as accepted or denied.

After these events the request still exists but it is handled. Only creating a
new moderation request deletes the previous one.

### Account deletion

Account can be moved to `pending deletion` state with `/account_api/delete`.
Also deletion date can be queried with HTTP GET to that address.
HTTP DELETE to that address will cancel deletetion request.

### Account ban status

If account is in `banned` state, HTTP GET to `/account_api/ban_status` can be
used for querying ban status.

### Profile flagging

Account can flag profiles with HTTP POST to `/profile_api/flag/{account_id}`.

### Account data export

Each server has different data on it, so client asks account data export from
each server. Server produces ZIP of account data at some point.

Account data export can be created with HTTP POST to
`/{api}/data_export_request`. HTTP GET to that returns status of the current
request. When data request is complete
`/{api}/data_export_request/{zip_file_name}` can be used to download the file.

### Likes

#### Like profiles

Use HTTP POST to `/account_api/like/{account_id}` to like a profile. Response
will tell if it was a match. To remove a
like use HTTP DELETE.

Use HTTP GET to `/account_api/my_likes/page/next` get next page of my likes.
HTTP POST to `/account_api/my_likes/page/reset` to reset paging.

Paging will start from the latest likes.

#### View received likes

Use HTTP GET to `/account_api/received_likes/page/next` get next page of my
likes. HTTP POST to `/account_api/received_likes/page/reset` to reset paging.

Likes will be removed from these lists when match is formed. Paging will start
from the latest likes.

### Bloking

#### Block account

To block visiblity and interactions to specific account use HTTP POST to
`/account_api/block/{account_id}`. Block can be reverted using HTTP DELETE to
that same address.

HTTP GET `/account_api/my_blocks/page/next` can used to get next list of account
which I have been bloked. HTTP POST `/account_api/my_blocks/page/reset` will
reset the paging.

#### Be blocked

Blocked account will get notification about the bloking using WebSocket.
This notification will repeat until received_blocks is queried at least once.

Use HTTP GET to `/account_api/received_blocks/page/next` get next page of received
blocks. HTTP POST to `/account_api/received_blocks/page/reset` to reset paging.

### Chat

#### Sending messages

HTTP POST to `/chat_api/message/send/{account_id}` sends a new message. Message
ID will be returned. Messages can only be sent to matches.

If WebSocket is connected, message delivery info will be received from it.

HTTP GET to `/chat_api/message/status/{message_id}` will tell is the message delivered.

Server saves hashes of latest 5 messages in an conversation. These will be used
for verifying flagged chat reports.

#### Receiving messages

When messages available event is received from WebSocket or as push
notification, the user can then get new messages using HTTP GET
`/chat_api/message/list_new`. Those messages can be marked as received using
HTTP POST `/chat_api/message/mark_received` with a list of message IDs.

### Chat flagging

Chat can be flagged with HTTP POST to `/chat_api/flag/{chat_id}`. Latest 5
messages in the chat will be sent to the server for admin moderation.

##### WebSocket and push notifications

Chat service has WebSocket for sendng events to users. If that is connected,
then push notifications are disabled from server. Other servers can also use
this API for sending events to clients.

### Admin features

#### Image moderation

If capability 'admin_moderate_images' can be found the client displays option to
go image moderation mode. In that mode the app will fetch all images which need
moderation using `/media_api/admin/moderation/page/next`.
That path will get next set of not handled image moderations.
Images in that request
will be downloaded using `/media_api/images/{account_id}/{image_id}`.
It does not matter if image is
accepted or not. Moderation requests have an unique id. That id can be accepted
or not using `/media_admin/admin/moderation/handle_request/{request_id}`.

#### Flagged profiles moderation

Admin can get next page of flagged profiles with HTTP GET to
`/profile_api/admin/moderation/page/next`. The profiles will be handled with
HTTP POST to
`/profile_api/admin/moderfation/handle_flagged/{account_id}`.

#### Flagged chat moderation

Admin can get next page of flagged chats with HTTP GET to
`/chat_api/admin/moderation/page/next`. The profiles will be handled with
HTTP POST to
`/chat_api/admin/moderfation/handle_flagged/{chat_id}`.
