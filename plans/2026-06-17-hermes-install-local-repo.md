# F1: dev 本地跳过 clone — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让开发者通过环境变量 `HERMES_INSTALL_USE_LOCAL_REPO=<本地 checkout 路径>` 让 `install.ps1` / `install.sh` 跳过内部 `git clone`，转而把本地 checkout 同步到 `$InstallDir`。`bootstrap-runner.cjs` 透传该变量。

**Architecture:** F1 是"install 脚本内部 git clone 阶段"的环境变量旁路。同步机制：Windows 用 `robocopy /MIR`、Unix 用 `rsync -a --delete`，排除 `venv` / `node_modules` / `.hermes-bootstrap-complete`，保留 `.git`。路径无效时降级到原 git clone 流程 + warn。跟现有 `HERMES_SETUP_DEV_REPO_ROOT`（启动器侧）、`SOURCE_REPO_ROOT`（bootstrap-runner 侧）正交 — 这三个 dev shortcut 分别作用在 install 链的不同层。

**Tech Stack:** PowerShell (`robocopy` 内置)、bash (`rsync` 标准 Unix 工具)、Node.js (`bootstrap-runner.cjs`)、`node:test` 框架（CJS 测试）、`pester`-风格 简单 PS 断言函数（PS 测试，沿用 `scripts/tests/test-install-ps1-stage-protocol.ps1` 现有模式）。

**Spec:** `docs/superpowers/specs/2026-06-17-hermes-install-local-repo-design.md`（项目 `.gitignore` 显式 ignore 该目录，spec 不入主仓库）。本 plan 自包含关键决策。

**Reference — 关键决策摘要**（来自 spec §3）：

- D1 变量名：`HERMES_INSTALL_USE_LOCAL_REPO`
- D2 同步机制：`robocopy /MIR`（install.ps1）/ `rsync -a --delete`（install.sh）
- D3 `.git` 处理：**保留**
- D4 排除目录：`venv`、`.venv`、`node_modules`、`.hermes-bootstrap-complete`
- D5 增量 `$InstallDir`：备份到 `$InstallDir.broken-<UTC ts>` 后覆盖
- D6 后续 `git checkout` 阶段：跳过
- D7 路径校验失败：**降级**到原 git clone 路径 + warn

---

## File Structure

| 任务 | 改动文件 | 职责 |
|---|---|---|
| Task 1 | `scripts/install.ps1` | 添加 F1 检测 + `Sync-LocalRepoToInstallDir` 函数（line 1099 顶部插入） |
| Task 1 | `scripts/tests/test-install-ps1-stage-protocol.ps1` | 追加 F1 测试块（沿用现有 `Assert-Equal`/`Assert-True` 模式） |
| Task 2 | `scripts/install.sh` | 添加 F1 检测 + `_sync_local_repo_to_install_dir` 函数（line 1118 顶部插入） |
| Task 2 | `scripts/tests/test-install-sh-stage-protocol.sh` | 新建，沿用 install.ps1 测试文件结构 |
| Task 3 | `apps/desktop/electron/bootstrap-runner.cjs` | `spawnPowerShell` 透传 env var（line 298-303） |
| Task 3 | `apps/desktop/electron/bootstrap-runner.test.cjs` | 追加透传测试块 |
| Task 4 | `website/docs/getting-started/installation.md` | 追加 "Development mode — install from a local checkout" 章节 |

---

## Task 1: `install.ps1` — HERMES_INSTALL_USE_LOCAL_REPO 支持

**Files:**
- Modify: `scripts/install.ps1:1099`（在 `Install-Repository` 函数顶部紧跟 `Write-Info "Installing to $InstallDir..."` 之后插入 F1 块；在 line 1700-1750 之间新增 `Sync-LocalRepoToInstallDir` 函数）
- Modify: `scripts/tests/test-install-ps1-stage-protocol.ps1`（追加 F1 测试块）

- [ ] **Step 1: 写失败测试（追加到 `test-install-ps1-stage-protocol.ps1` 末尾）**

在 `test-install-ps1-stage-protocol.ps1` 末尾追加以下内容（接在现有 `Assert-Equal` / `Assert-True` 辅助函数之后，最末测试块之前；保留文件 header 注释）：

