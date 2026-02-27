param(
    [string]$TargetVersion = "1.1.0",
    [switch]$SkipChecks,
    [switch]$DryRun,
    [switch]$NoPublish
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Update-FileContent {
    param(
        [string]$Path,
        [scriptblock]$Transform
    )

    $before = Get-Content -Raw -Path $Path
    $after = & $Transform $before
    if ($after -ne $before) {
        Set-Content -Path $Path -Value $after
        return $true
    }
    return $false
}

function Invoke-CargoPublish {
    param(
        [string]$CrateName,
        [switch]$DryRunMode
    )

    $args = @("publish", "-p", $CrateName, "--allow-dirty")
    if ($DryRunMode) {
        $args += "--dry-run"
    }

    Write-Host ("cargo " + ($args -join " ")) -ForegroundColor DarkGray
    $cmdLine = "cargo " + ($args -join " ") + " 2>&1"
    $output = cmd /c $cmdLine
    $output | ForEach-Object { Write-Host $_ }

    if ($LASTEXITCODE -eq 0) {
        return
    }

    $all = ($output | Out-String)
    if ($all -match "already exists on crates.io index") {
        Write-Host "Skipping $CrateName (already published for this version)." -ForegroundColor Yellow
        return
    }

    throw "Publish failed for $CrateName."
}

$repoRoot = Resolve-Path "."
$rootCargo = Join-Path $repoRoot "Cargo.toml"

if (-not (Test-Path $rootCargo)) {
    throw "Run this script from the repository root (Cargo.toml not found)."
}

Write-Step "Bumping workspace version to $TargetVersion"
$changed = @()

$rootChanged = Update-FileContent -Path $rootCargo -Transform {
    param($content)
    [regex]::Replace(
        $content,
        '(?m)^version\s*=\s*"\d+\.\d+\.\d+"\s*$',
        "version = `"$TargetVersion`"",
        1
    )
}
if ($rootChanged) { $changed += "Cargo.toml" }

Write-Step "Updating internal nestforge dependency pins to $TargetVersion"
$crateCargoFiles = Get-ChildItem -Path (Join-Path $repoRoot "crates") -Directory |
    ForEach-Object { Join-Path $_.FullName "Cargo.toml" } |
    Where-Object { Test-Path $_ }

foreach ($file in $crateCargoFiles) {
    $didChange = Update-FileContent -Path $file -Transform {
        param($content)
        $pattern = '(nestforge-[A-Za-z0-9_-]+\s*=\s*\{[^}]*\bversion\s*=\s*")([^"]+)(")'
        [regex]::Replace($content, $pattern, {
            param($m)
            $m.Groups[1].Value + $TargetVersion + $m.Groups[3].Value
        })
    }
    if ($didChange) {
        $relative = Resolve-Path -Relative $file
        $changed += $relative
    }
}

if ($changed.Count -eq 0) {
    Write-Host "No version text changes were needed."
} else {
    Write-Host "Updated files:"
    $changed | ForEach-Object { Write-Host " - $_" }
}

if (-not $SkipChecks) {
    Write-Step "Running workspace checks"
    & cargo check --workspace
    if ($LASTEXITCODE -ne 0) { throw "cargo check failed." }
}

if ($NoPublish) {
    Write-Step "NoPublish flag set, stopping after version/check steps"
    exit 0
}

$publishOrder = @(
    "nestforge-core",
    "nestforge-macros",
    "nestforge-config",
    "nestforge-data",
    "nestforge-db",
    "nestforge-openapi",
    "nestforge-http",
    "nestforge-testing",
    "nestforge-mongo",
    "nestforge-redis",
    "nestforge-orm",
    "nestforge-cli",
    "nestforge"
)

Write-Step "Publishing crates in dependency order"
foreach ($crate in $publishOrder) {
    Invoke-CargoPublish -CrateName $crate -DryRunMode:$DryRun
    if (-not $DryRun) {
        Start-Sleep -Seconds 20
    }
}

Write-Step "Done"
Write-Host "Target version: $TargetVersion"
if ($DryRun) {
    Write-Host "Mode: dry-run"
} else {
    Write-Host "Mode: publish"
}
