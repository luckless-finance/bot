.PHONY: help

help:
	@./utils/make-help.pl

format:
	@echo "-------------------"
	@echo "format source code"
	@echo "-------------------"
	rustfmt src/*

check:
	@echo "-------------------"
	@echo "check a local package and all of its dependencies for errors"
	@echo "-------------------"
	rustfmt --check --edition 2018 --quiet src/*;
	cargo check

test:
	@echo "-------------------"
	@echo "execute unit and integration tests"
	@echo "-------------------"
	cargo test

watch-test:
	@echo "-------------------"
	@echo "start infinite test loop"
	@echo "-------------------"
	./utils/watch-test.sh

