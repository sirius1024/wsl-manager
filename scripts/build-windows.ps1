#Requires -Version 5.1
<#
.SYNOPSIS
  Build the WSL Manager Tauri app for Windows.
.DESCRIPTION
  This script installs npm dependencies and runs `npm run tauri:build`.
  Run it from the project root on Windows 11 (or Windows 10 1903+).
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot

try {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " WSL Manager - Windows Build Script" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # Check Node.js
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        throw "Node.js is not installed. Please install it from https://nodejs.org/"
    }
    Write-Host "Node.js: $(node -v)" -ForegroundColor Green

    # Check npm
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        throw "npm is not installed."
    }
    Write-Host "npm: $(npm -v)" -ForegroundColor Green

    # Check Rust / Cargo
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "Rust/Cargo not found. Please install Rust from https://www.rust-lang.org/tools/install" -ForegroundColor Yellow
        throw "cargo is required to build the Tauri backend."
    }
    Write-Host "Cargo: $(cargo -V)" -ForegroundColor Green

    # Install frontend dependencies
    Write-Host "`nInstalling npm dependencies..." -ForegroundColor Cyan
    npm install

    # Build the Tauri app
    Write-Host "`nBuilding Tauri app for Windows..." -ForegroundColor Cyan
    npm run tauri:build

    # Locate installer
    $bundleDir = [System.IO.Path]::Combine($projectRoot, "src-tauri", "target", "release", "bundle")
    if (Test-Path $bundleDir) {
        Write-Host "`nBuild complete! Artifacts:" -ForegroundColor Green
        Get-ChildItem -Path $bundleDir -Recurse -File |
            Select-Object -ExpandProperty FullName |
            ForEach-Object { Write-Host "  $_" }
    } else {
        Write-Host "`nBuild finished, but bundle directory was not found at $bundleDir" -ForegroundColor Yellow
    }
} finally {
    Pop-Location
}
