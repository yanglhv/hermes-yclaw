#!/bin/bash
# Smoke tests for the install.sh F1 dev shortcut.
#
# Run from a bash prompt:
#
#   bash scripts/tests/test-install-sh-stage-protocol.sh
#
# These tests only exercise the F1 dev shortcut (HERMES_INSTALL_USE_LOCAL_REPO)
# by sourcing the sync function from install.sh. They DO NOT run a full install.

set -e

# Locate install.sh relative to this test file
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_SCRIPT="$TEST_DIR/../install.sh"

if [ ! -f "$INSTALL_SCRIPT" ]; then
    echo "FAIL: Could not locate install.sh at $INSTALL_SCRIPT" >&2
    exit 1
fi

failures=0
assert_equal() {
    local expected="$1" actual="$2" label="$3"
    if [ "$expected" = "$actual" ]; then
        echo "OK: $label"
    else
        echo "FAIL: $label (expected: '$expected', got: '$actual')"
        failures=$((failures + 1))
    fi
}
assert_true() {
    local condition="$1" label="$2"
    # condition is a shell expression string — eval it
    if eval "$condition"; then
        echo "OK: $label"
    else
        echo "FAIL: $label"
        failures=$((failures + 1))
    fi
}
assert_path() {
    local path="$1" expected="$2" label="$3"
    if [ "$expected" = "exists" ] && [ -e "$path" ]; then
        echo "OK: $label"
    elif [ "$expected" = "absent" ] && [ ! -e "$path" ]; then
        echo "OK: $label"
    else
        echo "FAIL: $label (path '$path' expected $expected)"
        failures=$((failures + 1))
    fi
}

# -----------------------------------------------------------------------------
# Test: HERMES_INSTALL_USE_LOCAL_REPO bypasses git clone
# -----------------------------------------------------------------------------
echo
echo "-- HERMES_INSTALL_USE_LOCAL_REPO --"

tmp_root="$(mktemp -d -t hermes-f1-test-XXXXXX)"
trap "rm -rf '$tmp_root'" EXIT

source_dir="$tmp_root/source"
install_dir="$tmp_root/install"
mkdir -p "$source_dir" "$install_dir"

# Create a temp git repo (the local "source")
(cd "$source_dir" && \
    git init -q && \
    git config user.email "test@local" && \
    git config user.name "test" && \
    echo "hello from local" > marker.txt && \
    git add -A && \
    git commit -q -m "init")

# Extract the sync function definition from install.sh and source it.
# This avoids the argument-parsing bail that happens when install.sh is run
# without recognized args. The function is detected by its opening "^_sync...{"
# line and we read until the closing "^}" at column 0.
sync_fn_def="$(awk '/^_sync_local_repo_to_install_dir\(\) \{/,/^\}/' "$INSTALL_SCRIPT")"
if [ -z "$sync_fn_def" ]; then
    echo "FAIL: F1: _sync_local_repo_to_install_dir function not found in install.sh"
    failures=$((failures + 1))
else
    eval "$sync_fn_def"
    # Provide a fake log_warn / log_info / log_error / log_success so the
    # function can call them.
    log_warn() { echo "WARN: $1"; }
    log_error() { echo "ERROR: $1"; }
    log_success() { echo "OK: $1"; }
    log_info() { echo "INFO: $1"; }

    # Run the sync function
    _sync_local_repo_to_install_dir "$source_dir" "$install_dir"

    assert_path "$install_dir/marker.txt" "exists" "F1: marker file mirrored to install dir"
    if [ -e "$install_dir/marker.txt" ]; then
        content="$(cat "$install_dir/marker.txt")"
        assert_equal "hello from local" "$content" "F1: marker file content preserved"
    fi
    assert_path "$install_dir/.git" "exists" "F1: .git preserved in install dir"
    assert_path "$install_dir/venv" "absent" "F1: venv excluded from mirror"
fi

# -----------------------------------------------------------------------------
# Test: invalid path falls back (warning, not failure)
# -----------------------------------------------------------------------------
# The "falling back to git clone" warning is emitted by the F1 dev shortcut
# block at the top of clone_repo (not by the sync helper). Extract that block
# from install.sh, wrap it in a function, and run it against a bogus path to
# verify the warning is emitted and the function continues (does not error).
echo
echo "-- HERMES_INSTALL_USE_LOCAL_REPO invalid path --"

tmp_root2="$(mktemp -d -t hermes-f1-test2-XXXXXX)"
bogus="$tmp_root2/not-a-repo"
mkdir -p "$bogus"  # exists but no .git

install_dir2="$tmp_root2/install"
mkdir -p "$install_dir2"

# Extract the F1 dev shortcut block from clone_repo (between the two marker
# comments). We strip the marker comment lines themselves and dedent.
f1_block="$(awk '/F1 dev shortcut: HERMES_INSTALL_USE_LOCAL_REPO/,/end F1 dev shortcut/' "$INSTALL_SCRIPT")"
f1_code="$(echo "$f1_block" | grep -vE '^[[:space:]]*#' | sed 's/^[[:space:]]*//')"

if [ -z "$f1_code" ]; then
    echo "FAIL: F1: F1 dev shortcut block not found in install.sh clone_repo"
    failures=$((failures + 1))
else
    output2="$(
        eval "$sync_fn_def"
        log_warn() { echo "WARN: $1"; }
        log_error() { echo "ERROR: $1"; }
        log_success() { echo "OK: $1"; }
        log_info() { echo "INFO: $1"; }

        # Wrap the extracted F1 block in a function so `local` and `return`
        # work as they do inside clone_repo.
        eval "_f1_shortcut() { $(printf '%s' "$f1_code"); }"

        # Drive the block with a bogus path and a fresh install dir.
        HERMES_INSTALL_USE_LOCAL_REPO="$bogus"
        INSTALL_DIR="$install_dir2"
        # Return code is expected to be 0 (the block does not error on invalid
        # path — it warns and falls through to the normal git clone path).
        _f1_shortcut 2>&1
        echo "F1_SHORTCUT_RC=$?"
    )"

    echo "$output2" | grep -q "falling back to git clone" \
        && echo "OK: F1: invalid path emits 'falling back to git clone' warning" \
        || { echo "FAIL: F1: invalid path missing warning (output: $output2)"; failures=$((failures + 1)); }

    # The block must not abort on invalid path — the clone_repo caller still
    # needs to fall through to its normal git clone path. Confirm we got a
    # clean 0 return (the F1 block's `return` runs only on the success path;
    # on the fallback path the `else` branch runs and the block falls off the
    # end, which returns the last command's status — here `log_warn` → 0).
    echo "$output2" | grep -q "F1_SHORTCUT_RC=0" \
        && echo "OK: F1: invalid path does not error (block falls through)" \
        || { echo "FAIL: F1: invalid path returned non-zero (output: $output2)"; failures=$((failures + 1)); }
fi

rm -rf "$tmp_root2"

# -----------------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------------
echo
if [ "$failures" -gt 0 ]; then
    echo "FAILED: $failures test(s) failed"
    exit 1
else
    echo "ALL OK"
fi
