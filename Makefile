
.PHONY: test

debug:
	GITUI_LOGGING=true cargo run --features=timing

test:
	cargo test --workspace

clippy:
	cargo clean
	cargo clippy --all-features

clippy-pedantic:
	cargo clean
	cargo clippy --all-features -- -W clippy::pedantic

install:
	cargo install --path "."

install-debug:
	cargo install --features=timing --path "."