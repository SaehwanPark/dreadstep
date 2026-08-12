#!/usr/bin/env bash

set -euo pipefail

readonly EXPECTED_ARCHIVE_SHA256="c109438ab06f65fd80f9b2686a4cf9c7c11dc64444b47333ec71d602f8bb5fc7"
readonly DEFAULT_ARCHIVE="art/kenney-tiny-dungeon.zip"
readonly DEFAULT_DESTINATION="assets/dreadstep"
readonly PNG_SIGNATURE="89504e470d0a1a0a"

usage() {
  cat <<'EOF'
Usage:
  scripts/prepare-local-assets.sh --check [archive]
  scripts/prepare-local-assets.sh --install [archive] [destination]

Validate or install the selected CC0 Kenney Tiny Dungeon source tiles. The archive and generated
PNG files remain under ignored local-media directories; no production binary is added to Git.
EOF
}

sha256() {
  shasum -a 256 "$1" | awk '{print $1}'
}

validate_archive() {
  local archive="$1"
  local actual_sha256

  if [[ ! -f "$archive" ]]; then
    echo "local asset archive is missing: $archive" >&2
    echo "Obtain the recorded Kenney Tiny Dungeon CC0 archive before retrying." >&2
    return 2
  fi
  actual_sha256="$(sha256 "$archive")"
  if [[ "$actual_sha256" != "$EXPECTED_ARCHIVE_SHA256" ]]; then
    echo "unexpected local asset archive SHA-256: $actual_sha256" >&2
    echo "expected: $EXPECTED_ARCHIVE_SHA256" >&2
    return 3
  fi
}

readonly TILE_BINDINGS=(
  "terrain:0040"
  "player:0100"
  "enemy:0112"
  "dead:0124"
  "ground-item:0064"
  "inventory-item:0065"
)

check_sources() {
  local archive="$1"
  local binding
  local family
  local tile_id
  local member

  for binding in "${TILE_BINDINGS[@]}"; do
    family="${binding%%:*}"
    tile_id="${binding##*:}"
    member="Tiles/tile_${tile_id}.png"
    unzip -p "$archive" "$member" >/dev/null
    echo "validated $family <- $member"
  done
}

validate_destination() {
  local destination="$1"

  if [[ "$destination" == /* || "$destination" == *..* ]]; then
    echo "destination must be a relative ignored media path: $destination" >&2
    return 5
  fi
  case "$destination" in
    assets/* | art/* | audio/* | crates/*/assets/* | crates/*/art/* | crates/*/audio/*) ;;
    *)
      echo "destination must be under assets/, art/, audio/, or a crate-local media directory: $destination" >&2
      return 5
      ;;
  esac
}

install_assets() {
  local archive="$1"
  local destination="$2"
  local binding
  local family
  local tile_id
  local member
  local target
  local temporary
  local signature

  mkdir -p "$destination"
  for binding in "${TILE_BINDINGS[@]}"; do
    family="${binding%%:*}"
    tile_id="${binding##*:}"
    member="Tiles/tile_${tile_id}.png"
    target="$destination/$family.png"
    temporary="$(mktemp "${TMPDIR:-/tmp}/dreadstep-asset.XXXXXX")"
    unzip -p "$archive" "$member" >"$temporary"
    signature="$(od -An -tx1 -N8 "$temporary" | tr -d ' \n')"
    if [[ "$signature" != "$PNG_SIGNATURE" ]]; then
      rm -f "$temporary"
      echo "source member is not a PNG: $member" >&2
      return 4
    fi
    mv "$temporary" "$target"
    echo "installed $target"
  done
}

main() {
  local mode=""
  local archive="$DEFAULT_ARCHIVE"
  local destination="$DEFAULT_DESTINATION"

  if [[ "$#" -eq 0 ]]; then
    usage >&2
    return 64
  fi
  mode="$1"
  shift
  case "$mode" in
    --check)
      if [[ "$#" -gt 1 ]]; then
        usage >&2
        return 64
      fi
      archive="${1:-$archive}"
      validate_archive "$archive"
      check_sources "$archive"
      ;;
    --install)
      if [[ "$#" -gt 2 ]]; then
        usage >&2
        return 64
      fi
      archive="${1:-$archive}"
      destination="${2:-$destination}"
      validate_archive "$archive"
      check_sources "$archive"
      validate_destination "$destination"
      install_assets "$archive" "$destination"
      ;;
    *)
      usage >&2
      return 64
      ;;
  esac
}

main "$@"
