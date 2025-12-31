# Architecture

## Processes and connections

### Server

Only one computer can run the server. Running multiple instances is not possible as in-memory cache contains data which does not go directly to database.

Image processing runs in a separate process. Server mode starts backend in image processing mode and sends image processing requests via standard input.

If local bots are enabled, server mode will start backend in bot client mode. Login for local bots happens using specific API. Remote bots are also possible with remote bot login API.

Clients connect to server with login method specific API route and connects WebSocket. APIs which require an access token will require a WebSocket connection.

Server connects to local manager process to receive info when system will restart to complete system updates. Server can also create new connections to manager process for new manager API requests.

### Manager

Manager mode does system and service management, like system reboots, backend restarts, backup transfers and system info providing to clients. Using manager mode is optional.

Manager starts and stops the server systemd service when needed.

Manager can forward manager API requests to another manager for example to reboot the computer which runs the another manager. Accessing the other manager does not need an open TCP port if the another manager is configured to create a connection between managers for API request forwarding.

Two managers are needed to transfer backups to another computer. The built-in backup solution works like this:

1. Server requests backup transfer from local manager.
2. Local manager forwards the request to remote manager.
3. Remote manager accepts the request and starts backup session. Image backup is refreshed and new backup is creared from each SQLite database.

## Code

### Server

Server mode code is structured as layers in this order: simple backend (general backend logic), model, database, data (in-memory cache and database access for API routes), API routes and server (implements trait from simple backend for building runnable server with daily automatic tasks)

Some layers have been split to several crates to speed up compilation. Categories for the crates are common, account, profile, media and chat. Common crate is accessible from all other crates, but for example accessing profile crate from chat crate is not possible to make compiling crates parallel. To workaround this limit data layer has "all" crate which implements trait which provides actions which depend on multiple components.

## History

Original plan was to make possible to split server mode to 4 microservices: account, profile, media and chat. That was too complex to implement so partial support for that was removed. Main issue with the microservice design was data dependencies between components. For example profile access needs match status info from chat server when profile is set to private.
