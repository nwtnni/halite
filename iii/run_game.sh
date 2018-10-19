#!/bin/bash

set -e

cargo build
./game/halite --replay-directory replays/ -vvv --width 32 --height 32 "./target/debug/MyBot" "./target/debug/MyBot"
