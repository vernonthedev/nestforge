param(
    [string]$TargetVersion = "",
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

function Invoke-Git {
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Args
    )

    $output = & git @Args 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Args -join ' ') failed.`n$($output | Out-String)"
    }
    $output
}

function Get-WorkspaceVersion {
    param([string]$Path)

    $content = Get-Content -Raw -Path $Path
    $match = [regex]::Match($content, '(?ms)^\[workspace\.package\].*?^version\s*=\s*"(?<version>\d+\.\d+\.\d+)"\s*$')
    if (-not $match.Success) {
        throw "Could not locate [workspace.package] version in $Path"
    }
    $match.Groups["version"].Value
}

function Get-LatestVersionTag {
    @(Invoke-Git tag --list "v*" --sort=-version:refname | Select-Object -First 1)[0]
}

function Get-CommitObjects {
    param([string]$SinceTag)

    $range = if ([string]::IsNullOrWhiteSpace($SinceTag)) { "HEAD" } else { "$SinceTag..HEAD" }
    $lines = Invoke-Git log $range --format="%H`t%s"

    $commits = foreach ($line in $lines) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $parts = $line -split "`t", 2
        if ($parts.Count -lt 2) { continue }
        [PSCustomObject]@{
            Sha     = $parts[0]
            Subject = $parts[1]
        }
    }

    $commits
}

function Get-ReleasableCommits {
    param([object[]]$Commits)

    $Commits | Where-Object {
        $_.Subject -match '^(feat|fix|perf|refactor|docs|chore)(\(.+\))?(!)?:' -and
        $_.Subject -notmatch '^chore\(release\):' -and
        $_.Subject -ne 'docs: sync from repository'
    }
}

function Get-VersionBump {
    param([object[]]$Commits)

    if ($Commits | Where-Object { $_.Subject -match '^(feat|fix|perf|refactor|docs|chore)(\(.+\))?!:' }) {
        return "major"
    }
    if ($Commits | Where-Object { $_.Subject -match 'BREAKING CHANGE' }) {
        return "major"
    }
    if ($Commits | Where-Object { $_.Subject -match '^feat(\(.+\))?:' }) {
        return "minor"
    }
    "patch"
}

