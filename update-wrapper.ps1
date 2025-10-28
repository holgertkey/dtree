# update-wrapper.ps1
# Updates the dt function in PowerShell profile

$profilePath = $PROFILE

if (-not (Test-Path $profilePath)) {
    Write-Host "PowerShell profile not found at: $profilePath" -ForegroundColor Red
    Write-Host "Run .\install-windows.ps1 first to create the profile" -ForegroundColor Yellow
    exit 1
}

Write-Host "Updating dt function in PowerShell profile..." -ForegroundColor Cyan
Write-Host "Profile location: $profilePath" -ForegroundColor Gray

# Read current profile
$profileContent = Get-Content $profilePath -Raw

# Check if function exists
if ($profileContent -notmatch 'function dt') {
    Write-Host "dt function not found in profile. Run .\install-windows.ps1 to install it." -ForegroundColor Yellow
    exit 1
}

# Remove old function (everything from "# dtree wrapper" or "function dt" to the next function or end)
$lines = Get-Content $profilePath
$newLines = @()
$inDtFunction = $false
$skipUntilBlankLine = $false

for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]

    # Start of dt function
    if ($line -match '^\s*#\s*dtree wrapper' -or ($line -match '^\s*function dt\s*{' -and -not $inDtFunction)) {
        $inDtFunction = $true
        $skipUntilBlankLine = $false
        continue
    }

    # Inside function - look for closing brace at the start of line
    if ($inDtFunction) {
        if ($line -match '^\}') {
            $inDtFunction = $false
            $skipUntilBlankLine = $true
            continue
        }
        continue
    }

    # Skip blank lines after function
    if ($skipUntilBlankLine) {
        if ($line -match '^\s*$') {
            continue
        } else {
            $skipUntilBlankLine = $false
        }
    }

    $newLines += $line
}

# Write cleaned profile
$newLines | Set-Content $profilePath

Write-Host "✓ Old dt function removed" -ForegroundColor Green

# Now run install-windows.ps1 to add the new version
Write-Host "`nAdding new dt function..." -ForegroundColor Cyan
& "$PSScriptRoot\install-windows.ps1"

Write-Host "`n✓ PowerShell wrapper updated successfully!" -ForegroundColor Green
Write-Host "Reload your profile with: . `$PROFILE" -ForegroundColor Yellow
