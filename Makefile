
CARGO_CRATE_ARGS = 	-p config \
					-p database \
					-p database_account \
					-p database_profile \
					-p database_media \
					-p database_chat \
					-p model \
					-p model_server_data \
					-p model_server_state \
					-p model_account \
					-p model_profile \
					-p model_media \
					-p model_chat \
					-p server \
					-p server_state \
					-p server_router_account \
					-p server_router_profile \
					-p server_router_media \
					-p server_router_chat \
					-p server_common \
					-p server_api \
					-p server_api_account \
					-p server_api_profile \
					-p server_api_media \
					-p server_api_chat \
					-p server_data \
					-p server_data_all \
					-p server_data_account \
					-p server_data_profile \
					-p server_data_media \
					-p server_data_chat \
					-p test_mode \
					-p test_mode_bot \
					-p test_mode_bot_runner \
					-p test_mode_tests \
					-p test_mode_tests_account \
					-p test_mode_tests_profile \
					-p test_mode_tests_media \
					-p test_mode_tests_runner \
					-p test_mode_utils \
					-p test_mode_macro \
					-p utils \
					-p simple_backend \
					-p simple_backend_utils \
					-p simple_backend_model \
					-p simple_backend_config \
					-p simple_backend_database \
					-p simple_backend_image_process \
					-p manager \
					-p manager_api \
					-p manager_config \
					-p manager_model \
					-p afrodite-backend

ifdef CONTINUE_FROM
TEST_QA_ARGS = --continue-from $(CONTINUE_FROM)
endif

TMP_FILE = ./target/tmp_file_for_makefile

# Default rule
run:
	RUST_LOG=$${RUST_LOG:-info} cargo run --bin afrodite-backend

run-release:
	RUST_LOG=$${RUST_LOG:-info} cargo run --bin afrodite-backend --release

fmt:
	cargo +nightly fmt $(CARGO_CRATE_ARGS)
check-fmt:
	cargo +nightly fmt --quiet --check $(CARGO_CRATE_ARGS)
fix:
	cargo fix ${CARGO_CRATE_ARGS}
test:
	RUST_LOG=info cargo run --bin afrodite-backend -- --sqlite-in-ram test ${TEST_ARGS} qa ${TEST_QA_ARGS}
unit-test:
	mkdir -p database/sqlite/current
	DATABASE_URL="sqlite:database/sqlite/current/current.db" cargo test

update-api-bindings-step-update-binary:
	cargo build --bin afrodite-backend
update-api-bindings-step-generate-bindings:
	./target/debug/afrodite-backend open-api > $(TMP_FILE)
	openapi-generator-cli generate \
	-i $(TMP_FILE) \
	-g rust \
	-o crates/api_client \
	--package-name api_client \
	--global-property apiDocs=false,modelDocs=false
# Workarounds for generator bugs
# Command output is redirected as macOS sed doesn't support normal -i
	sed 's/software_options: SoftwareOptions/software_options: crate::models::SoftwareOptions/g' crates/api_client/src/apis/common_admin_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/common_admin_api.rs
	sed 's/queue: ModerationQueueType/queue: crate::models::ModerationQueueType/g' crates/api_client/src/apis/media_admin_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/media_admin_api.rs
	sed 's/content_type: MediaContentType/content_type: crate::models::MediaContentType/g' crates/api_client/src/apis/media_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/media_api.rs
	sed 's/models::models::UnixTime/models::UnixTime/g' crates/api_client/src/apis/common_admin_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/common_admin_api.rs
update-api-bindings: update-api-bindings-step-update-binary update-api-bindings-step-generate-bindings
	echo "API bindings updated"
update-api-bindings-with-existing-binary: update-api-bindings-step-generate-bindings
	echo "API bindings updated"
validate-openapi:
	cargo build --bin afrodite-backend
	./target/debug/afrodite-backend open-api > $(TMP_FILE)
	openapi-generator-cli validate \
	-i $(TMP_FILE)

migrations-run:
	DATABASE_URL="postgres://localhost/afrodite_dev" diesel migration run
	rustfmt +nightly crates/model/src/schema.rs
	sh tools/convert_migrations.sh
reset-database:
	DATABASE_URL="postgres://localhost/afrodite_dev" diesel database reset
	rustfmt +nightly crates/model/src/schema.rs
	sh tools/convert_migrations.sh

profile-build:
	RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zself-profile=target/profile-build cargo build --bin afrodite-backend

code-stats:
	@/bin/echo -n "Lines:"
	@find \
	crates/config \
	crates/database \
	crates/database_account \
	crates/database_profile \
	crates/database_media \
	crates/database_chat \
	crates/model \
	crates/model_server_data \
	crates/model_server_state \
	crates/model_account \
	crates/model_profile \
	crates/model_media \
	crates/model_chat \
	crates/server \
	crates/server_state \
	crates/server_router_account \
	crates/server_router_profile \
	crates/server_router_media \
	crates/server_router_chat \
	crates/server_api \
	crates/server_api_account \
	crates/server_api_profile \
	crates/server_api_media \
	crates/server_api_chat \
	crates/server_data \
	crates/server_data_all \
	crates/server_data_account \
	crates/server_data_profile \
	crates/server_data_media \
	crates/server_data_chat \
	crates/test_mode \
	crates/test_mode_macro \
	crates/utils \
	crates/simple_backend \
	crates/simple_backend_utils \
	crates/simple_backend_model \
	crates/simple_backend_config \
	crates/simple_backend_database \
	crates/simple_backend_image_process \
	crates/manager \
	crates/manager_api \
	crates/manager_config \
	crates/manager_model \
	crates/afrodite-backend \
	-name '*.rs' | xargs wc -l | tail -n 1
	@echo "\nCommits:   `git rev-list --count HEAD` total"
