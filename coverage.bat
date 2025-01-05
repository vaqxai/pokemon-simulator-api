@echo off
SET CARGO_INCREMENTAL=0
SET RUSTFLAGS=-Cinstrument-coverage
SET LLVM_PROFILE_FILE=cargo-test-%%p-%%m.profraw

cargo clean
cargo test

grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore "/*" -o ./coverage/