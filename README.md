# pihka-backend
Dating app backend


## Running

```
RUST_LOG=debug cargo run
```

Initial build needs the database.
```
mkdir -p database/current
sqlx database setup
```
