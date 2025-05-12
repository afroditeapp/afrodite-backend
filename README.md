# Afrodite

Afrodite is a dating app aiming to change dating app market to offer more
ethical, private and secure dating apps. Written using modern technologies,
Flutter and Rust, the app offers ethical profile browsing centered UI together
with private and secure end-to-end encrypted chat messaging. The app supports
Android and iOS platforms.

This repository contains the backend part. [Frontend repository](https://github.com/afroditeapp/afrodite-frontend)

The app is under development and it is not ready for production.

[Screenshots](https://github.com/afroditeapp#screenshots) |
[Video](https://afroditeapp.github.io/videos/basic-usage.webm)

## Features

Check [features.md](docs/features.md).

## Building and running

Tagged development preview versions (0.x) of frontend and backend
with the same minor version number are compatible with each other.
Main branch might be broken or incompatible with the frontend.

1. Install [dependencies](#dependencies).

2. Build and run the backend.

```
make run-release
```

3. Configure backend using [config files](#config-files) and restart it.

4. Optionally install [development dependencies](#development-dependencies).

### Dependencies

#### Ubuntu 22.04

1. Install [Rust](https://www.rust-lang.org/learn/get-started).

2. Install other dependencies.

```
sudo apt install build-essential pkg-config libsqlite3-dev
```

#### macOS

1. Install [Rust](https://www.rust-lang.org/learn/get-started) and
   [Homebrew](https://brew.sh).

2. Install other dependencies.

```
brew install openssl@3
```

### Development dependencies

Command `make reset-database` requires `diesel_cli`.

```
cargo install diesel_cli --no-default-features --features sqlite
```

Command `make update-api-bindings` requires `openapi-generator-cli`.

1. Install node version manager (nvm) <https://github.com/nvm-sh/nvm>
2. Install latest node LTS with nvm. For example `nvm install 18`
3. Install openapi-generator from npm.
   `npm install @openapitools/openapi-generator-cli -g`

## Config files

Check backend code located at `crates/config` and `crates/simple_backend_config`
for all available config file options.

### Simple config for debugging and development

`server_config.toml`

```toml
[[demo_mode]]
database_id = 0
password_stage0 = "test"
password_stage1 = "tThlYqVHIiY="
access_all_accounts = true

[grant_admin_access]
email = "admin@example.com"
debug_for_every_matching_new_account = true
debug_match_only_email_domain = true
```

`simple_backend_config.toml`

```toml
[general]
# Run backend in debug mode which
#  - disables TLS config check,
#  - adds additional info to some error messages,
#  - enables Swagger UI on local bot API port and
#  - changes other things as well.
# Check backend code for details.
debug = true
debug_override_face_detection_result = true

[socket]
public_api = "127.0.0.1:3000"
local_bot_api_port = 3002
```

With the above options Swagger UI will be available on
<http://localhost:3002/swagger-ui>.

## Questions and answers

### Where the name comes from?

The name is [Aphrodite](https://en.wikipedia.org/wiki/Aphrodite) in Finnish.

### Why the project is permissively licensed?

That will make easier for businesses to modify, extend and
host their own versions of the service. This helps businesses
to enter to the dating app market without huge technical investments.

It is also possible to reduce bot users with security by obscurity. Backend
API can be modified and the modified frontend and backend code can be hidden.
(Yes, I know that this is not a reliable solution for preventing bots. I have
planned to implement support for EU digital wallet when that is possible.
It will most likely solve bot user issues if all users must verify their
identity with the wallet.)

### Where can I download the app?

It is not available anywhere yet.

A release for Finland with another branding will happen when the app
is considered ready for production.

Global app releases are not planned and are not possible in practice
as the backend is monolithic. Also the backend profile iterator API uses
2D matrix with jump info for iterating profiles from nearest to farthest, so
it might use a lot of RAM depending on how large the matrix is.

## Contributions

Bug fixes or documentation improvements are usually welcome. For new features,
please open a new issue and ask could the new feature be accepted. The
feature might not be accepted for example if it is considered unethical
or adds too much maintenance burden.

Contributions must have the same license as the project (dual-licensed with
MIT and Apache 2.0).

Also note that TODO comments and documentation might be outdated.

## License

MIT License or Apache License 2.0