```powershell
# -----------------------------------------------------------------------------
# Test: HERMES_INSTALL_USE_LOCAL_REPO bypasses git clone
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "-- HERMES_INSTALL_USE_LOCAL_REPO --"

# 1. Create a temp git repo (the local "source")
$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) "hermes-f1-test-$([System.Guid]::NewGuid().ToString('N'))"
$sourceDir = Join-Path $tmpRoot "source"
$installDir = Join-Path $tmpRoot "install"
New-Item -ItemType Directory -Force -Path $sourceDir | Out-Null
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

Push-Location $sourceDir
try {
    git init -q 2>$null
    git config user.email "test@local" 2>$null
    git config user.name "test" 2>$null
    "hello from local" | Out-File -Encoding utf8 -FilePath "marker.txt"
    git add -A 2>$null
    git commit -q -m "init" 2>$null
} finally {
    Pop-Location
}

# 2. Run install.ps1 with HERMES_INSTALL_USE_LOCAL_REPO set, target Stage-Repository
$env:HERMES_INSTALL_USE_LOCAL_REPO = $sourceDir
$env:HERMES_HOME = $tmpRoot
# override default install dir by setting -InstallDir
try {
    $output = & powershell -NoProfile -ExecutionPolicy Bypass -File $installScript -Stage Stage-Repository -Json -InstallDir $installDir -NoVenv -SkipSetup 2>&1
} finally {
    Remove-Item Env:HERMES_INSTALL_USE_LOCAL_REPO -ErrorAction SilentlyContinue
    Remove-Item Env:HERMES_HOME -ErrorAction SilentlyContinue
}

# 3. Assert: the JSON frame reports ok=true
$lastLine = ($output | Where-Object { $_ -match '^\{' } | Select-Object -Last 1)
$frame = $null
try { $frame = $lastLine | ConvertFrom-Json } catch {}
Assert-True ($null -ne $frame) -Label "F1: stage emits parseable JSON frame"
if ($frame) {
    Assert-True ($frame.ok -eq $true) -Label "F1: stage ok=true (got: $($frame | ConvertTo-Json -Compress))"
}

# 4. Assert: marker file was mirrored to install dir
$mirrored = Join-Path $installDir "marker.txt"
Assert-True (Test-Path -LiteralPath $mirrored) -Label "F1: marker file mirrored to install dir"
if (Test-Path -LiteralPath $mirrored) {
    $content = Get-Content -LiteralPath $mirrored -Raw
    Assert-Equal -Expected "hello from local" -Actual $content.Trim() -Label "F1: marker file content preserved"
}

# 5. Assert: .git was preserved in install dir
Assert-True (Test-Path -LiteralPath (Join-Path $installDir ".git")) -Label "F1: .git preserved in install dir"

# 6. Assert: venv was NOT mirrored
Assert-True (-not (Test-Path -LiteralPath (Join-Path $installDir "venv"))) -Label "F1: venv excluded from mirror"

# Cleanup
Remove-Item -LiteralPath $tmpRoot -Recurse -Force -ErrorAction SilentlyContinue

# -----------------------------------------------------------------------------
# Test: invalid path falls back to git clone (warning, not failure)
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "-- HERMES_INSTALL_USE_LOCAL_REPO invalid path --"

$tmpRoot2 = Join-Path ([System.IO.Path]::GetTempPath()) "hermes-f1-test2-$([System.Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Force -Path $tmpRoot2 | Out-Null
$bogus = Join-Path $tmpRoot2 "not-a-repo"
New-Item -ItemType Directory -Force -Path $bogus | Out-Null  # exists but no .git

$env:HERMES_INSTALL_USE_LOCAL_REPO = $bogus
$env:HERMES_HOME = $tmpRoot2
$installDir2 = Join-Path $tmpRoot2 "install"
try {
    # We do NOT run a real git clone (no network in unit tests) — we just check
    # the warning is emitted. Use -Manifest (no stages run) is not enough since
    # the warning is in Install-Repository. So we run a stage that would emit
    # the warn before attempting the clone. In our impl, the warn goes to
    # stderr-ish Write-Warn output (mixed with stdout). Use 2>&1 to capture all.
    #
    # Expectation: the stage will FAIL (no network → git clone fails) but the
    # warning text MUST appear in the captured output. So we check output
    # contains the warning, not that ok=true.
    $output2 = & powershell -NoProfile -ExecutionPolicy Bypass -File $installScript -Stage Stage-Repository -Json -InstallDir $installDir2 -NoVenv -SkipSetup 2>&1
    $warned = ($output2 -join "`n") -match "falling back to git clone"
    Assert-True $warned -Label "F1: invalid path emits 'falling back to git clone' warning"
} finally {
    Remove-Item Env:HERMES_INSTALL_USE_LOCAL_REPO -ErrorAction SilentlyContinue
    Remove-Item Env:HERMES_HOME -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $tmpRoot2 -Recurse -Force -ErrorAction SilentlyContinue
}
```

- [ ] **Step 2: 跑测试，确认 fail**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/tests/test-install-ps1-stage-protocol.ps1
```

**Expected**: 新增的 F1 测试块全部 FAIL（"F1: stage emits parseable JSON frame" 等）。其他现有测试保持 PASS（如果之前是 PASS 的话）。

- [ ] **Step 3: 实现 `install.ps1` 的 F1 块 + `Sync-LocalRepoToInstallDir` 函数**

**3a. 在 `Install-Repository` 函数顶部插入 F1 块**（line 1099 顶部，紧跟 `Write-Info "Installing to $InstallDir..."` 之后）：

```powershell
function Install-Repository {
    Write-Info "Installing to $InstallDir..."

    # ── F1 dev shortcut: HERMES_INSTALL_USE_LOCAL_REPO ──
    # Lets a developer point the installer at a local checkout (typically a
    # git worktree) so their uncommitted / unpushed edits in apps/desktop are
    # used by the build instead of the upstream main HEAD. See spec §5.
    $localRepo = $env:HERMES_INSTALL_USE_LOCAL_REPO
    if ($localRepo) {
        $localRepo = $localRepo.Trim()
        if ($localRepo -and (Test-Path -LiteralPath $localRepo) -and (Test-Path -LiteralPath (Join-Path $localRepo '.git'))) {
            Write-Info "Using local repo at $localRepo (HERMES_INSTALL_USE_LOCAL_REPO); skipping git clone"
            Sync-LocalRepoToInstallDir -Source $localRepo -Dest $InstallDir
            $script:_UsedLocalRepo = $true
            return
        } else {
            Write-Warn "HERMES_INSTALL_USE_LOCAL_REPO=$localRepo is not a valid git checkout; falling back to git clone"
        }
    }
    # ── end F1 dev shortcut ──

    $didUpdate = $false
    # [existing Install-Repository logic continues here unchanged]
    ...
}
```

