#!/bin/bash

export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="cargo-tests-%p-%m.profraw"
cargo test --workspace
mkdir -p target/coverage
grcov . --binary-path ./target/debug/deps/ \
--source-dir . \
--output-types lcov,html,markdown,cobertura \
--branch \
--ignore-not-existing \
--ignore **/tests.rs  \
--ignore "/*" \
--output-path target/coverage
find . -name '*.profraw' -type f -delete
