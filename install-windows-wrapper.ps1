# install-windows-wrapper.ps1
# PowerShell wrapper installer for dtree - similar to bash dt() function
# Supports both Windows PowerShell 5.x and PowerShell Core 7.x+
# This script can be run multiple times to update the wrapper function

Write-Host "=== dtree PowerShell Wrapper Installer ===" -ForegroundColor Cyan
Write-Host ""

# Detect PowerShell version
$psVersion = $PSVersionTable.PSVersion.Major
Write-Host "Running under PowerShell $($PSVersionTable.PSVersion)" -ForegroundColor Gray
Write-Host ""

# Define markers for identifying our code block
$START_MARKER = "# >>> dtree wrapper start <<<"
$END_MARKER = "# >>> dtree wrapper end <<<"

# The function definition
$functionCode = @"
$START_MARKER
function dt {
    param(
        [switch]`$v,      # -v flag for view mode
        [switch]`$view,   # --view flag for view mode
        [Parameter(ValueFromRemainingArguments=`$true)]
        [string[]]`$Arguments
    )

    `$prevDir = `$PWD.Path

    # Handle -v or --view flags: rebuild arguments array to include them
    if (`$v) {
        `$Arguments = @('-v') + `$Arguments
    }
    if (`$view) {
        `$Arguments = @('--view') + `$Arguments
    }

    # Handle dt without arguments → open interactive TUI
    if (`$Arguments.Count -eq 0) {
        # dtree.exe uses stderr for TUI and stdout for output path
        # We capture only stdout (path) but let stderr (TUI) display normally
        `$result = & dtree.exe
        if (`$LASTEXITCODE -eq 0 -and `$result -and (Test-Path `$result)) {
            Set-Location `$result
            `$env:DTREE_PREV_DIR = `$prevDir
        }
        return
    }

    # Handle dt - (return to previous directory)
    if (`$Arguments.Count -eq 1 -and `$Arguments[0] -eq "-") {
        if (`$env:DTREE_PREV_DIR -and (Test-Path `$env:DTREE_PREV_DIR)) {
            Set-Location `$env:DTREE_PREV_DIR
            `$env:DTREE_PREV_DIR = `$prevDir
        } else {
            Write-Error "dt: no previous directory"
        }
        return
    }

    # Handle flags that should run dtree directly without cd
    if (`$Arguments.Count -gt 0) {
        switch (`$Arguments[0]) {
            {`$_ -in "-h", "--help", "--version"} {
                & dtree.exe `$Arguments
                return
            }
            "-bm" {
                & dtree.exe `$Arguments
                return
            }
            {`$_ -in "-v", "--view"} {
                # View mode: need to resolve relative path to absolute
                if (`$Arguments.Count -ge 2) {
                    `$filePath = `$Arguments[1]

                    # Convert relative path to absolute
                    if (-not [System.IO.Path]::IsPathRooted(`$filePath)) {
                        `$filePath = Join-Path `$PWD.Path `$filePath
                    }

                    # Run dtree with absolute path (preserve original flag)
                    # Don't capture stderr - let TUI display
                    `$result = & dtree.exe `$Arguments[0] `$filePath
                    `$exitCode = `$LASTEXITCODE

                    if (`$exitCode -ne 0) {
                        return
                    }

                    # dtree may return a directory path to cd into
                    if (`$result -and (Test-Path `$result) -and (Test-Path `$result -PathType Container)) {
                        Set-Location `$result
                        `$env:DTREE_PREV_DIR = `$prevDir
                    }
                } else {
                    # No file specified, just pass through
                    & dtree.exe `$Arguments
                }
                return
            }
        }
    }

    # Navigation mode: capture stdout (path) only, let stderr display to user
    `$result = & dtree.exe `$Arguments
    `$exitCode = `$LASTEXITCODE

    if (`$exitCode -ne 0) {
        # Error occurred, dtree already printed to stderr
        return
    }

    if (`$result) {
        `$result = `$result | Out-String
        `$result = `$result.Trim()
    }

    # Only cd if result is a valid directory
    if (`$result -and (Test-Path `$result)) {
        Set-Location `$result
        `$env:DTREE_PREV_DIR = `$prevDir
    }
}
$END_MARKER
"@

# Function to install/update wrapper in a specific profile
function Install-WrapperToProfile {
    param(
        [string]$ProfilePath,
        [string]$ProfileName
    )

    Write-Host "Processing $ProfileName..." -ForegroundColor Cyan

    # Ensure profile directory exists
    $profileDir = Split-Path $ProfilePath -Parent
    if (-not (Test-Path $profileDir)) {
        Write-Host "  Creating profile directory: $profileDir" -ForegroundColor Yellow
        New-Item -Path $profileDir -ItemType Directory -Force | Out-Null
    }

    # Ensure profile file exists
    if (-not (Test-Path $ProfilePath)) {
        Write-Host "  Creating profile file: $ProfilePath" -ForegroundColor Yellow
        New-Item -Path $ProfilePath -ItemType File -Force | Out-Null
    }

    # Read current profile content
    $profileContent = Get-Content -Path $ProfilePath -Raw -ErrorAction SilentlyContinue
    if (-not $profileContent) {
        $profileContent = ""
    }

    # Check if our markers exist
    $hasStartMarker = $profileContent -match [regex]::Escape($START_MARKER)
    $hasEndMarker = $profileContent -match [regex]::Escape($END_MARKER)

    if ($hasStartMarker -and $hasEndMarker) {
        # Update existing installation
        Write-Host "  Found existing dtree wrapper, updating..." -ForegroundColor Yellow

        # Count how many blocks exist
        $startCount = ([regex]::Matches($profileContent, [regex]::Escape($START_MARKER))).Count
        $endCount = ([regex]::Matches($profileContent, [regex]::Escape($END_MARKER))).Count

        if ($startCount -gt 1 -or $endCount -gt 1) {
            Write-Host "  [WARNING] Found $startCount start markers and $endCount end markers" -ForegroundColor Yellow
            Write-Host "  Removing ALL dtree blocks and adding fresh copy..." -ForegroundColor Yellow
        }

        # Remove ALL old code blocks between markers (including markers)
        $pattern = "(?s)$([regex]::Escape($START_MARKER)).*?$([regex]::Escape($END_MARKER))"
        $newContent = $profileContent -replace $pattern, ''

        # Clean up multiple consecutive blank lines
        $newContent = $newContent -replace '(\r?\n){3,}', "`n`n"

        # Add the new function code
        $newContent = $newContent.TrimEnd() + "`n`n" + $functionCode

        # Save updated profile
        Set-Content -Path $ProfilePath -Value $newContent -NoNewline
        Write-Host "  [OK] Successfully updated 'dt' function" -ForegroundColor Green
        return $true

    } elseif ($profileContent -match "function\s+dt\s*\{") {
        # Function exists but without markers - warn user
        Write-Host "  [WARNING] Found existing 'dt' function without markers" -ForegroundColor Red
        Write-Host "  Your profile contains a 'dt' function that wasn't installed by this script." -ForegroundColor Yellow
        Write-Host "  To update safely, please:" -ForegroundColor Yellow
        Write-Host "    1. Backup your profile: Copy-Item '$ProfilePath' '$ProfilePath.backup'" -ForegroundColor White
        Write-Host "    2. Remove the existing 'dt' function manually" -ForegroundColor White
        Write-Host "    3. Run this installer again" -ForegroundColor White
        Write-Host ""
        return $false

    } else {
        # Fresh installation
        Write-Host "  Installing 'dt' function..." -ForegroundColor Green
        Add-Content -Path $ProfilePath -Value "`n$functionCode"
        Write-Host "  [OK] Successfully installed 'dt' function" -ForegroundColor Green
        return $true
    }
}

# Define profile paths for both PowerShell versions
$profiles = @(
    @{
        Name = "Windows PowerShell 5.x"
        Path = "$HOME\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1"
        ExeName = "powershell.exe"
        Version = 5
    },
    @{
        Name = "PowerShell Core 7.x+"
        Path = "$HOME\Documents\PowerShell\Microsoft.PowerShell_profile.ps1"
        ExeName = "pwsh.exe"
        Version = 7
    }
)

# Track installation results
$installedProfiles = @()
$skippedProfiles = @()

# Process each profile
foreach ($profile in $profiles) {
    # Check if PowerShell version is available
    $psExe = Get-Command $profile.ExeName -ErrorAction SilentlyContinue

    if (-not $psExe) {
        Write-Host "$($profile.Name) not found (skipping)" -ForegroundColor Gray
        Write-Host ""
        continue
    }

    Write-Host "Found $($profile.Name) at: $($psExe.Source)" -ForegroundColor Green

    # Install wrapper to this profile
    $success = Install-WrapperToProfile -ProfilePath $profile.Path -ProfileName $profile.Name

    if ($success) {
        $installedProfiles += $profile
    } else {
        $skippedProfiles += $profile
    }

    Write-Host ""
}

# Print summary
Write-Host "=== Installation Summary ===" -ForegroundColor Cyan
Write-Host ""

if ($installedProfiles.Count -gt 0) {
    Write-Host "Successfully installed/updated in:" -ForegroundColor Green
    foreach ($p in $installedProfiles) {
        Write-Host "  - $($p.Name)" -ForegroundColor Green
        Write-Host "    Profile: $($p.Path)" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($skippedProfiles.Count -gt 0) {
    Write-Host "Skipped (manual intervention required):" -ForegroundColor Yellow
    foreach ($p in $skippedProfiles) {
        Write-Host "  - $($p.Name)" -ForegroundColor Yellow
        Write-Host "    Profile: $($p.Path)" -ForegroundColor Gray
    }
    Write-Host ""
}

if ($installedProfiles.Count -gt 0) {
    Write-Host "=== Next Steps ===" -ForegroundColor Cyan
    Write-Host "1. Restart PowerShell or reload profile:" -ForegroundColor White
    if ($psVersion -eq 5) {
        Write-Host "   . `$PROFILE" -ForegroundColor White
    } else {
        Write-Host "   . `$PROFILE" -ForegroundColor White
    }
    Write-Host "2. Test with: dt --version" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "[ERROR] No profiles were updated successfully" -ForegroundColor Red
    Write-Host "Please resolve the issues above and run the installer again" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}
