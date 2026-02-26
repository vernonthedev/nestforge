param(
    [switch]$Offline = $true,
    [switch]$RunRuntime,
    [switch]$RunPostgresMigrations
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $RepoRoot

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )

    Write-Host ""
    Write-Host "==> $Name" -ForegroundColor Cyan
    & $Action
}

function Cargo-Cmd {
    param([string[]]$Args)
    if ($Offline) {
        & cargo @Args --offline
    } else {
        & cargo @Args
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo command failed: cargo $($Args -join ' ')"
    }
}

Invoke-Step "Workspace check/test/clippy" {
    Cargo-Cmd @("check", "--workspace")
    Cargo-Cmd @("test", "--workspace")
    Cargo-Cmd @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
}

Invoke-Step "CLI help + utility commands" {
    Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "--help")
    Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "fmt")
    Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "docs")
    Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "init")
    Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "generate", "create_users_table")
    Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "status")
}

$TmpRoot = Join-Path $env:TEMP "nestforge-script-test"
$DemoAppDir = Join-Path $TmpRoot "demo_api"

Invoke-Step "Generator flow: new + module + resource" {
    if (Test-Path $TmpRoot) {
        Remove-Item -Recurse -Force $TmpRoot
    }
    New-Item -ItemType Directory -Path $TmpRoot | Out-Null

    Push-Location $TmpRoot
    try {
        Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "new", "demo_api")
        Push-Location $DemoAppDir
        try {
            Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "g", "module", "auth")
            Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "g", "resource", "users")
            Cargo-Cmd @("check")
        } finally {
            Pop-Location
        }
    } finally {
        Pop-Location
    }
}

Invoke-Step "Feature-gated API compile check" {
    Cargo-Cmd @("check", "-p", "nestforge", "--features", "db orm config openapi data mongo redis testing")
}

if ($RunRuntime) {
    Invoke-Step "Run hello-nestforge app (Ctrl+C to stop)" {
        if ($Offline) {
            & cargo run -p hello-nestforge --offline
        } else {
            & cargo run -p hello-nestforge
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to run hello-nestforge"
        }
    }
}

if ($RunPostgresMigrations) {
    Invoke-Step "Postgres migration flow (requires DATABASE_URL)" {
        Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "init")
        Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "generate", "smoke_migration")
        Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "migrate")
        Cargo-Cmd @("run", "-p", "nestforge-cli", "--", "db", "status")
    }
}

Write-Host ""
Write-Host "All selected feature checks completed." -ForegroundColor Green
