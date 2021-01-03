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
	rustfmt src/**/*;

check:
	@echo "-------------------"
	@echo "check a local package and all of its dependencies for errors"
	@echo "-------------------"
	rustfmt --check --edition 2018 --quiet src/**/*;
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

pre-commit:
	@echo "-------------------"
	@echo "pre-commit validation: test, format, check"
	@echo "-------------------"
	make test && rustfmt src/**/* --check && make check

watch-test:
	@echo "-------------------"
	@echo "run tests whenever files change (see .ignore)"
	@echo "-------------------"
	cargo watch -x test