function Get-NextVersion {
    param(
        [string]$CurrentVersion,
        [string]$Bump
    )

    $parts = $CurrentVersion.Split(".") | ForEach-Object { [int]$_ }
    $major = $parts[0]
    $minor = $parts[1]
    $patch = $parts[2]

    switch ($Bump) {
        "major" { return "{0}.0.0" -f ($major + 1) }
        "minor" { return "{0}.{1}.0" -f $major, ($minor + 1) }
        default { return "{0}.{1}.{2}" -f $major, $minor, ($patch + 1) }
    }
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

function Convert-ToChangelogSection {
    param(
        [string]$Version,
        [string]$PreviousTag,
        [object[]]$Commits
    )

    $date = (Get-Date).ToString("yyyy-MM-dd")
    $compareFrom = if ([string]::IsNullOrWhiteSpace($PreviousTag)) { "v$Version" } else { $PreviousTag }
    $header = "## [$Version](https://github.com/vernonthedev/nestforge/compare/$compareFrom...v$Version) ($date)"

    $groups = [ordered]@{
        "Features" = @($Commits | Where-Object { $_.Subject -match '^feat(\(.+\))?(!)?:' })
        "Fixes"    = @($Commits | Where-Object { $_.Subject -match '^fix(\(.+\))?(!)?:' })
        "Other"    = @($Commits | Where-Object { $_.Subject -notmatch '^(feat|fix)(\(.+\))?(!)?:' })
    }

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add($header)
    $lines.Add("")

    foreach ($group in $groups.GetEnumerator()) {
        if ($group.Value.Count -eq 0) { continue }
        $lines.Add("### $($group.Key)")
        $lines.Add("")
        foreach ($commit in $group.Value) {
            $shortSha = $commit.Sha.Substring(0, 7)
            $lines.Add("* $($commit.Subject) ([${shortSha}](https://github.com/vernonthedev/nestforge/commit/$($commit.Sha)))")
        }
        $lines.Add("")
    }

    ($lines -join "`n").TrimEnd()
}

function Update-Changelog {
    param(
        [string]$Path,
        [string]$Section
    )

    $existing = Get-Content -Raw -Path $Path
    if ($existing -match [regex]::Escape($Section)) {
        return $false
    }

    $prefix = "# Changelog`n`nAll notable changes to the ``nestforge`` crate are documented in this file."
    if ($existing.StartsWith($prefix)) {
        $rest = $existing.Substring($prefix.Length).TrimStart("`r", "`n")
        $updated = "$prefix`n`n$Section`n`n$rest".TrimEnd() + "`n"
    } else {
        $updated = "$existing`n`n$Section`n"
    }

    Set-Content -Path $Path -Value $updated
    $true
}

function Get-PublishRetryDelaySeconds {
    param([string]$PublishOutput)

    $match = [regex]::Match(
        $PublishOutput,
        'Please try again after (?<retry>[A-Z][a-z]{2}, \d{2} [A-Z][a-z]{2} \d{4} \d{2}:\d{2}:\d{2} GMT)'
    )

    if (-not $match.Success) {
        return 60
    }

    $retryAt = [DateTimeOffset]::ParseExact(
        $match.Groups["retry"].Value,
        "ddd, dd MMM yyyy HH:mm:ss 'GMT'",
        [System.Globalization.CultureInfo]::InvariantCulture
    )

    $delay = [Math]::Ceiling(($retryAt - [DateTimeOffset]::UtcNow).TotalSeconds) + 5
    if ($delay -lt 5) {
        return 5
    }

    [int]$delay
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

    for ($attempt = 1; $attempt -le 5; $attempt++) {
        Write-Host ("cargo " + ($args -join " ")) -ForegroundColor DarkGray
        $output = & cargo @args 2>&1
        $output | ForEach-Object { Write-Host $_ }

        if ($LASTEXITCODE -eq 0) {
            return
        }

        $all = ($output | Out-String)
        if ($all -match "already exists on crates.io index") {
            Write-Host "Skipping $CrateName (already published for this version)." -ForegroundColor Yellow
            return
        }

        if ($all -match "429 Too Many Requests") {
            if ($attempt -eq 5) {
                throw "Publish failed for $CrateName after repeated crates.io rate limiting."
            }

            $delaySeconds = Get-PublishRetryDelaySeconds -PublishOutput $all
            Write-Host "crates.io rate limited $CrateName. Retrying in $delaySeconds seconds." -ForegroundColor Yellow
            Start-Sleep -Seconds $delaySeconds
            continue
        }

        throw "Publish failed for $CrateName."
    }
}

$repoRoot = Resolve-Path "."
$rootCargo = Join-Path $repoRoot "Cargo.toml"
$crateChangelog = Join-Path $repoRoot "crates\nestforge\CHANGELOG.md"
$rootChangelog = Join-Path $repoRoot "CHANGELOG.md"
$releaseNotesPath = Join-Path $repoRoot "target\release-notes.md"

if (-not (Test-Path $rootCargo)) {
    throw "Run this script from the repository root (Cargo.toml not found)."
}

$currentVersion = Get-WorkspaceVersion -Path $rootCargo
$latestTag = Get-LatestVersionTag
$allCommits = Get-CommitObjects -SinceTag $latestTag
$releasableCommits = @(Get-ReleasableCommits -Commits $allCommits)

if ([string]::IsNullOrWhiteSpace($TargetVersion)) {
    if ($releasableCommits.Count -eq 0) {
        Write-Step "No releasable commits found since $latestTag"
        exit 0
    }

    $bump = Get-VersionBump -Commits $releasableCommits
    $TargetVersion = Get-NextVersion -CurrentVersion $currentVersion -Bump $bump
}

if ($TargetVersion -eq $currentVersion -and $releasableCommits.Count -eq 0) {
    Write-Step "Current version already matches target and there are no releasable commits"
    exit 0
}

$newTag = "v$TargetVersion"
$section = Convert-ToChangelogSection -Version $TargetVersion -PreviousTag $latestTag -Commits $releasableCommits

Write-Step "Bumping workspace version to $TargetVersion"
$changed = @()

$rootChanged = Update-FileContent -Path $rootCargo -Transform {
    param($content)
    [regex]::Replace(
        $content,
        '(?ms)(^\[workspace\.package\].*?^version\s*=\s*")(\d+\.\d+\.\d+)(")',
        {
            param($m)
            $m.Groups[1].Value + $TargetVersion + $m.Groups[3].Value
        },
        1
    )
}
if ($rootChanged) { $changed += "Cargo.toml" }

Write-Step "Updating internal nestforge dependency pins to $TargetVersion"
$cargoFiles = Get-ChildItem -Path (Join-Path $repoRoot "crates") -Directory |
    ForEach-Object { Join-Path $_.FullName "Cargo.toml" } |
    Where-Object { Test-Path $_ }

foreach ($file in $cargoFiles) {
    $didChange = Update-FileContent -Path $file -Transform {
        param($content)
        $pattern = '(nestforge(?:-[A-Za-z0-9_-]+)?\s*=\s*\{[^}]*\bversion\s*=\s*")([^"]+)(")'
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

Write-Step "Updating changelogs"
if (Update-Changelog -Path $crateChangelog -Section $section) {
    $changed += "crates/nestforge/CHANGELOG.md"
}
if (Test-Path $rootChangelog) {
    if (Update-Changelog -Path $rootChangelog -Section $section) {
        $changed += "CHANGELOG.md"
    }
}

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $releaseNotesPath) | Out-Null
Set-Content -Path $releaseNotesPath -Value $section

if (-not $SkipChecks) {
    Write-Step "Running workspace checks"
    & cargo check --workspace
    if ($LASTEXITCODE -ne 0) { throw "cargo check failed." }
}

Write-Step "Creating release commit and tag"
$filesToAdd = @("Cargo.toml", "crates/nestforge/CHANGELOG.md")
$filesToAdd += $cargoFiles | ForEach-Object { Resolve-Path -Relative $_ }
if (Test-Path $rootChangelog) {
    $filesToAdd += "CHANGELOG.md"
}
Invoke-Git add @filesToAdd
Invoke-Git commit -m "chore(release): $TargetVersion [skip ci]"
Invoke-Git tag $newTag

if ($NoPublish) {
    Write-Step "NoPublish flag set, stopping after commit/tag steps"
    exit 0
}

$publishOrder = @(
    "nestforge-core",
    "nestforge-macros",
    "nestforge-config",
    "nestforge-data",
    "nestforge-db",
    "nestforge-orm",
    "nestforge-openapi",
    "nestforge-graphql",
    "nestforge-schedule",
    "nestforge-http",
    "nestforge-cache",
    "nestforge-microservices",
    "nestforge-grpc",
    "nestforge-websockets",
    "nestforge-mongo",
    "nestforge-redis",
    "nestforge-testing",
    "nestforge-cli",
    "nestforge"
)

Write-Step "Publishing crates in dependency order"
foreach ($crate in $publishOrder) {
    Invoke-CargoPublish -CrateName $crate -DryRunMode:$DryRun
    if (-not $DryRun) {
        Start-Sleep -Seconds 15
    }
}

if (-not $DryRun -and $env:GITHUB_TOKEN) {
    Write-Step "Pushing release commit and tag"
    Invoke-Git push origin HEAD:main
    Invoke-Git push origin $newTag

    Write-Step "Creating GitHub release"
    & gh release create $newTag --title $newTag --notes-file $releaseNotesPath
    if ($LASTEXITCODE -ne 0) {
        throw "gh release create failed."
    }
}

Write-Step "Done"
Write-Host "Target version: $TargetVersion"
Write-Host "Tag: $newTag"
if ($DryRun) {
    Write-Host "Mode: dry-run"
} else {
    Write-Host "Mode: publish"
}
