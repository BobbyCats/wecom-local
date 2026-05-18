#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Review a GitHub pull request for wecom-local without merging it.

Usage:
  scripts/review-pr.sh <PR_NUMBER>

The script prints PR metadata, checks commit-author hygiene, creates a temporary
worktree for validation, runs local gates, scans tracked files for private data
risk, and exits after printing one conclusion:

  SAFE_TO_MERGE
  NEEDS_CLEAN_LAND
  REQUEST_CHANGES
  BLOCKED

It never merges, closes, or comments on the pull request.
USAGE
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

if [ "$#" -ne 1 ]; then
  usage >&2
  exit 2
fi

pr_number="$1"
if ! [[ "$pr_number" =~ ^[0-9]+$ ]]; then
  echo "PR number must be numeric: $pr_number" >&2
  exit 2
fi

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
    exit 127
  fi
}

require_cmd git
require_cmd gh
require_cmd jq
require_cmd cargo

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
echo "Reviewing PR #$pr_number in $repo"
echo "This script does not merge, close, comment on, or approve PRs."
echo

json_fields=number,title,state,isDraft,baseRefName,headRefName,headRefOid,baseRefOid,mergeable,statusCheckRollup,commits,files,url
if ! pr_json="$(gh pr view "$pr_number" --json "$json_fields")"; then
  echo "CONCLUSION: BLOCKED"
  echo "Reason: failed to read PR metadata with gh."
  exit 1
fi

title="$(jq -r '.title' <<<"$pr_json")"
state="$(jq -r '.state' <<<"$pr_json")"
draft="$(jq -r '.isDraft' <<<"$pr_json")"
base="$(jq -r '.baseRefName' <<<"$pr_json")"
head="$(jq -r '.headRefName' <<<"$pr_json")"
head_oid="$(jq -r '.headRefOid' <<<"$pr_json")"
base_oid="$(jq -r '.baseRefOid' <<<"$pr_json")"
mergeable="$(jq -r '.mergeable' <<<"$pr_json")"
url="$(jq -r '.url' <<<"$pr_json")"

echo "== PR Metadata =="
echo "URL: $url"
echo "Title: $title"
echo "State: $state"
echo "Draft: $draft"
echo "Base: $base ($base_oid)"
echo "Head: $head ($head_oid)"
echo "Mergeable: $mergeable"
echo

echo "== Status Checks =="
if [ "$(jq '.statusCheckRollup | length' <<<"$pr_json")" -eq 0 ]; then
  echo " - no status checks reported"
else
  jq -r '
    .statusCheckRollup[]
    | " - \((.workflowName // "check")) / \((.name // "check")): \(.status)\(if .conclusion then " / " + .conclusion else "" end)"
  ' <<<"$pr_json"
fi
echo

echo "== Files =="
jq -r '
  .files[]
  | " - \(.path) (\(.changeType), +\(.additions)/-\(.deletions))"
' <<<"$pr_json"
echo

echo "== Commits =="
jq -r '
  .commits[]
  | " - \(.oid[0:7]) \(.messageHeadline)\n   authors: \([.authors[]? | "\(.name) <\(.email)>"] | join(", "))"
' <<<"$pr_json"
echo

failures=()
blocked=()

author_lines="$(jq -r '.commits[] | .oid as $oid | .authors[]? | "\($oid[0:7]) \(.name) <\(.email)>"' <<<"$pr_json")"
author_risk_regex='(\.local|localhost|/Users|MacBook|MacBook-Air|MacBook-Pro|Mac-mini|Mac Studio|iMac)'
author_findings="$(grep -Eai "$author_risk_regex" <<<"$author_lines" || true)"
host_name="$(hostname -s 2>/dev/null || true)"
if [ -n "$host_name" ]; then
  host_findings="$(grep -Fai "$host_name" <<<"$author_lines" || true)"
  if [ -n "$host_findings" ]; then
    author_findings="$author_findings"$'\n'"$host_findings"
  fi
fi
author_findings="$(sed '/^[[:space:]]*$/d' <<<"$author_findings" | sort -u || true)"

echo "== Commit Author Hygiene =="
if [ -n "$author_findings" ]; then
  echo "$author_findings" | sed 's/^/ - risky author metadata: /'
else
  echo " - ok"
fi
echo

tmp_parent="$(mktemp -d "${TMPDIR:-/tmp}/wecom-local-pr-review.XXXXXX")"
worktree="$tmp_parent/worktree"
tmp_ref="refs/review-pr/$pr_number-$$"
smoke_codex_home="$tmp_parent/codex-home"

cleanup() {
  cd "$repo_root" >/dev/null 2>&1 || true
  git worktree remove --force "$worktree" >/dev/null 2>&1 || true
  git update-ref -d "$tmp_ref" >/dev/null 2>&1 || true
  rm -rf "$tmp_parent"
}
trap cleanup EXIT

echo "== Temporary Worktree =="
if ! git fetch --quiet origin \
  "+refs/heads/$base:refs/remotes/origin/$base" \
  "+refs/pull/$pr_number/head:$tmp_ref"; then
  echo " - failed to fetch base or PR head"
  blocked+=("failed to fetch PR head")
