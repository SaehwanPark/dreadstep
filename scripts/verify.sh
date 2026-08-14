#!/usr/bin/env bash

set -euo pipefail

scripts/check-repository.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo run -p dreadstep-tui --locked -- \
  --smoke --seed 7 --log-dir target/dreadstep-ci-logs

echo "all repository checks passed"
