# uninstall-windows.ps1
# Uninstalls dtree from Windows system
# Removes binary, wrappers, and optionally configuration

param(
    [string]$InstallPath = "$env:USERPROFILE\bin",
    [switch]$RemoveConfig,
    [switch]$RemoveFromPath,
    [switch]$Force
)

Write-Host "=== dtree Windows Uninstaller ===" -ForegroundColor Cyan
Write-Host ""

# Define markers for PowerShell wrapper
$START_MARKER = "# >>> dtree wrapper start <<<"
$END_MARKER = "# >>> dtree wrapper end <<<"

# Track what was removed
$removedItems = @()
$failedItems = @()

# Function to remove wrapper from PowerShell profile
function Remove-WrapperFromProfile {
    param(
        [string]$ProfilePath,
        [string]$ProfileName
    )

    Write-Host "Checking $ProfileName..." -ForegroundColor Cyan

    if (-not (Test-Path $ProfilePath)) {
        Write-Host "  Profile not found (nothing to remove)" -ForegroundColor Gray
        return $false
    }

    # Read profile content
    $profileContent = Get-Content -Path $ProfilePath -Raw -ErrorAction SilentlyContinue
    if (-not $profileContent) {
        Write-Host "  Profile is empty (nothing to remove)" -ForegroundColor Gray
        return $false
    }

    # Check if wrapper exists
    $hasStartMarker = $profileContent -match [regex]::Escape($START_MARKER)
    $hasEndMarker = $profileContent -match [regex]::Escape($END_MARKER)

    if (-not ($hasStartMarker -and $hasEndMarker)) {
        Write-Host "  dtree wrapper not found (nothing to remove)" -ForegroundColor Gray
        return $false
    }

    # Remove the wrapper code block
    Write-Host "  Removing dtree wrapper function..." -ForegroundColor Yellow
    $pattern = "(?s)$([regex]::Escape($START_MARKER)).*?$([regex]::Escape($END_MARKER))"
    $newContent = $profileContent -replace $pattern, ''

    # Clean up multiple consecutive blank lines
    $newContent = $newContent -replace '(\r?\n){3,}', "`n`n"
    $newContent = $newContent.TrimEnd()

    # Save updated profile
    try {
        Set-Content -Path $ProfilePath -Value $newContent -NoNewline -ErrorAction Stop
        Write-Host "  [OK] Removed 'dt' function from $ProfileName" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "  [ERROR] Failed to update profile: $_" -ForegroundColor Red
        return $false
    }
}

# 1. Remove PowerShell wrapper from profiles
Write-Host "Step 1: Removing PowerShell wrapper" -ForegroundColor Cyan
Write-Host ""

$profiles = @(
    @{
        Name = "Windows PowerShell 5.x"
        Path = "$HOME\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1"
        ExeName = "powershell.exe"
    },
    @{
        Name = "PowerShell Core 7.x+"
        Path = "$HOME\Documents\PowerShell\Microsoft.PowerShell_profile.ps1"
        ExeName = "pwsh.exe"
    }
)

foreach ($profile in $profiles) {
    $psExe = Get-Command $profile.ExeName -ErrorAction SilentlyContinue
    if (-not $psExe) {
        Write-Host "$($profile.Name) not installed (skipping)" -ForegroundColor Gray
        continue
    }

    $success = Remove-WrapperFromProfile -ProfilePath $profile.Path -ProfileName $profile.Name
    if ($success) {
        $removedItems += "PowerShell wrapper from $($profile.Name)"
    }
}

Write-Host ""

# 2. Remove binary files
Write-Host "Step 2: Removing binary files" -ForegroundColor Cyan
Write-Host ""

$filesToRemove = @(
    @{
        Path = Join-Path $InstallPath "dtree.exe"
        Name = "dtree.exe (main binary)"
    },
    @{
        Path = Join-Path $InstallPath "dt.bat"
        Name = "dt.bat (cmd.exe wrapper)"
    }
)

foreach ($file in $filesToRemove) {
    if (Test-Path $file.Path) {
        Write-Host "  Removing $($file.Name)..." -ForegroundColor Yellow
        try {
            Remove-Item $file.Path -Force -ErrorAction Stop
            Write-Host "  [OK] Removed: $($file.Path)" -ForegroundColor Green
            $removedItems += $file.Name
        } catch {
            Write-Host "  [ERROR] Failed to remove: $_" -ForegroundColor Red
            $failedItems += $file.Name
        }
    } else {
        Write-Host "  File not found: $($file.Path)" -ForegroundColor Gray
    }
}

Write-Host ""

# 3. Remove from PATH (optional)
if ($RemoveFromPath) {
    Write-Host "Step 3: Removing from PATH" -ForegroundColor Cyan
    Write-Host ""

    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -like "*$InstallPath*") {
        Write-Host "  Removing $InstallPath from PATH..." -ForegroundColor Yellow

        # Remove the path (handle both ;path and path; cases)
        $newPath = $currentPath -replace [regex]::Escape(";$InstallPath"), ""
        $newPath = $newPath -replace [regex]::Escape("$InstallPath;"), ""
        $newPath = $newPath -replace [regex]::Escape("$InstallPath"), ""

        try {
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Host "  [OK] Removed from PATH" -ForegroundColor Green
            Write-Host "  Note: Restart terminal for PATH changes to take effect" -ForegroundColor Yellow
            $removedItems += "PATH entry: $InstallPath"
        } catch {
            Write-Host "  [ERROR] Failed to update PATH: $_" -ForegroundColor Red
            $failedItems += "PATH entry"
        }
    } else {
        Write-Host "  $InstallPath not found in PATH" -ForegroundColor Gray
    }

    Write-Host ""
} else {
    Write-Host "Step 3: Keeping PATH entry (use -RemoveFromPath to remove)" -ForegroundColor Gray
    Write-Host ""
}

