#!/bin/bash

set -e

cargo build
./game/halite --replay-directory replays/ -vvv --width 32 --height 32 "RUST_BACKTRACE=1 ./target/debug/MyBot" "RUST_BACKTRACE=1 ./target/debug/MyBot"