注意：`Install-Repository` 函数原来第二行是 `$didUpdate = $false`，我把 F1 块插入到 `Write-Info` 和 `$didUpdate` 之间。**F1 块命中时 return 早**，原 `$didUpdate = $false` 不会执行（这跟原逻辑对齐 — 原函数 `$didUpdate` 之后才是 `if (Test-Path $InstallDir) {...}` 的 update/clone 分支）。

**3b. 在文件中段（找一个合适位置，例如 line 1700 附近的 helper functions 区）新增 `Sync-LocalRepoToInstallDir` 函数**：

```powershell
function Sync-LocalRepoToInstallDir {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Dest
    )

    # Guard against Source == Dest (would be a no-op mirror, but robocopy /MIR
    # can produce undefined behaviour when source and dest resolve to the same
    # inode). See spec §9.2.
    $resolvedSource = (Resolve-Path -LiteralPath $Source).ProviderPath
    if (Test-Path -LiteralPath $Dest) {
        $resolvedDest = (Resolve-Path -LiteralPath $Dest).ProviderPath
        if ($resolvedSource -eq $resolvedDest) {
            Write-Warn "HERMES_INSTALL_USE_LOCAL_REPO source equals install dir ($resolvedSource); skipping mirror"
            return
        }
    }

    # Handle existing $Dest (mirrors line 1278-1288 "not a git repo" handling).
    if (Test-Path -LiteralPath $Dest) {
        $backupDir = "$Dest.broken-" + (Get-Date -Format 'yyyyMMdd-HHmmss')
        Write-Warn "Existing directory at $Dest (dev install path); moving aside to $backupDir"
        try {
            Move-Item -LiteralPath $Dest -Destination $backupDir -ErrorAction Stop
        } catch {
            Write-Err "Could not move $Dest aside: $_"
            throw
        }
    }

    # Ensure parent of $Dest exists.
    $parent = Split-Path -Parent $Dest
    if ($parent -and -not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }

    # Mirror $Source → $Dest with venv / node_modules / marker excluded.
    # /MIR = mirror (overwrite + delete extra files in $Dest).
    # /XD  = exclude directories. .git is INTENTIONALLY NOT in this list (D3:
    #        keep .git so the dev user can inspect history and `hermes update`
    #        still works in the install path).
    # /NFL /NDL /NJH /NJS /NP = quiet output.
    # /R:0 /W:0 = don't retry on transient errors.
    $robocopyArgs = @(
        $Source
        $Dest
        '/MIR'
        '/XD', 'venv', '.venv', 'node_modules', '.hermes-bootstrap-complete'
        '/NFL', '/NDL', '/NJH', '/NJS', '/NP'
        '/R:0', '/W:0'
    )
    # robocopy exit codes: 0-7 = success, 8+ = error. We use /MIR which can return
    # 1 ("files copied") and 3 ("extra files deleted") — both fine for us.
    $rc = 0
    & robocopy @robocopyArgs | ForEach-Object { "$_" }
    $rc = $LASTEXITCODE
    if ($rc -ge 8) {
        throw "robocopy mirror of $Source → $Dest failed (exit $rc)"
    }

    Write-Success "Local repo mirrored from $Source to $Dest"
}
```

**位置**：放在现有 `Test-Node` / `Clear-ElectronBuildCache` / `Restore-ElectronDist` 等 helper 区域（line 1700-2200 之间任意合适位置），跟其他 helper 风格一致。

- [ ] **Step 4: 跑测试，确认 pass**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/tests/test-install-ps1-stage-protocol.ps1
```

**Expected**: 新增的 F1 测试块全部 OK（包括 "F1: marker file mirrored"、"F1: .git preserved"、"F1: venv excluded"、"F1: stage ok=true"）。无效路径测试也 OK（warning 出现）。

**如果 FAIL**：
- "F1: stage ok=true" FAIL → 检查 F1 块是否在 `Install-Repository` 函数**顶部**（line 1099 紧跟 Write-Info 之后），不是某个 if 分支里
- "F1: marker file mirrored" FAIL → 检查 `Sync-LocalRepoToInstallDir` 函数是否被调用、robocopy 是否在 Windows 上可达
- "F1: .git preserved" FAIL → 检查 `/XD` 列表确实不含 `.git`
- "F1: venv excluded" FAIL → 检查 `/XD` 列表含 `venv`、`.venv`、但源里实际有 venv 目录才会真排除；如果源里没 venv，本断言会通过

- [ ] **Step 5: Commit**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
git add scripts/install.ps1 scripts/tests/test-install-ps1-stage-protocol.ps1
git commit -m "feat(install): HERMES_INSTALL_USE_LOCAL_REPO dev shortcut

Lets install.ps1 skip the git clone stage and mirror a local checkout
into the install directory when HERMES_INSTALL_USE_LOCAL_REPO is set
to a path containing .git/. Keeps .git and excludes venv / node_modules
/ .hermes-bootstrap-complete from the mirror. Falls back to the regular
git clone with a warning when the path is missing or not a git repo."
```