else
  echo " - fetched PR head into $tmp_ref"
fi

if [ "${#blocked[@]}" -eq 0 ]; then
  if ! git worktree add --detach "$worktree" "$tmp_ref" >/dev/null; then
    echo " - failed to create temporary worktree"
    blocked+=("failed to create temporary worktree")
  else
    echo " - created $worktree"
  fi
fi
echo

run_check() {
  local label="$1"
  shift
  echo "==> $label"
  if "$@"; then
    echo "PASS: $label"
  else
    echo "FAIL: $label"
    failures+=("$label")
  fi
  echo
}

scan_fixed() {
  local label="$1"
  local pattern="$2"
  local matches

  if [ -z "$pattern" ]; then
    return
  fi

  matches="$(git grep -nI -F "$pattern" -- . ':!Cargo.lock' || true)"
  if [ -n "$matches" ]; then
    echo "Sensitive scan hit: $label"
    echo "$matches" | sed 's/^/  /'
    failures+=("sensitive text scan: $label")
  fi
}

scan_regex() {
  local label="$1"
  local pattern="$2"
  local matches

  matches="$(git grep -nI -E "$pattern" -- . ':!Cargo.lock' || true)"
  if [ -n "$matches" ]; then
    echo "Sensitive scan hit: $label"
    echo "$matches" | sed 's/^/  /'
    failures+=("sensitive text scan: $label")
  fi
}

if [ "${#blocked[@]}" -eq 0 ]; then
  cd "$worktree"

  run_check "git diff --check" git diff --check "origin/$base...HEAD"
  run_check "cargo fmt --check" cargo fmt --check
  run_check "cargo clippy --all-targets -- -D warnings" cargo clippy --all-targets -- -D warnings
  run_check "cargo test" cargo test
  run_check "cargo build" cargo build
  run_check "cargo package --list" cargo package --list

  changed_files="$(git diff --name-only "origin/$base...HEAD")"
  if grep -Fxq "scripts/install-codex-skills.sh" <<<"$changed_files"; then
    run_check "bash -n scripts/install-codex-skills.sh" bash -n scripts/install-codex-skills.sh
    run_check "Codex skill install smoke" env CODEX_HOME="$smoke_codex_home" bash scripts/install-codex-skills.sh
    run_check "Codex skill repeated install skip smoke" env CODEX_HOME="$smoke_codex_home" bash scripts/install-codex-skills.sh
    run_check "Codex skill --force replace smoke" env CODEX_HOME="$smoke_codex_home" bash scripts/install-codex-skills.sh --force
  fi

  echo "== Tracked File Risk Scan =="
  tracked_findings="$(
    {
      git ls-files | grep -E '(^|/)PROJECT_CONTEXT\.md$|(^|/)target/' || true
      git ls-files | grep -Ei '\.(db|sqlite|sqlite3|key|pem|dump|png|jpe?g|mp4|zip)$' || true
    } | sort -u
  )"
  if [ -n "$tracked_findings" ]; then
    echo "$tracked_findings" | sed 's/^/ - risky tracked file: /'
    failures+=("tracked file risk scan")
  else
    echo " - ok"
  fi
  echo

  echo "== Sensitive Text Scan =="
  user_name="${SUDO_USER:-${USER:-}}"
  scan_fixed "local user path" "/Users/$user_name"
  scan_fixed "private session marker" "019e""375f"
  private_group_marker="$(printf '\350\247\206\351\242\221\347\240\224\347\251\266\345\260\217\347\273\204')"
  scan_fixed "private group name marker" "$private_group_marker"
  scan_fixed "private key marker" "PRIVATE"" KEY"
  scan_fixed "OpenSSH key marker" "BEGIN"" OPENSSH"
  scan_regex "password-like assignment" '(^|[^A-Za-z0-9_])(MACOS_|SUDO_)?(PASSWORD|PASSWD|PWD)[[:space:]]*='
  scan_regex "Chinese password assignment" '密码[[:space:]]*[:=]'
  if [ "${#failures[@]}" -eq 0 ]; then
    echo " - ok"
  fi
  echo
fi

echo "== Conclusion =="
if [ "${#blocked[@]}" -gt 0 ]; then
  echo "CONCLUSION: BLOCKED"
  printf ' - %s\n' "${blocked[@]}"
  exit 1
fi

if [ "${#failures[@]}" -gt 0 ]; then
  echo "CONCLUSION: REQUEST_CHANGES"
  printf ' - %s\n' "${failures[@]}"
elif [ -n "$author_findings" ]; then
  echo "CONCLUSION: NEEDS_CLEAN_LAND"
  echo " - PR diff may be usable, but commit metadata should not enter public history."
elif [ "$draft" = "true" ]; then
  echo "CONCLUSION: BLOCKED"
  echo " - PR is still marked draft; do not merge until it is ready for review."
elif [ "$state" != "OPEN" ]; then
  echo "CONCLUSION: BLOCKED"
  echo " - PR state is $state."
else
  echo "CONCLUSION: SAFE_TO_MERGE"
fi
