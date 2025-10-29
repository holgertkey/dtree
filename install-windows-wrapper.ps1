# install-windows-wrapper.ps1
# PowerShell wrapper installer for dtree - similar to bash dt() function
# This script can be run multiple times to update the wrapper function

Write-Host "=== dtree PowerShell Wrapper Installer ===" -ForegroundColor Cyan
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

    # Navigation mode: capture stdout (path) separately from stderr (errors)
    `$result = & dtree.exe `$Arguments 2>&1
    `$exitCode = `$LASTEXITCODE

    if (`$exitCode -ne 0) {
        # Error occurred, dtree already printed to stderr
        return
    }

    `$result = `$result | Out-String
    `$result = `$result.Trim()

    # Only cd if result is a valid directory
    if (`$result -and (Test-Path `$result)) {
        Set-Location `$result
        `$env:DTREE_PREV_DIR = `$prevDir
    }
}
$END_MARKER
"@

# Ensure profile exists
$profilePath = $PROFILE
if (-not (Test-Path $profilePath)) {
    Write-Host "Creating PowerShell profile at: $profilePath" -ForegroundColor Yellow
    New-Item -Path $profilePath -ItemType File -Force | Out-Null
}

# Read current profile content
$profileContent = Get-Content -Path $profilePath -Raw -ErrorAction SilentlyContinue
if (-not $profileContent) {
    $profileContent = ""
}

# Check if our markers exist
$hasStartMarker = $profileContent -match [regex]::Escape($START_MARKER)
$hasEndMarker = $profileContent -match [regex]::Escape($END_MARKER)

if ($hasStartMarker -and $hasEndMarker) {
    # Update existing installation
    Write-Host "Found existing dtree wrapper, updating..." -ForegroundColor Yellow

    # Count how many blocks exist
    $startCount = ([regex]::Matches($profileContent, [regex]::Escape($START_MARKER))).Count
    $endCount = ([regex]::Matches($profileContent, [regex]::Escape($END_MARKER))).Count

    if ($startCount -gt 1 -or $endCount -gt 1) {
        Write-Host "[WARNING] Found $startCount start markers and $endCount end markers" -ForegroundColor Yellow
        Write-Host "Removing ALL dtree blocks and adding fresh copy..." -ForegroundColor Yellow
    }

    # Remove ALL old code blocks between markers (including markers)
    # PowerShell -replace replaces all matches by default (no need for loop or count parameter)
    $pattern = "(?s)$([regex]::Escape($START_MARKER)).*?$([regex]::Escape($END_MARKER))"
    $newContent = $profileContent -replace $pattern, ''

    # Clean up multiple consecutive blank lines
    $newContent = $newContent -replace '(\r?\n){3,}', "`n`n"

    # Add the new function code
    $newContent = $newContent.TrimEnd() + "`n`n" + $functionCode

    # Save updated profile
    Set-Content -Path $profilePath -Value $newContent -NoNewline
    Write-Host "[OK] Successfully updated 'dt' function in profile" -ForegroundColor Green

} elseif ($profileContent -match "function\s+dt\s*\{") {
    # Function exists but without markers - warn user
    Write-Host "[WARNING] Found existing 'dt' function without markers" -ForegroundColor Red
    Write-Host ""
    Write-Host "Your profile contains a 'dt' function that wasn't installed by this script." -ForegroundColor Yellow
    Write-Host "To update safely, please:" -ForegroundColor Yellow
    Write-Host "  1. Backup your profile: Copy-Item `$PROFILE `$PROFILE.backup" -ForegroundColor White
    Write-Host "  2. Remove the existing 'dt' function from: $profilePath" -ForegroundColor White
    Write-Host "  3. Run this installer again" -ForegroundColor White
    Write-Host ""

    $response = Read-Host "Do you want to APPEND the new function anyway? (yes/no)"
    if ($response -eq "yes") {
        Add-Content -Path $profilePath -Value "`n$functionCode"
        Write-Host "[OK] Appended new 'dt' function to profile" -ForegroundColor Green
        Write-Host "[WARNING] You may have duplicate 'dt' functions - please review your profile" -ForegroundColor Yellow
    } else {
        Write-Host "Installation cancelled" -ForegroundColor Red
        exit 1
    }

} else {
    # Fresh installation
    Write-Host "Installing 'dt' function to profile..." -ForegroundColor Green
    Add-Content -Path $profilePath -Value "`n$functionCode"
    Write-Host "[OK] Successfully installed 'dt' function" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== Next Steps ===" -ForegroundColor Cyan
Write-Host "1. Restart PowerShell or run: . `$PROFILE" -ForegroundColor White
Write-Host "2. Test with: dt --version" -ForegroundColor White
Write-Host ""
Write-Host "Profile location: $profilePath" -ForegroundColor Gray
