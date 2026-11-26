# Builds ClickLoop (release) and packages the standalone Windows installer into dist/.
param(
    [switch]$SkipAppBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$DistDir = Join-Path $Root "dist"

Push-Location $Root
try {
    if (-not $SkipAppBuild) {
        Write-Host "==> Building ClickLoop (release)..." -ForegroundColor Cyan
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build ClickLoop release binary."
        }
    }

    $AppExe = Join-Path $Root "target\release\clickloop.exe"
    if (-not (Test-Path $AppExe)) {
        throw "Expected release binary not found at $AppExe"
    }

    Write-Host "==> Building ClickLoop-Setup.exe (release)..." -ForegroundColor Cyan
    cargo build --release --manifest-path (Join-Path $Root "installer\Cargo.toml")
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to build ClickLoop-Setup.exe."
    }

    New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

    $SetupSrc = Join-Path $Root "installer\target\release\ClickLoop-Setup.exe"
    $SetupDst = Join-Path $DistDir "ClickLoop-Setup.exe"
    $PortableDst = Join-Path $DistDir "clickloop.exe"

    Copy-Item -Force $SetupSrc $SetupDst
    Copy-Item -Force $AppExe $PortableDst

    $SetupSize = [Math]::Round((Get-Item $SetupDst).Length / 1KB, 1)
    $AppSize = [Math]::Round((Get-Item $PortableDst).Length / 1KB, 1)

    Write-Host ""
    Write-Host "Build succeeded!" -ForegroundColor Green
    Write-Host "  Installer : $SetupDst ($SetupSize KB)"
    Write-Host "  Portable  : $PortableDst ($AppSize KB)"
} finally {
    Pop-Location
}
