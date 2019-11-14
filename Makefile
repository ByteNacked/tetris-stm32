export ELLOCOPO_SCHEME_PATH=$(CURDIR)/scheme.rsl

dev:
	cargo build

check c:
	cargo check

release r:
	cargo build --release