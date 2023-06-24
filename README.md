# pihka-backend
Dating app backend


## Running

Initial build needs the database.
```
mkdir -p database/current
sqlx database setup
```

```
RUST_LOG=debug cargo run
```

Add `debug = true` to config file and restart server.

<http://localhost:3000/swagger-ui/>

### Ubuntu 20.04

```
sudo apt install libssl-dev
cargo install sqlx-cli
```

### MacOS

Install OpenSSL <https://docs.rs/openssl/latest/openssl/>
```
brew install openssl@1.1
```

```
cargo install sqlx-cli
```


## Update server API bindings

1. Install node version manager (nvm) <https://github.com/nvm-sh/nvm>
2. Install latest node LTS with nvm. For example `nvm install 18`
3. Install openapi-generator from npm.
   `npm install @openapitools/openapi-generator-cli -g`
4. Start pihka backend in debug mode.
5. Generate bindings
```
openapi-generator-cli generate -i http://localhost:3000/api-doc/pihka_api.json -g rust -o api_client --package-name api_client
```

## Reset database

```
sqlx database drop && sqlx database create && sqlx migrate run
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