# Smoke tests for the install.ps1 stage protocol.
#
# Run from a PowerShell prompt:
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/tests/test-install-ps1-stage-protocol.ps1
#
# These tests only exercise the metadata surface (-ProtocolVersion, -Manifest,
# unknown -Stage handling).  They DO NOT actually run any install stages --
# those have heavy side effects (winget, git clone, pip install, PATH writes)
# and are out of scope for a unit smoke test.  All three metadata commands
# below return without invoking Main / Invoke-AllStages.
#
# To exercise real install stages, drive the script from a clean VM.

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))
$installScript = Join-Path $repoRoot "scripts\install.ps1"

if (-not (Test-Path $installScript)) {
    throw "Could not locate install.ps1 at $installScript"
}

$failures = 0
function Assert-Equal {
    param([Parameter(Mandatory=$true)] $Expected,
          [Parameter(Mandatory=$true)] $Actual,
          [Parameter(Mandatory=$true)] [string]$Label)
    if ($Expected -ne $Actual) {
        Write-Host "FAIL: $Label" -ForegroundColor Red
        Write-Host "  expected: $Expected"
        Write-Host "  actual:   $Actual"
        $script:failures++
    } else {
        Write-Host "OK: $Label" -ForegroundColor Green
    }
}
function Assert-True {
    param([Parameter(Mandatory=$true)] $Condition,
          [Parameter(Mandatory=$true)] [string]$Label)
    if (-not $Condition) {
        Write-Host "FAIL: $Label" -ForegroundColor Red
        $script:failures++
    } else {
        Write-Host "OK: $Label" -ForegroundColor Green
    }
}

# -----------------------------------------------------------------------------
# Test: -ProtocolVersion emits a single integer
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "-- -ProtocolVersion --"
$output = & powershell -NoProfile -ExecutionPolicy Bypass -File $installScript -ProtocolVersion
Assert-Equal -Expected 0 -Actual $LASTEXITCODE -Label "-ProtocolVersion exits 0"
Assert-True ($output -match '^\d+$') -Label "-ProtocolVersion emits an integer (got: $output)"

# -----------------------------------------------------------------------------
# Test: -Manifest emits valid JSON with expected shape
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "-- -Manifest --"
$manifestJson = & powershell -NoProfile -ExecutionPolicy Bypass -File $installScript -Manifest
Assert-Equal -Expected 0 -Actual $LASTEXITCODE -Label "-Manifest exits 0"

$manifest = $null
try {
    $manifest = $manifestJson | ConvertFrom-Json
    Assert-True $true -Label "-Manifest output parses as JSON"
} catch {
    Assert-True $false -Label "-Manifest output parses as JSON (parse error: $_)"
}

if ($manifest) {
    Assert-True ($manifest.protocol_version -is [int] -or $manifest.protocol_version -is [long]) `
        -Label "manifest.protocol_version is an integer"
    Assert-True ($manifest.stages.Count -gt 0) -Label "manifest.stages is non-empty"

    # Every stage has the four required fields
    $allValid = $true
    foreach ($stage in $manifest.stages) {
        foreach ($field in @("name", "title", "category", "needs_user_input")) {
            if (-not ($stage.PSObject.Properties.Name -contains $field)) {
                Write-Host "  stage missing field '$field': $($stage | ConvertTo-Json -Compress)" -ForegroundColor Red
                $allValid = $false
            }
        }
    }
    Assert-True $allValid -Label "every stage has name/title/category/needs_user_input"

    # Specific stage names that the GUI driver will rely on
    $names = $manifest.stages | ForEach-Object { $_.name }
    foreach ($expected in @("uv", "python", "git", "venv", "dependencies", "configure", "gateway")) {
        Assert-True ($names -contains $expected) -Label "manifest contains stage '$expected'"
    }

    # The two known-interactive stages must declare needs_user_input
    $interactive = $manifest.stages | Where-Object { $_.needs_user_input } | ForEach-Object { $_.name }
    Assert-True ($interactive -contains "configure") -Label "'configure' stage flagged needs_user_input"
    Assert-True ($interactive -contains "gateway") -Label "'gateway' stage flagged needs_user_input"
}

# -----------------------------------------------------------------------------
# Test: unknown stage name -> exit 2, structured JSON error
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "-- -Stage with unknown name --"
$errOutput = & powershell -NoProfile -ExecutionPolicy Bypass -File $installScript -Stage "does-not-exist"
Assert-Equal -Expected 2 -Actual $LASTEXITCODE -Label "unknown -Stage exits 2"

$errFrame = $null
try {
    $errFrame = $errOutput | ConvertFrom-Json
    Assert-True $true -Label "unknown-stage output parses as JSON"
} catch {
    Assert-True $false -Label "unknown-stage output parses as JSON (parse error: $_)"
}

if ($errFrame) {
    Assert-Equal -Expected $false -Actual $errFrame.ok -Label "unknown-stage frame has ok=false"
    Assert-Equal -Expected "does-not-exist" -Actual $errFrame.stage -Label "unknown-stage frame echoes stage name"
    Assert-True ($errFrame.reason -match "unknown stage") -Label "unknown-stage frame explains why"
}

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

# -----------------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------------
Write-Host ""
if ($failures -gt 0) {
    Write-Host "FAILED: $failures assertion(s) failed" -ForegroundColor Red
    exit 1
} else {
    Write-Host "All smoke tests passed." -ForegroundColor Green
    exit 0
}
