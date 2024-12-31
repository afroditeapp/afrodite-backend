# Afrodite
Afrodite is a dating app. This
repository contains the backend part. [Frontend repository](https://github.com/jutuon/afrodite-frontend)

The app is under development and it is not ready for production.

<img src="https://raw.githubusercontent.com/jutuon/afrodite-frontend/refs/heads/images/profiles-view.jpg" alt="Profiles view screenshot" width="30%">

## Features

Check [features.md](docs/features.md).

## Building and running

Tagged development preview versions (0.x) of frontend and backend
with the same minor version number are compatible with each other.
Main branch might be broken or incompatible with the frontend.

1. Update Git submodule `crates/app-manager`.

```
git submodule update --init
```

2. Install [dependencies](#dependencies).

3. Build and run the backend.

```
make run-release
```

4. Configure backend using [config files](#config-files) and restart it.

5. Optionally install [development dependencies](#development-dependencies).

### Dependencies

#### Ubuntu 22.04

1. Install [Rust](https://www.rust-lang.org/learn/get-started).

2. Install other dependencies.

```
sudo apt install build-essential libssl-dev pkg-config
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

### Debugging and development configuration options

`server_config.toml`

```toml
[[demo_mode]]
database_id = 0
password_stage0 = "test"
password_stage1 = "tThlYqVHIiY="
access_all_accounts = true

[grant_admin_access]
email = "admin@example.com"
for_every_matching_new_account = true

# ...
```
`simple_backend_config.toml`
```toml
# Run backend in debug mode which
#  - disables TLS config check,
#  - enables better error messages,
#  - enables Swagger UI on server internal API port and
#  - changes other things as well.
# Check backend code for details.
debug = true
debug_override_face_detection_result = true

[socket]
public_api = "127.0.0.1:3000"
internal_api = "127.0.0.1:3001"

# ...
```

With the above options Swagger UI will be available on
<http://localhost:3001/swagger-ui>.

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

Only bug fixes or documentation improvements are accepted at the moment.

Contributions must have the same license as the project (dual-licensed with
MIT and Apache 2.0).

## License

MIT License or Apache License 2.0
