# replace-wrapper.ps1
# Simple script to replace dt function in PowerShell profile

$profilePath = $PROFILE

# Check if profile exists
if (-not (Test-Path $profilePath)) {
    Write-Output "Creating new PowerShell profile..."
    New-Item -Path $profilePath -ItemType File -Force | Out-Null
}

# Read current profile content
$content = ""
if ((Get-Item $profilePath).Length -gt 0) {
    $content = Get-Content $profilePath -Raw
}

# Remove old dtree wrapper section if exists
if ($content -match '# dtree wrapper') {
    # Find the start of the comment and end of the function
    $pattern = '(?ms)# dtree wrapper.*?^}\s*$'
    $content = $content -replace $pattern, ''

    # Clean up extra blank lines
    $content = $content -replace '(\r?\n){3,}', "`r`n`r`n"

    Set-Content -Path $profilePath -Value $content.TrimEnd() -NoNewline
    Write-Output "Removed old dt function"
}

# Now add the new function by running install-windows.ps1
Write-Output "Installing new dt function..."
& "$PSScriptRoot\install-windows.ps1"
