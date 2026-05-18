#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Install wecom-local Codex skills into $CODEX_HOME/skills.

Usage:
  scripts/install-codex-skills.sh [--force]

Options:
  --force   Replace existing wc-* skill directories.
  -h, --help
            Show this help.

CODEX_HOME defaults to ~/.codex.
USAGE
}

force=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --force)
      force=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
codex_home="${CODEX_HOME:-$HOME/.codex}"
skills_dir="$codex_home/skills"

mkdir -p "$skills_dir"

install_skill() {
  local source_dir="$1"
  local name="$2"
  local dest_dir="$skills_dir/$name"

  if [ ! -f "$source_dir/SKILL.md" ]; then
    echo "missing SKILL.md in $source_dir" >&2
    exit 1
  fi

  if [ -e "$dest_dir" ]; then
    if [ "$force" -eq 0 ]; then
      echo "skip $name: $dest_dir already exists (use --force to replace)"
      return
    fi
    rm -rf "$dest_dir"
  fi

  mkdir -p "$dest_dir"
  cp -R "$source_dir/." "$dest_dir/"
  echo "installed $name -> $dest_dir"
}

install_skill "$repo_root/skills/codex" "wc-local"
install_skill "$repo_root/skills/wc-brief" "wc-brief"
install_skill "$repo_root/skills/wc-scan" "wc-scan"
install_skill "$repo_root/skills/wc-audit" "wc-audit"
install_skill "$repo_root/skills/wc-style" "wc-style"
install_skill "$repo_root/skills/wc-draft" "wc-draft"

echo
echo "Restart Codex to pick up newly installed skills."
