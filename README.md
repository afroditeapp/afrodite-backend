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
