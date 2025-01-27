

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

# Litestream

Example config file:
```yml
dbs:
 - path: /afrodite-secure-storage/afrodite/backend/database/current/current.db
   replicas:
     - type:    sftp
       host:    192.168.64.77:22
       user:    ubuntu
       path:    /home/ubuntu/litestream/current
       key-path: /afrodite-secure-storage/afrodite/.ssh/id_ed25519
```

# Profiling build

cargo build --bin afrodite-backend --timings

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