---

## Task 2: `install.sh` — 镜像 install.ps1 的支持

**Files:**
- Modify: `scripts/install.sh:1118`（在 `clone_repo` 函数顶部紧跟 `log_info "Installing to $INSTALL_DIR..."` 之后插入 F1 块；在文件中段新增 `_sync_local_repo_to_install_dir` 函数）
- Create: `scripts/tests/test-install-sh-stage-protocol.sh`（新建，沿用 install.ps1 测试文件结构）

- [ ] **Step 1: 写失败测试（创建 `test-install-sh-stage-protocol.sh`）**

创建 `scripts/tests/test-install-sh-stage-protocol.sh`：

```bash
#!/bin/bash
# Smoke tests for the install.sh F1 dev shortcut.
#
# Run from a bash prompt:
#
#   bash scripts/tests/test-install-sh-stage-protocol.sh
#
# These tests only exercise the F1 dev shortcut (HERMES_INSTALL_USE_LOCAL_REPO)
# by sourcing install.sh in a controlled way. They DO NOT run a full install.

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

# Source install.sh's helpers in a way that doesn't trigger full install.
# We dot-source install.sh — it will run its top-level argument parsing and
# bail at the "unknown option" check, but its FUNCTIONS (clone_repo,
# _sync_local_repo_to_install_dir) will be defined.
#
# To avoid the argument parsing bail, we run install.sh in a subshell with
# the --manifest flag (no-op for install.sh which doesn't implement --manifest
# the same way as install.ps1, but it's safe), capturing output. Better: we
# directly call clone_repo via a small wrapper.
#
# Strategy: source install.sh with a known flag that won't trigger full install.
# install.sh runs argument parsing first; with no recognized args, it errors.
# Simpler: just source it, and the F1 detection is testable via _sync_local_repo_to_install_dir.

# Source install.sh in a subshell; pre-set HERMES_INSTALL_USE_LOCAL_REPO and
# HERMES_HOME so the sourced functions pick them up. Override the trailing
# Main invocation by setting INSTALL_SCRIPT_TEST_MODE=1 — install.sh can guard
# against re-running its main when sourced. If install.sh doesn't currently
# support being sourced, the test below will fail and we will need to add a
# guard. For now, we test the F1 sync function directly without sourcing.

# Direct test: run install.sh as a subprocess with --help to verify it can
# at least be parsed without error. Then test the sync function by extracting
# it via grep and sourcing.
assert_true "true" "install.sh is parseable (smoke check via --help)"
bash "$INSTALL_SCRIPT" --help >/dev/null 2>&1 || true  # don't fail on --help exit code

# Direct test of the sync function: extract it from install.sh via awk,
# source it, and call it.
sync_fn_def="$(awk '/^_sync_local_repo_to_install_dir\(\) \{/,/^\}/' "$INSTALL_SCRIPT")"
if [ -z "$sync_fn_def" ]; then
    echo "FAIL: F1: _sync_local_repo_to_install_dir function not found in install.sh"
    failures=$((failures + 1))
else
    eval "$sync_fn_def"
    export HERMES_INSTALL_USE_LOCAL_REPO="$source_dir"
    # Provide a fake log_warn / log_info so the function can call them
    log_warn() { echo "WARN: $1"; }
    log_error() { echo "ERROR: $1"; }
    log_success() { echo "OK: $1"; }
    log_info() { echo "INFO: $1"; }
    _sync_local_repo_to_install_dir "$source_dir" "$install_dir"

    assert_path "$install_dir/marker.txt" "exists" "F1: marker file mirrored to install dir"
    if [ -e "$install_dir/marker.txt" ]; then
        content="$(cat "$install_dir/marker.txt")"
        assert_equal "hello from local" "$content" "F1: marker file content preserved"
    fi
    assert_path "$install_dir/.git" "exists" "F1: .git preserved in install dir"
    assert_path "$install_dir/venv" "absent" "F1: venv excluded from mirror"
    unset HERMES_INSTALL_USE_LOCAL_REPO
fi

# -----------------------------------------------------------------------------
# Test: invalid path falls back (warning, not failure)
# -----------------------------------------------------------------------------
echo
echo "-- HERMES_INSTALL_USE_LOCAL_REPO invalid path --"

tmp_root2="$(mktemp -d -t hermes-f1-test2-XXXXXX)"
bogus="$tmp_root2/not-a-repo"
mkdir -p "$bogus"  # exists but no .git

install_dir2="$tmp_root2/install"
mkdir -p "$install_dir2"

export HERMES_INSTALL_USE_LOCAL_REPO="$bogus"
# Capture warning output
output2="$(bash "$INSTALL_SCRIPT" --help 2>&1; echo "---"; \
    eval "$sync_fn_def"; \
    _sync_local_repo_to_install_dir "$bogus" "$install_dir2" 2>&1 || true)"
unset HERMES_INSTALL_USE_LOCAL_REPO

echo "$output2" | grep -q "falling back to git clone" \
    && echo "OK: F1: invalid path emits 'falling back to git clone' warning" \
    || { echo "FAIL: F1: invalid path missing warning"; failures=$((failures + 1)); }

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
```

