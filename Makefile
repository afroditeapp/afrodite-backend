
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
