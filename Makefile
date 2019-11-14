export ELLOCOPO_SCHEME_PATH=$(CURDIR)/scheme.rsl

dev:
	cargo build

o:
	Ozone $(CURDIR)/ozone_dev.jdebug

or: 
	Ozone $(CURDIR)/ozone_rel.jdebug

check c:
	cargo check

release r:
	cargo build --release