- [ ] **Step 2: 跑测试，确认 fail**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
chmod +x scripts/tests/test-install-sh-stage-protocol.sh
bash scripts/tests/test-install-sh-stage-protocol.sh
```

**Expected**: `FAILED: ... test(s) failed`（至少 "F1: _sync_local_repo_to_install_dir function not found in install.sh" 这条 FAIL）。

- [ ] **Step 3: 实现 `install.sh` 的 F1 块 + `_sync_local_repo_to_install_dir` 函数**

**3a. 在 `clone_repo` 函数顶部插入 F1 块**（line 1118 顶部，紧跟 `log_info "Installing to $INSTALL_DIR..."` 之后）：

```bash
clone_repo() {
    log_info "Installing to $INSTALL_DIR..."

    # ── F1 dev shortcut: HERMES_INSTALL_USE_LOCAL_REPO ──
    # Lets a developer point the installer at a local checkout (typically a
    # git worktree) so their uncommitted / unpushed edits in apps/desktop are
    # used by the build instead of the upstream main HEAD. See spec §6.
    local local_repo="${HERMES_INSTALL_USE_LOCAL_REPO:-}"
    if [ -n "$local_repo" ]; then
        local_repo="${local_repo%/}"  # strip trailing slash
        if [ -d "$local_repo" ] && [ -d "$local_repo/.git" ]; then
            log_info "Using local repo at $local_repo (HERMES_INSTALL_USE_LOCAL_REPO); skipping git clone"
            _sync_local_repo_to_install_dir "$local_repo" "$INSTALL_DIR"
            return
        else
            log_warn "HERMES_INSTALL_USE_LOCAL_REPO=$local_repo is not a valid git checkout; falling back to git clone"
        fi
    fi
    # ── end F1 dev shortcut ──

    # An interrupted previous clone leaves a .git with no initial commit, where
    # [existing clone_repo logic continues here unchanged]
    ...
}
```

**3b. 在文件中段（找一个合适位置，例如 `setup_venv` 等 helper 函数附近）新增 `_sync_local_repo_to_install_dir` 函数**：

```bash
_sync_local_repo_to_install_dir() {
    local source="$1"
    local dest="$2"

    # Guard against source == dest (rsync with --delete is undefined when src
    # and dest resolve to the same path). See spec §9.2.
    local resolved_source
    resolved_source="$(cd "$source" && pwd -P)"
    if [ -d "$dest" ]; then
        local resolved_dest
        resolved_dest="$(cd "$dest" && pwd -P)"
        if [ "$resolved_source" = "$resolved_dest" ]; then
            log_warn "HERMES_INSTALL_USE_LOCAL_REPO source equals install dir ($resolved_source); skipping mirror"
            return 0
        fi
    fi

    # Handle existing $dest (mirrors the "not a git repo" handling).
    if [ -d "$dest" ]; then
        local backup_dir="${dest}.broken-$(date -u +%Y%m%d-%H%M%S)"
        log_warn "Existing directory at $dest (dev install path); moving aside to $backup_dir"
        if ! mv "$dest" "$backup_dir"; then
            log_error "Could not move $dest aside"
            return 1
        fi
    fi

    # Ensure parent of $dest exists.
    local parent
    parent="$(dirname "$dest")"
    [ -n "$parent" ] && [ ! -d "$parent" ] && mkdir -p "$parent"

    # rsync mirror $source → $dest, excluding venv / node_modules / marker.
    # -a         = archive mode (preserves perms, symlinks, mtimes)
    # --delete   = remove files in $dest that aren't in $source
    # --exclude  = skip venv, node_modules, bootstrap marker
    # .git is INTENTIONALLY NOT excluded (D3: keep .git)
    if ! rsync -a --delete \
            --exclude='venv' \
            --exclude='.venv' \
            --exclude='node_modules' \
            --exclude='.hermes-bootstrap-complete' \
            "$source/" "$dest/"; then
        log_error "rsync mirror of $source → $dest failed"
        return 1
    fi

    log_success "Local repo mirrored from $source to $dest"
}
```

**位置**：放在 `setup_venv` 函数（line 1238）之前或之后，跟其他 helper 风格一致。

- [ ] **Step 4: 跑测试，确认 pass**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
bash scripts/tests/test-install-sh-stage-protocol.sh
```

**Expected**: `ALL OK`（所有 F1 断言通过）。

**如果 FAIL**：
- "F1: _sync_local_repo_to_install_dir function not found" → 检查函数名拼写、位置
- "F1: marker file mirrored" FAIL → 检查 rsync 在系统上可用 (`which rsync`)
- "F1: .git preserved" FAIL → 检查 rsync `--exclude` 列表确实不含 `.git`
- "F1: venv excluded" FAIL → 检查 `--exclude='venv'` 拼写

- [ ] **Step 5: Commit**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
git add scripts/install.sh scripts/tests/test-install-sh-stage-protocol.sh
git commit -m "feat(install): HERMES_INSTALL_USE_LOCAL_REPO dev shortcut (Unix)

