#!/usr/bin/env bash

set -euo pipefail

readonly REQUIRED_FILES=(
  ".agents/skills/develop-dreadstep/SKILL.md"
  ".agents/skills/develop-dreadstep/agents/openai.yaml"
  ".agents/skills/review-dreadstep/SKILL.md"
  ".agents/skills/review-dreadstep/agents/openai.yaml"
  ".agents/skills/test-player/SKILL.md"
  ".agents/skills/test-player/agents/openai.yaml"
  ".editorconfig"
  "AGENTS.md"
  "ARCHITECTURE.md"
  "CHANGELOG.md"
  "CONTRIBUTING.md"
  "docs/demo.md"
  "LESSONS.md"
  "README.md"
  "SPEC.md"
  "crates/dreadstep-bevy/Cargo.toml"
  "crates/dreadstep-bevy/src/lib.rs"
  "crates/dreadstep-content/Cargo.toml"
  "crates/dreadstep-content/src/lib.rs"
  "crates/dreadstep-core/Cargo.toml"
  "crates/dreadstep-core/src/lib.rs"
  "crates/dreadstep-headless/Cargo.toml"
  "crates/dreadstep-headless/src/lib.rs"
  "crates/dreadstep-mcp/Cargo.toml"
  "crates/dreadstep-mcp/src/lib.rs"
  "crates/dreadstep-protocol/Cargo.toml"
  "crates/dreadstep-protocol/src/lib.rs"
  "docs/adr/0001-functional-core-and-adapters.md"
  "docs/harness/dreadstep/evals/cases.json"
  "docs/harness/dreadstep/team-spec.md"
  "rust-toolchain.toml"
  "rustfmt.toml"
  "scripts/prepare-local-assets.sh"
)

for path in "${REQUIRED_FILES[@]}"; do
  if [[ ! -f "${path}" ]]; then
    echo "missing required repository file: ${path}" >&2
    exit 1
  fi
done

if [[ -e "src/main.rs" ]]; then
  echo "the workspace root must remain a virtual manifest without a root binary" >&2
  exit 1
fi

if git grep --untracked -n $'\t' -- \
  '*.json' '*.md' '*.rs' '*.sh' '*.toml' '*.yaml' '*.yml'; then
  echo "tab characters are not allowed in human-authored files" >&2
  exit 1
fi

grep -Fxq 'hard_tabs = false' rustfmt.toml
grep -Fxq 'tab_spaces = 2' rustfmt.toml
grep -Fxq 'indent_size = 2' .editorconfig
grep -Fxq 'tab_width = 2' .editorconfig

check_skill() {
  local skill_name="$1"
  local skill_path=".agents/skills/${skill_name}"

  head -n 1 "${skill_path}/SKILL.md" | grep -Fxq -- '---'
  grep -Fxq "name: ${skill_name}" "${skill_path}/SKILL.md"
  grep -Eq '^description: .+' "${skill_path}/SKILL.md"
  grep -Fq "\$${skill_name}" "${skill_path}/agents/openai.yaml"
}

check_skill "develop-dreadstep"
check_skill "review-dreadstep"
check_skill "test-player"

check_local_media_policy() {
  local ignored_path
  local tracked_path

  for ignored_path in \
    "assets/example.aiff" \
    "art/example.psd" \
    "audio/example.caf" \
    "crates/dreadstep-bevy/assets/example.unknown"; do
    if ! git check-ignore --no-index -q -- "${ignored_path}"; then
      echo "presentation binary should be ignored: ${ignored_path}" >&2
      exit 1
    fi
  done

  for tracked_path in \
    "dreadstep-concept-art.png" \
    "screenshots/future.png" \
    "crates/dreadstep-bevy/src/audio/mod.rs" \
    "docs/audio/licensing.md" \
    "LICENSES/CC-BY-4.0.txt"; do
    if git check-ignore --no-index -q -- "${tracked_path}"; then
      echo "tracked exception or source/documentation path is unexpectedly ignored: ${tracked_path}" >&2
      exit 1
    fi
  done

  git ls-files --error-unmatch -- "dreadstep-concept-art.png" >/dev/null
}

check_local_media_policy

# Production sources must stay reviewable. desktop/tests.rs is excluded because it is a
# cfg(test) characterization suite, not a production module.
check_source_line_budget() {
  local path
  local lines
  local budget=800
  while IFS= read -r path; do
    case "${path}" in
      *tests.rs) continue ;;
    esac
    lines="$(wc -l < "${path}" | tr -d ' ')"
    if (( lines > budget )); then
      echo "source file exceeds ${budget}-line budget (${lines} lines): ${path}" >&2
      exit 1
    fi
  done < <(find crates -path '*/src/*.rs' -type f | sort)
}

check_source_line_budget

check_forbidden_dependency() {
  local package="$1"
  local tree

  tree="$(cargo tree --locked -p "${package}" --edges normal,build)"
  if grep -Eq '(^| )(bevy|rmcp) v' <<<"${tree}"; then
    echo "${package} must not depend on Bevy or rmcp" >&2
    exit 1
  fi
}

check_forbidden_dependency "dreadstep-core"
check_forbidden_dependency "dreadstep-protocol"
check_forbidden_dependency "dreadstep-content"

if cargo tree --locked -p dreadstep-bevy -e features | grep -Eq \
  'bevy feature "(audio|default_platform|wayland)"'; then
  echo "dreadstep-bevy enables forbidden desktop/audio features" >&2
  exit 1
fi

if ! cargo tree --locked -p dreadstep-bevy --features desktop -e normal,build \
  | grep -Fq "bevy_winit v"; then
  echo "dreadstep-bevy desktop feature must include the winit backend" >&2
  exit 1
fi

echo "repository structure checks passed"
