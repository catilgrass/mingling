#!/usr/bin/env pwsh

# Save original directory, restore after execution
$_load_original_dir = Get-Location

# Resolve script directory (works with dot-source: . ./load.ps1)
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $scriptPath) {
    Write-Error "load.ps1: failed to resolve script directory"
    return
}

# Load completion script mling_comp.ps1 from the .comp subdirectory
$compScript = Join-Path -Path $scriptPath -ChildPath ".comp" | Join-Path -ChildPath "mling_comp.ps1"
if (Test-Path $compScript) {
    . $compScript
}

# Change to script directory
try {
    Set-Location $scriptPath -ErrorAction Stop
} catch {
    Write-Error "load.ps1: failed to cd to script directory"
    return
}

# Add bin directories from all namespaces to PATH
$allDirs = Get-ChildItem -Directory
Get-ChildItem -Directory | Where-Object { Test-Path (Join-Path -Path $_.FullName -ChildPath "bin") } | ForEach-Object {
    $binPath = Join-Path -Path $_.FullName -ChildPath "bin"
    $env:PATH = "$binPath;$env:PATH"
}

# Helper function: execute script with appropriate shell
function _load_script {
    param([string]$script)
    if ($script -like "*.ps1") {
        & $script 2>$null
    }
}

# Iterate over all namespaces (top-level directories except .comp)
$nsDirs = Get-ChildItem -Directory -Exclude ".comp"
foreach ($_dir in $nsDirs) {
    $ns = $_dir.Name

    # Skip if UNTRUSTED marker exists
    $untrustedMarker = Join-Path -Path $ns -ChildPath "UNTRUSTED"
    if (Test-Path $untrustedMarker) { continue }

    $compDir = Join-Path -Path $ns -ChildPath "comp"
    if (-not (Test-Path $compDir -PathType Container)) { continue }

    # Find all loadable scripts under comp
    $scripts = Get-ChildItem -Path $compDir -Filter "*.ps1" -File -ErrorAction SilentlyContinue
    if (-not $scripts) { continue }

    $count = ($scripts | Measure-Object).Count

    # If TRUSTED marker exists, load directly
    $trustedMarker = Join-Path -Path $ns -ChildPath "TRUSTED"
    if (Test-Path $trustedMarker) {
        foreach ($_script in $scripts) { _load_script $_script.FullName }
        continue
    }

    # No marker, ask user
    $answer = Read-Host "'$ns' has $count completion script(s) to load, do you trust it? [Y/n] "
    if ($answer -eq "" -or $answer -match "^(y|yes)$") {
        # Mark as TRUSTED
        New-Item -ItemType File -Path $trustedMarker -Force | Out-Null

        # Ask whether to load immediately
        $load_answer = Read-Host "Load it immediately? [Y/n] "
        if ($load_answer -eq "" -or $load_answer -match "^(y|yes)$") {
            foreach ($_script in $scripts) { _load_script $_script.FullName }
        }
    } else {
        New-Item -ItemType File -Path $untrustedMarker -Force | Out-Null
    }
}

# Restore original working directory
try {
    Set-Location $_load_original_dir -ErrorAction Stop
} catch {}

# Cleanup
Remove-Variable -Name _load_original_dir -ErrorAction SilentlyContinue
Remove-Item Function:_load_script -ErrorAction SilentlyContinue
