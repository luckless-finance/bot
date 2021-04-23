.PHONY: help

help:
	@./utils/make-help.pl

clean:
	@echo "-------------------"
	@echo "delete generated artifacts"
	@echo "-------------------"
	rm -fr target

docs:
	@echo "-------------------"
	@echo "make and open docs"
	@echo "-------------------"
	cargo doc --open --no-deps

format:
	@echo "-------------------"
	@echo "format source code"
	@echo "-------------------"
	rustfmt src/*;
	rustfmt tests/*;

check:
	@echo "-------------------"
	@echo "check a local package and all of its dependencies for errors"
	@echo "-------------------"
	rustfmt --check --edition 2018 --quiet src/*;
	rustfmt --check --edition 2018 --quiet tests/*;
	cargo check;

fix:
	@echo "-------------------"
	@echo "fix imports and unused"
	@echo "-------------------"
	cargo fix;

test:
	@echo "-------------------"
	@echo "execute unit and integration tests"
	@echo "-------------------"
	cargo test;

pre-commit: check
	@echo "-------------------"
	@echo "pre-commit validation: test, format, check"
	@echo "-------------------"
	cargo test --quiet

watch-test:
	@echo "-------------------"
	@echo "run tests whenever files change (see .ignore)"
	@echo "-------------------"
	cargo watch -x test

release:
	@echo "-------------------"
	@echo "build release binary (see ./target/release)"
	@echo "-------------------"
	cargo build --release --all-targets