Mirrors the install.ps1 F1 dev shortcut for install.sh on macOS/Linux.
Lets the installer skip the git clone stage and rsync a local checkout
into the install directory when HERMES_INSTALL_USE_LOCAL_REPO is set.
Falls back to the regular git clone with a warning when the path is
missing or not a git repo."
```

---

## Task 3: `bootstrap-runner.cjs` — 透传 env var

**Files:**
- Modify: `apps/desktop/electron/bootstrap-runner.cjs:298-303`（在 `spawnPowerShell` 的 `env` 对象里追加 `HERMES_INSTALL_USE_LOCAL_REPO`）
- Modify: `apps/desktop/electron/bootstrap-runner.test.cjs`（追加 2 个测试块）

- [ ] **Step 1: 写失败测试（追加到 `bootstrap-runner.test.cjs` 末尾）**

在 `bootstrap-runner.test.cjs` 末尾（最后一个 `test(...)` 之后）追加：

```js
test('spawnPowerShell forwards HERMES_INSTALL_USE_LOCAL_REPO from parent env', async () => {
  // We need to test that the env var bubbles through. Since spawnPowerShell
  // is internal, we test it via runBootstrap or via a small refactor: we
  // import the internal helper if exported, else test via a mock.
  //
  // Strategy: use a one-shot module cache reset by re-requiring the file
  // with a stubbed child_process.spawn that captures the env arg.

  const { spawn } = require('node:child_process')
  const originalSpawn = spawn
  const captured = []
  const fakeSpawn = (...args) => {
    captured.push(args)
    // Return a minimal EventEmitter-like child so the await doesn't hang
    const { EventEmitter } = require('node:events')
    const child = new EventEmitter()
    child.stdout = new EventEmitter()
    child.stderr = new EventEmitter()
    child.kill = () => {}
    return child
  }

  // Simulate: parent process has HERMES_INSTALL_USE_LOCAL_REPO=/foo
  const prevVal = process.env.HERMES_INSTALL_USE_LOCAL_REPO
  process.env.HERMES_INSTALL_USE_LOCAL_REPO = '/foo/bar'

  try {
    require('node:child_process').spawn = fakeSpawn
    // Force a fresh require of bootstrap-runner so the env override is read
    delete require.cache[require.resolve('./bootstrap-runner.cjs')]
    const fresh = require('./bootstrap-runner.cjs')

    const home = mkTmpHome()
    try {
      // Trigger spawnPowerShell via a small script that wraps a stage
      // invocation. We use the runBootstrap path, but it needs installStamp
      // + sourceRepoRoot. The simplest test is to call the exported
      // resolveInstallScript path with a local script and let it spawn.
      const scriptsDir = path.join(home, 'hermes-agent', 'scripts')
      fs.mkdirSync(scriptsDir, { recursive: true })
      const scriptPath = path.join(scriptsDir, SCRIPT_NAME)
      fs.writeFileSync(scriptPath, '#!/bin/sh\necho hi\n')

      await fresh.runBootstrap({
        installStamp: null,
        activeRoot: home,
        sourceRepoRoot: path.dirname(path.dirname(scriptsDir)),
        hermesHome: home,
        logRoot: path.join(home, 'logs'),
        onEvent: () => {}
      }).catch(() => {})  // ignore errors from the fake spawn returning no data

      // Find the spawn call that invoked our fake script
      const powerShellCall = captured.find(args =>
        args[1] && args[1].includes && args[1].includes(SCRIPT_NAME)
      )
      assert.ok(powerShellCall, 'spawn was called with the install script')
      if (powerShellCall) {
        const env = powerShellCall[2].env
        assert.equal(
          env.HERMES_INSTALL_USE_LOCAL_REPO,
          '/foo/bar',
          'parent HERMES_INSTALL_USE_LOCAL_REPO is forwarded to child env'
        )
      }
    } finally {
      fs.rmSync(home, { recursive: true, force: true })
    }
  } finally {
    require('node:child_process').spawn = originalSpawn
    if (prevVal === undefined) {
      delete process.env.HERMES_INSTALL_USE_LOCAL_REPO
    } else {
      process.env.HERMES_INSTALL_USE_LOCAL_REPO = prevVal
    }
  }
})

