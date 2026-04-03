#!/usr/bin/env bash
# Push main to the optional "testing" remote (e.g. a second GitHub account for CI / quota testing).
# Does not push tags (no release).
#
# Before running:
#   1. Add the remote once:  git remote add testing https://github.com/OWNER/winusb-switcher-tauri-lite.git
#   2. Authenticate as that account (HTTPS + PAT, or SSH + ~/.ssh/config host).
#
# Usage:
#   ./scripts/push-testing-remote.sh
#   TESTING_REMOTE=myfork ./scripts/push-testing-remote.sh

set -euo pipefail

REMOTE="${TESTING_REMOTE:-testing}"
BRANCH="${TESTING_BRANCH:-main}"

if ! git remote get-url "$REMOTE" &>/dev/null; then
  echo "Remote '$REMOTE' is not configured. Example:" >&2
  echo "  git remote add testing https://github.com/ntgiahuy25d/winusb-switcher-tauri-lite.git" >&2
  exit 1
fi

echo "Remote: $REMOTE ($(git remote get-url "$REMOTE"))"
echo "Branch: $BRANCH"
echo "Pushing (no tags)..."

git push -u "$REMOTE" "$BRANCH"

# Ensure LFS objects exist on the new remote when this repo uses Git LFS for jlink-bundles.
if command -v git-lfs >/dev/null 2>&1 || git lfs version >/dev/null 2>&1; then
  if git lfs ls-files >/dev/null 2>&1 && [ -n "$(git lfs ls-files 2>/dev/null)" ]; then
    echo "Uploading Git LFS objects to $REMOTE..."
    git lfs push "$REMOTE" "$BRANCH"
  fi
fi

echo "Done."
