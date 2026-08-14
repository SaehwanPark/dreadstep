#!/usr/bin/env bash

set -euo pipefail

# Deferred visual-enhancement gate. Default verify.sh and CI use the terminal
# showcase instead. Keep this wrapper for the later pixel-2D stage.

cargo run -p dreadstep-bevy --features desktop --bin dreadstep --locked -- \
  --smoke --seed 7 --log-dir target/dreadstep-bevy-ci-logs
