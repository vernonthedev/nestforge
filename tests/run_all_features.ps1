param(
    [switch]$Offline = $true,
    [switch]$RunRuntime,
    [switch]$RunPostgresMigrations
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$CliManifest = Join-Path $RepoRoot "crates/nestforge-cli/Cargo.toml"
$ExampleAppDir = Join-Path $RepoRoot "examples/hello-nestforge"
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
    param([string[]]$CommandArgs)
    if ($Offline) {
        & cargo --offline @CommandArgs
    } else {
        & cargo @CommandArgs
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo command failed: cargo $($CommandArgs -join ' ')"
    }
}

Invoke-Step "Workspace check/test/clippy" {
    Cargo-Cmd -CommandArgs @("check", "--workspace")
    Cargo-Cmd -CommandArgs @("test", "--workspace")
    Cargo-Cmd -CommandArgs @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
}

Invoke-Step "CLI help + utility commands" {
    Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "--help")
    Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "fmt")
    Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "docs")
    Push-Location $ExampleAppDir
    try {
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "init")
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "generate", "create_users_table")
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "status")
    } finally {
        Pop-Location
    }
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
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "new", "demo_api")
        $DemoCargoToml = Join-Path $DemoAppDir "Cargo.toml"
        $DemoCargo = Get-Content -Raw -Path $DemoCargoToml
        $LocalNestforgePath = (Join-Path $RepoRoot "crates/nestforge").Replace("\", "/")
        $DemoCargo = $DemoCargo -replace 'nestforge = ".*"', "nestforge = { path = `"$LocalNestforgePath`" }"
        Set-Content -Path $DemoCargoToml -Value $DemoCargo
        Push-Location $DemoAppDir
        try {
            Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "g", "module", "auth")
            Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "g", "resource", "users")
            Cargo-Cmd -CommandArgs @("check", "--manifest-path", (Join-Path $DemoAppDir "Cargo.toml"))
        } finally {
            Pop-Location
        }
    } finally {
        Pop-Location
    }
}

Invoke-Step "Feature-gated API compile check" {
    Cargo-Cmd -CommandArgs @("check", "-p", "nestforge", "--features", "db orm config openapi data mongo redis testing")
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
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "init")
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "generate", "smoke_migration")
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "migrate")
        Cargo-Cmd -CommandArgs @("run", "--manifest-path", $CliManifest, "--", "db", "status")
    }
}

Write-Host ""
Write-Host "All selected feature checks completed." -ForegroundColor Green
