# Afrodite
Afrodite is a permissively licensed dating app based on profile browsing. This
repository contains the backend part.

The app is under development and it is not ready for production.

## Running

```
make run
```

Add `debug = true` to config file and restart server.

<http://localhost:3000/swagger-ui/>

### Ubuntu 20.04

```
sudo apt install libssl-dev
cargo install diesel_cli --no-default-features --features sqlite
```

### MacOS

Install OpenSSL <https://docs.rs/openssl/latest/openssl/>
```
brew install openssl@1.1
```

```
cargo install diesel_cli --no-default-features --features sqlite
```


## Update server API bindings

1. Install node version manager (nvm) <https://github.com/nvm-sh/nvm>
2. Install latest node LTS with nvm. For example `nvm install 18`
3. Install openapi-generator from npm.
   `npm install @openapitools/openapi-generator-cli -g`
4. Start the backend in debug mode.
5. Generate bindings
```
openapi-generator-cli generate -i http://localhost:3000/api-doc/app_api.json -g rust -o crates/api_client --package-name api_client
```

## Reset database

```
make reset-database
```

## Manual database modifications

Open database with sqlite3 `sqlite3 database.file`.

Run command `PRAGMA foreign_keys = ON;`

All data: `.dump`

## Count lines of code

`find src -name '*.rs' | xargs wc -l`

Commit count:

```
git rev-list --count HEAD
```


# TLS certificate generation

## Root certificate

Generate private key:

```
openssl genrsa -out root-private-key.key 4096
```

Create certificate signing request (CSR):
```
openssl req -new -sha256 -key root-private-key.key -out root-csr.csr
```

100 years = 36500 days

Sign root certificate:
```
openssl x509 -req -sha256 -days 36500 -in root-csr.csr -signkey root-private-key.key -out root.crt
```

## Server certificate

Use domain as Common Name. IP address does not work with Dart and Rustls.

```
openssl genrsa -out server-private-key.key 4096
openssl req -new -sha256 -key server-private-key.key -out server.csr
openssl x509 -req -in server.csr -CA ../root/root.crt -CAkey ../root/root-private-key.key -CAcreateserial -out server.crt -days 365 -sha256
```

## Viewing certificates

```
openssl x509 -in server.crt -text -noout
```

# Bot mode

```
RUST_LOG=debug cargo run -- test --tasks 10 --save-state --no-servers --test bot
```

# Update app-manager submodule to latest

git submodule update --remote --merge


# Building script for Multipass VM

Script which can be used when when app-manager is installed to multipass VM
and source files are mounted. Replace SRC_DIR_LOCATION with the location of
mouted source directory.

```bash
#!/bin/bash -eux

cd
mkdir -p backend_src
rsync -av --delete --progress --exclude="/target" /SRC_DIR_LOCATION/ ~/backend_src

cd ~/backend_src
cargo build --bin dating_app_backend --release
sudo -u app mkdir -p /app-secure-storage/app/binaries
sudo -u app mkdir -p /app-secure-storage/app/backend-working-dir
sudo systemctl stop app-backend
sudo cp target/release/dating_app_backend /app-secure-storage/app/binaries
sudo chown app:app /app-secure-storage/app/binaries/dating_app_backend
sudo systemctl restart app-backend
sudo journalctl -u app-backend.service -b -e -f
```

Edit config file script:

```bash
#!/bin/bash -eux

sudo -u app vim /app-secure-storage/app/backend-working-dir/server_config.toml
```

# Litestream

Example config file:
```yml
dbs:
 - path: /app-secure-storage/app/backend-working-dir/database/current/current.db
   replicas:
     - type:    sftp
       host:    192.168.64.77:22
       user:    ubuntu
       path:    /home/ubuntu/litestream/current
       key-path: /app-secure-storage/app/.ssh/id_ed25519
```

# Diesel

Reset current database:

DATABASE_URL="database/current/current.db" diesel database reset

# Profiling build

cargo build --bin dating_app_backend --timings

https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/self-profile.html
Command for this is in Makefile.

https://github.com/rust-lang/measureme/blob/master/crox/README.md
Covert .mm_profdata to .json with
crox file.mm_profdata
Then open it in https://ui.perfetto.dev/

# Debugging with tokio-console

Only on debug builds.

The .cargo/config.toml has the required build flag.

```
cargo install --locked tokio-console
make
tokio-console
```

# Sign in with Google

If another Email is wanted to be visible in the Sign in with Google dialog, then
Google Cloud project needs to have another Google Account added with
permissions:

```
Access Context Manager Reader
OAuth Config Editor
Service Usage Viewer
```

## Contributions

Only bug fixes or documentation improvements are accepted at the moment.

Contributions must have the same license as the project (dual-licensed with
MIT and Apache 2.0).

## License

MIT License or Apache License 2.0
