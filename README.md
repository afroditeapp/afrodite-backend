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

<http://localhost:3000/swagger-ui/>

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
