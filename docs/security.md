# App backend security

## App session security

Two factor auth for reconnecting to server is a must. With that it is not
in practise possible to guess the session key.

How to archieve that?

### Option 1: Persistent HTTP connection to server

Client does the normal login procedure and gets API key. Also start session
token is recieved with the API key.

When client wants to use HTTP API it needs to use persistent connection to it.
Every connection needs to be started with `/common_api/validate_access_token` and
that requires the API key. If that succeedes
`/common_api/validate_session_token` request is done. If that fails, logout will
be done, which in practise removes the current API key and session token. If the
request is successfull then server will update the current connection IP and
port for that account. Every other api requests will check that API key, IP and
port matches.

IP and port should be reseted when connection closes or timeouts. How to do that?

### Option 2: Websocket

Client does the normal login and gets API key and session token (connection two
factor token). After that it connects to `/common_api/connect` with API key and
establishes Websocket connection. First message in the websocket is the session
token. If that fails then connection quits and API key and session token resets.

The connection is now authenticated and the client is really the account owner.
All API calls are now available automatically through the websocket with making
HTTP requests over the current websocket connection. HTTP requests still have
the API key in place, but also axum's ConnectionInfo has that. Websocket code
adds that to it. If server is not running in debug mode, then equality of those
is checked.

With websocket it is also possible to send events to the clients. Maybe
websockeet code could add client connection handle to the account in ram data
store?

Multiple websocket conenctions with same credentials is possible but client does
not support that. If that is the case then, the older event handle is discarded
and API requests can be done the same way. Maybe that is possible if IP address
changes and there is no disconnect to previous connection? Anyway, good that
this case also works!

Best to investigate first how google single sign on works before changing
anything.


----------------------------------------------

New plan as WebSocket method seems to have same probablity to session hijacking
as the normal API key auth.


1. Add refresh token support to the app. The login route will return the refresh
token (base64 encoded very long random value, 1 KiB number) and the current
access token (old API key; base64 encoded, 256 bit lenght). The access token
will only be useful when server has debug mode enabled, as client IP address
will not be validated in that case. When it is disabled the IP address is not
currently set.

2. The client connects websocket to the server. First the client will send the
refresh token as Binary message to the server. If it is not valid, the server
just disconnects. If it is valid, then the server will send a new refresh token
to the client, set current IP address for the account and after that the new
access token as Binary message. The access token is valid until WebSocket
closes. The server will send events to server using Text messages.

3. All microservices have their own refresh token and access token. The account
server will set those up and return in login information.


# Chat

Chat most likely contain sensitive information, so only to be delivered chat
messages should be stored on the server. Delivered messages are not stored on
the server. Server makes JWT token from every chat message, which client can
send back in case conversation is reported. Messages stored to the server use
cesar cipher to make accidental reading of chat message contents not possible.
