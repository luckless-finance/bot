#!/usr/bin/env bash

function unit_test() {
  clear
  cargo check && cargo test
  printf "current time: %s$(date +%FT%H.%M.%S%Z)%s" "${GREEN}" "${RESET}";
}
unit_test
while sleep 3; do unit_test; done