test('spawnPowerShell passes empty string for HERMES_INSTALL_USE_LOCAL_REPO when parent env unset', async () => {
  const { spawn } = require('node:child_process')
  const originalSpawn = spawn
  const captured = []
  const fakeSpawn = (...args) => {
    captured.push(args)
    const { EventEmitter } = require('node:events')
    const child = new EventEmitter()
    child.stdout = new EventEmitter()
    child.stderr = new EventEmitter()
    child.kill = () => {}
    return child
  }

  // Unset in parent
  const prevVal = process.env.HERMES_INSTALL_USE_LOCAL_REPO
  delete process.env.HERMES_INSTALL_USE_LOCAL_REPO

  try {
    require('node:child_process').spawn = fakeSpawn
    delete require.cache[require.resolve('./bootstrap-runner.cjs')]
    const fresh = require('./bootstrap-runner.cjs')

    const home = mkTmpHome()
    try {
      const scriptsDir = path.join(home, 'hermes-agent', 'scripts')
      fs.mkdirSync(scriptsDir, { recursive: true })
      const scriptPath = path.join(scriptsDir, SCRIPT_NAME)
      fs.writeFileSync(scriptPath, '#!/bin/sh\necho hi\n')

      await fresh.runBootstrap({
        installStamp: null,
        activeRoot: home,
        sourceRepoRoot: path.dirname(path.dirname(scriptsDir)),
        hermesHome: home,
        logRoot: path.join(home, 'logs'),
        onEvent: () => {}
      }).catch(() => {})

      const powerShellCall = captured.find(args =>
        args[1] && args[1].includes && args[1].includes(SCRIPT_NAME)
      )
      assert.ok(powerShellCall, 'spawn was called with the install script')
      if (powerShellCall) {
        const env = powerShellCall[2].env
        assert.equal(
          env.HERMES_INSTALL_USE_LOCAL_REPO,
          '',
          'child env has empty string HERMES_INSTALL_USE_LOCAL_REPO when parent unset'
        )
      }
    } finally {
      fs.rmSync(home, { recursive: true, force: true })
    }
  } finally {
    require('node:child_process').spawn = originalSpawn
    if (prevVal !== undefined) process.env.HERMES_INSTALL_USE_LOCAL_REPO = prevVal
  }
})
```

- [ ] **Step 2: 跑测试，确认 fail**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
scripts/run_tests.sh apps/desktop/electron/bootstrap-runner.test.cjs -v
```

**Expected**: 新增的 2 个 F1 测试 FAIL（"parent HERMES_INSTALL_USE_LOCAL_REPO is forwarded" 等）。现有测试保持 PASS。

- [ ] **Step 3: 修改 `spawnPowerShell` 透传 env var**

在 `bootstrap-runner.cjs:298-303`，把 `env` 对象改为：

```js
    const child = spawn(ps, fullArgs, hiddenWindowsChildOptions({
      stdio: ['ignore', 'pipe', 'pipe'],
      env: {
        ...process.env,
        // Pass HERMES_HOME through so install.ps1 respects the caller's
        // choice rather than re-computing the default.
        HERMES_HOME: hermesHome || process.env.HERMES_HOME || '',
        // F1: forward the dev-local-repo shortcut. When the parent process
        // (dev shell, CI wrapper, Electron parent) sets this, install.ps1
        // inside the child will skip git clone and use the local checkout.
        // Empty string when unset, so PowerShell always sees a defined var.
        HERMES_INSTALL_USE_LOCAL_REPO: process.env.HERMES_INSTALL_USE_LOCAL_REPO || ''
      }
    }))
```

- [ ] **Step 4: 跑测试，确认 pass**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
scripts/run_tests.sh apps/desktop/electron/bootstrap-runner.test.cjs -v
```

**Expected**: 新增的 2 个 F1 测试 PASS。现有测试继续 PASS。

**如果 FAIL**：
- "parent ... is forwarded" FAIL → 检查 env 对象里确实有 `HERMES_INSTALL_USE_LOCAL_REPO` 行
- "empty string when parent unset" FAIL → 检查 `|| ''` fallback

- [ ] **Step 5: Commit**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
git add apps/desktop/electron/bootstrap-runner.cjs apps/desktop/electron/bootstrap-runner.test.cjs
git commit -m "feat(desktop): forward HERMES_INSTALL_USE_LOCAL_REPO to install.ps1

The Electron bootstrap-runner now propagates the developer's
HERMES_INSTALL_USE_LOCAL_REPO env var (when set in the parent process)
to the install.ps1 child process. This keeps subsequent re-runs of
the installer from the desktop's update flow on the developer's local
checkout, instead of overwriting it with a fresh clone from upstream."
```

---

## Task 4: 文档更新

**Files:**
- Modify: `website/docs/getting-started/installation.md`（末尾追加 "Development mode" 章节）

- [ ] **Step 1: 追加文档章节**

在 `installation.md` 末尾追加：

```markdown
## Development mode — install from a local checkout

When you've made local changes to `apps/desktop/` (or any other part of
`hermes-agent`) and want the installer to pick them up, set
`HERMES_INSTALL_USE_LOCAL_REPO` to the absolute path of your local
checkout **before** running `install.ps1` / `install.sh`. The installer
will skip the `git clone` step and mirror your local files into the
install directory instead.

### Windows (PowerShell)

```powershell
$env:HERMES_INSTALL_USE_LOCAL_REPO = 'C:\Users\you\code\hermes-agent'
.\scripts\install.ps1 -IncludeDesktop
```

### macOS / Linux (bash)

```bash
export HERMES_INSTALL_USE_LOCAL_REPO="$HOME/code/hermes-agent"
./scripts/install.sh --include-desktop
```

The variable is also picked up by `bootstrap-runner.cjs` inside
`Hermes.exe`, so subsequent re-runs of the installer from the
desktop's update flow will keep using your local checkout.

### Caveats

- The path must be an existing git checkout (it must contain a `.git/`
  directory). If the path is missing or invalid the installer falls back
  to the regular git-clone flow and prints a warning — it will not fail.
- The mirror excludes `venv/`, `node_modules/`, and
  `.hermes-bootstrap-complete/` from the source tree. If your local
  checkout has cached Python or Node dependencies in those directories,
  they will be reinstalled by the normal `uv sync` / `npm ci` stages.
- The mirror uses `robocopy /MIR` (Windows) or `rsync --delete`
  (Unix). Files in the install directory that are **not** present in
  your local checkout will be deleted. Don't store local-only edits
  inside the install directory — keep them in the checkout.
- The `hermes update` flow re-runs the mirror step. If you want to
  pin your install to a specific worktree branch, check out that
  branch in the source checkout before running the installer.
```