# 4. Remove configuration (optional)
$configDir = Join-Path $env:APPDATA "dtree"

if ($RemoveConfig) {
    Write-Host "Step 4: Removing configuration and data" -ForegroundColor Cyan
    Write-Host ""

    if (Test-Path $configDir) {
        # List what will be removed
        $configFiles = Get-ChildItem -Path $configDir -Recurse -File
        Write-Host "  Found configuration directory: $configDir" -ForegroundColor Yellow
        Write-Host "  Files to be removed:" -ForegroundColor Yellow
        foreach ($file in $configFiles) {
            Write-Host "    - $($file.Name)" -ForegroundColor Gray
        }
        Write-Host ""

        if (-not $Force) {
            $confirmation = Read-Host "  Are you sure you want to delete configuration? (yes/no)"
            if ($confirmation -ne "yes") {
                Write-Host "  [SKIPPED] Configuration kept" -ForegroundColor Yellow
                Write-Host ""
            } else {
                try {
                    Remove-Item $configDir -Recurse -Force -ErrorAction Stop
                    Write-Host "  [OK] Removed configuration directory" -ForegroundColor Green
                    $removedItems += "Configuration directory ($configDir)"
                    Write-Host ""
                } catch {
                    Write-Host "  [ERROR] Failed to remove configuration: $_" -ForegroundColor Red
                    $failedItems += "Configuration directory"
                    Write-Host ""
                }
            }
        } else {
            try {
                Remove-Item $configDir -Recurse -Force -ErrorAction Stop
                Write-Host "  [OK] Removed configuration directory (forced)" -ForegroundColor Green
                $removedItems += "Configuration directory ($configDir)"
                Write-Host ""
            } catch {
                Write-Host "  [ERROR] Failed to remove configuration: $_" -ForegroundColor Red
                $failedItems += "Configuration directory"
                Write-Host ""
            }
        }
    } else {
        Write-Host "  Configuration directory not found: $configDir" -ForegroundColor Gray
        Write-Host ""
    }
} else {
    Write-Host "Step 4: Keeping configuration (use -RemoveConfig to remove)" -ForegroundColor Gray
    if (Test-Path $configDir) {
        Write-Host "  Configuration location: $configDir" -ForegroundColor Gray
        Write-Host "  To remove manually: Remove-Item '$configDir' -Recurse -Force" -ForegroundColor Gray
    }
    Write-Host ""
}

# 5. Check if bin directory is empty
Write-Host "Step 5: Checking installation directory" -ForegroundColor Cyan
Write-Host ""

if (Test-Path $InstallPath) {
    $remainingFiles = Get-ChildItem -Path $InstallPath -File
    if ($remainingFiles.Count -eq 0) {
        Write-Host "  Directory is empty: $InstallPath" -ForegroundColor Gray
        if (-not $Force) {
            $removeDir = Read-Host "  Remove empty directory? (yes/no)"
            if ($removeDir -eq "yes") {
                try {
                    Remove-Item $InstallPath -Force -ErrorAction Stop
                    Write-Host "  [OK] Removed empty directory" -ForegroundColor Green
                    $removedItems += "Empty bin directory"
                } catch {
                    Write-Host "  [ERROR] Failed to remove directory: $_" -ForegroundColor Red
                }
            }
        }
    } else {
        Write-Host "  Directory contains other files ($($remainingFiles.Count) files)" -ForegroundColor Gray
        Write-Host "  Keeping directory: $InstallPath" -ForegroundColor Gray
    }
}

Write-Host ""

# Print summary
Write-Host "=== Uninstallation Summary ===" -ForegroundColor Cyan
Write-Host ""

if ($removedItems.Count -gt 0) {
    Write-Host "Successfully removed:" -ForegroundColor Green
    foreach ($item in $removedItems) {
        Write-Host "  [OK] $item" -ForegroundColor Green
    }
    Write-Host ""
}

if ($failedItems.Count -gt 0) {
    Write-Host "Failed to remove:" -ForegroundColor Red
    foreach ($item in $failedItems) {
        Write-Host "  [ERROR] $item" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "You may need to remove these manually with administrator privileges" -ForegroundColor Yellow
    Write-Host ""
}

if ($removedItems.Count -eq 0 -and $failedItems.Count -eq 0) {
    Write-Host "Nothing was removed (dtree may not be installed)" -ForegroundColor Yellow
    Write-Host ""
}

# Final notes
Write-Host "=== Next Steps ===" -ForegroundColor Cyan
Write-Host ""

if ($removedItems -match "PowerShell wrapper") {
    Write-Host "1. Restart PowerShell or reload profile to apply changes" -ForegroundColor White
}

if ($RemoveFromPath) {
    Write-Host "2. Restart terminal for PATH changes to take effect" -ForegroundColor White
}

if (-not $RemoveConfig -and (Test-Path $configDir)) {
    Write-Host "3. Configuration preserved at: $configDir" -ForegroundColor White
    Write-Host "   To remove: .\uninstall-windows.ps1 -RemoveConfig" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Uninstallation complete!" -ForegroundColor Green
Write-Host ""

# Exit with appropriate code
if ($failedItems.Count -gt 0) {
    exit 1
}
exit 0
