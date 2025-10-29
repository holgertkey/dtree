# install-windows-binary.ps1
# Installs dtree to user's bin directory and sets up wrapper

param(
    [string]$InstallPath = "$env:USERPROFILE\bin"
)

Write-Host "=== dtree Windows Installation ===" -ForegroundColor Cyan
Write-Host ""

# Create bin directory if it doesn't exist
if (-not (Test-Path $InstallPath)) {
    Write-Host "Creating directory: $InstallPath" -ForegroundColor Yellow
    New-Item -Path $InstallPath -ItemType Directory -Force | Out-Null
    Write-Host "[OK] Created directory: $InstallPath" -ForegroundColor Green
} else {
    Write-Host "[OK] Directory already exists: $InstallPath" -ForegroundColor Green
}

# Check if cargo project exists
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Cargo.toml not found! Please run this script from the dtree project directory."
    exit 1
}

# Build release binary
Write-Host ""
Write-Host "Building dtree..." -ForegroundColor Yellow
$buildResult = cargo build --release 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed!"
    Write-Host $buildResult -ForegroundColor Red
    exit 1
}
Write-Host "[OK] Build successful" -ForegroundColor Green

# Copy binary
$sourcePath = "target\release\dtree.exe"
$destPath = Join-Path $InstallPath "dtree.exe"

if (-not (Test-Path $sourcePath)) {
    Write-Error "Built binary not found at: $sourcePath"
    exit 1
}

Write-Host ""
Write-Host "Installing binary..." -ForegroundColor Yellow
Copy-Item $sourcePath $destPath -Force
Write-Host "[OK] Installed dtree.exe to $destPath" -ForegroundColor Green

# Check binary size
$binarySize = (Get-Item $destPath).Length / 1MB
Write-Host "[OK] Binary size: $([math]::Round($binarySize, 2)) MB" -ForegroundColor Cyan

# Add to PATH if not already there
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallPath*") {
    Write-Host ""
    Write-Host "Adding $InstallPath to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$currentPath;$InstallPath",
        "User"
    )
    Write-Host "[OK] Added to PATH" -ForegroundColor Green
    Write-Host "  Note: Restart terminal for PATH changes to take effect" -ForegroundColor Yellow
} else {
    Write-Host "[OK] $InstallPath already in PATH" -ForegroundColor Green
}

# Test installation
Write-Host ""
Write-Host "Testing installation..." -ForegroundColor Yellow
try {
    $version = & $destPath --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] Installation test successful: $version" -ForegroundColor Green
    } else {
        Write-Warning "Installation test failed"
    }
} catch {
    Write-Warning "Could not test installation: $_"
}

# Set up PowerShell wrapper
Write-Host ""
Write-Host "Setting up PowerShell wrapper..." -ForegroundColor Yellow
if (Test-Path "install-windows-wrapper.ps1") {
    try {
        & .\install-windows-wrapper.ps1
        Write-Host "[OK] PowerShell wrapper setup completed" -ForegroundColor Green
    } catch {
        Write-Warning "PowerShell wrapper setup failed: $_"
        Write-Host "You can run it manually later: .\install-windows-wrapper.ps1" -ForegroundColor Yellow
    }
} else {
    Write-Warning "install-windows-wrapper.ps1 not found. PowerShell wrapper not installed."
}

Write-Host ""
Write-Host "=== Installation Complete! ===" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Restart your terminal (to pick up PATH changes)" -ForegroundColor White
Write-Host "2. Run: dt --help" -ForegroundColor White
Write-Host "3. Try: dt (to open interactive tree)" -ForegroundColor White
Write-Host "4. Create bookmarks: dt -bm add myproject" -ForegroundColor White
Write-Host ""
Write-Host "Enjoy using dtree!" -ForegroundColor Green