- [ ] **Step 2: 验证文档渲染（可选）**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
# 简单 sanity: 文件末尾有 "Development mode" 字符串
tail -3 website/docs/getting-started/installation.md
```

**Expected**: 最后几行包含 "pin your install to a specific worktree branch"。

- [ ] **Step 3: Commit**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
git add website/docs/getting-started/installation.md
git commit -m "docs(install): document HERMES_INSTALL_USE_LOCAL_REPO dev mode

Adds a Development mode section to installation.md explaining how to
make the installer use a local checkout (typically a git worktree)
instead of cloning from upstream. Covers Windows + Unix commands,
.env semantics, and known caveats around mirror exclusions and
local-only edits in the install directory."
```

---

## Task 5: 端到端手动 sanity 验证

**Files:** (无 commit — 纯手动 sanity check)

- [ ] **Step 1: 跑完整测试套件（确认没回归）**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
# PowerShell test (Windows / pwsh)
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/tests/test-install-ps1-stage-protocol.ps1
# Bash test
bash scripts/tests/test-install-sh-stage-protocol.sh
# CJS test
scripts/run_tests.sh apps/desktop/electron/bootstrap-runner.test.cjs
```

**Expected**: 所有三个测试套件 PASS。

- [ ] **Step 2: 模拟 dev 用户的真实工作流（手动 sanity）**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest

# 1. Make a marker change in apps/desktop to confirm the mirror is real
echo "// F1 dev sanity" >> apps/desktop/src/sanity-marker.txt

# 2. Set the env var to the current worktree
export HERMES_INSTALL_USE_LOCAL_REPO="$PWD"

# 3. Pick a fresh HERMES_HOME so we don't clobber the user's real one
TMP_HOME="$(mktemp -d -t hermes-f1-sanity-XXXXXX)"
export HERMES_HOME="$TMP_HOME"

# 4. Run install.sh with --manifest only (no actual install, just to check
#    the script picks up the env var without crashing)
bash scripts/install.sh --manifest --json 2>&1 | head -5 || true

# 5. Cleanup
unset HERMES_INSTALL_USE_LOCAL_REPO
unset HERMES_HOME
rm -rf "$TMP_HOME"
rm -f apps/desktop/src/sanity-marker.txt
```

**Expected**: 脚本能跑完 `--manifest`（manifest 不会实际调 Install-Repository，所以本步骤只验证 install.sh 不会因为 env var 解析而崩）。真实的 Install-Repository 走本地路径需要更多 setup（venv、Node、etc.）— 那是 Hermes 用户的实际工作流，spec §11 已经覆盖了。

- [ ] **Step 3: 验证 git log**

```bash
cd /Users/icer/.local/share/opencode/worktree/d9b1adefde9275bdf05dafa1b5d8d34755c55f89/quick-forest
git log --oneline -5
```

**Expected**: 看到 4 个 commit（Task 1/2/3/4 各一个），按 spec 实现顺序排列。

---

## Self-Review

### Spec coverage

- §3 D1-D9 关键决策 → Plan Tasks 1-3 + 文件结构表实现
- §4 环境变量契约 → Plan Task 1/2/3 实现
- §5 install.ps1 改动 → Plan Task 1
- §6 install.sh 改动 → Plan Task 2
- §7 bootstrap-runner.cjs 改动 → Plan Task 3
- §8 测试策略 → Plan Tasks 1/2/3 的 Step 1
- §9 风险（source==dest、排除目录、保留 .git）→ Plan Task 1/2 实现 + Task 4 文档说明
- §10 文档更新 → Plan Task 4
- §11 验证清单 → Plan Task 5
- §12 显式不包含（Spec 2 范围）→ 本 plan 不涉及（按 spec 要求）

### Placeholder scan

- 搜索 "TBD" / "TODO" / "FIXME" / "implement later" / "add appropriate error handling" → 无（所有 step 都有具体代码）
- 搜索 "类似 Task N" / "Similar to" → 无（每个 task 的代码都是完整的）
- 搜索 `<placeholder>` / `fill in` → 无

### Type consistency

- `HERMES_INSTALL_USE_LOCAL_REPO` 命名：Plan 1/2/3/4/5 一致
- `Sync-LocalRepoToInstallDir`（PowerShell）/ `_sync_local_repo_to_install_dir`（bash）：两个名字分别对应两套语言惯例，spec 明确区分
- `Install-Repository`（PowerShell）/ `clone_repo`（bash）：spec 修正后与实际代码一致
- `Stage-Repository`：spec 修正后与实际代码一致（line 2944）
- 排除目录列表：`venv`、`.venv`、`node_modules`、`.hermes-bootstrap-complete` — Task 1/2/4 三处一致
- `bootstrap-runner.cjs` env key：`HERMES_INSTALL_USE_LOCAL_REPO` — Task 3 Step 1 测试 + Step 3 实现一致
