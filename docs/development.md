

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
cargo build --bin afrodite_backend --release
sudo -u app mkdir -p /app-secure-storage/app/binaries
sudo -u app mkdir -p /app-secure-storage/app/backend-working-dir
sudo systemctl stop app-backend
sudo cp target/release/afrodite_backend /app-secure-storage/app/binaries
sudo chown app:app /app-secure-storage/app/binaries/afrodite_backend
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

# Profiling build

cargo build --bin afrodite_backend --timings

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
