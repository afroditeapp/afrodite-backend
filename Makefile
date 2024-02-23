
CARGO_CRATE_ARGS = 	-p api_internal \
					-p config \
					-p database \
					-p model \
					-p server \
					-p test_mode \
					-p test_mode_macro \
					-p utils \
					-p simple_backend \
					-p simple_backend_utils \
					-p simple_backend_model \
					-p simple_backend_config \
					-p simple_backend_database \
					-p simple_backend_image_process \
					-p pihka-backend

ifdef CONTINUE_FROM
TEST_QA_ARGS = --continue-from $(CONTINUE_FROM)
endif

# Default rule
run:
	RUST_LOG=$${RUST_LOG:-info} cargo run --bin pihka-backend

run-release:
	RUST_LOG=$${RUST_LOG:-info} cargo run --bin pihka-backend --release

fmt:
	cargo +nightly fmt $(CARGO_CRATE_ARGS)
fix:
	cargo fix ${CARGO_CRATE_ARGS}
test:
	RUST_LOG=info cargo run --bin pihka-backend -- --sqlite-in-ram test ${TEST_ARGS} qa ${TEST_QA_ARGS}
unit-test:
	mkdir -p database/sqlite/current
	DATABASE_URL="sqlite:database/sqlite/current/current.db" cargo test

update-manager-submodule:
	git submodule update --remote --merge
update-api-bindings:
	openapi-generator-cli generate \
	-i http://localhost:3000/api-doc/pihka_api.json \
	-g rust \
	-o crates/api_client \
	--package-name api_client
validate-openapi:
	openapi-generator-cli validate \
	-i http://localhost:3000/api-doc/pihka_api.json

migrations-run:
	mkdir -p database/sqlite/current
	DATABASE_URL="database/sqlite/current/current.db" diesel migration run
reset-database:
	mkdir -p database/sqlite/current
	DATABASE_URL="database/sqlite/current/current.db" diesel database reset

profile-build:
	RUSTFLAGS=-Zself-profile=target/profile-build cargo +nightly build --bin pihka-backend

code-stats:
	@/bin/echo -n "Lines:"
	@find \
	crates/api_internal \
	crates/config \
	crates/database \
	crates/model \
	crates/server \
	crates/test_mode \
	crates/test_mode_macro \
	crates/utils \
	crates/simple_backend \
	crates/simple_backend_utils \
	crates/simple_backend_model \
	crates/simple_backend_config \
	crates/simple_backend_database \
	crates/simple_backend_image_process \
	crates/pihka-backend \
	-name '*.rs' | xargs wc -l | tail -n 1
	@echo "\nCommits:   `git rev-list --count HEAD` total"
