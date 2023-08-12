
CARGO_CRATE_ARGS = 	-p api_internal \
					-p config \
					-p database \
					-p model \
					-p server \
					-p test_mode \
					-p utils \
					-p pihka-backend

fmt:
	cargo +nightly fmt $(CARGO_CRATE_ARGS)
fix:
	cargo fix ${CARGO_CRATE_ARGS}
test:
	RUST_LOG=info cargo run --bin pihka-backend -- --sqlite-in-ram test
unit-test:
	DATABASE_URL="sqlite:database/current/current.db" cargo test
run:
	RUST_LOG=debug cargo run --bin pihka-backend

update-manager-submodule:
	git submodule update --remote --merge

migrations-run:
	DATABASE_URL="database/current/current.db" diesel migration run
reset-database:
	DATABASE_URL="database/current/current.db" diesel database reset

profile-build:
	RUSTFLAGS=-Zself-profile=target/profile-build cargo +nightly build --bin pihka-backend
