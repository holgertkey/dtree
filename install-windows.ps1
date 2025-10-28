# install-windows.ps1
# PowerShell wrapper for dtree - similar to bash dt() function

function dt {
    param(
        [switch]$v,  # -v flag for view mode
        [Parameter(ValueFromRemainingArguments=$true)]
        [string[]]$Arguments
    )

    $prevDir = $PWD.Path

    # Handle -v flag: rebuild arguments array to include it
    if ($v) {
        $Arguments = @('-v') + $Arguments
    }

    # Handle dt without arguments → open interactive TUI
    if ($Arguments.Count -eq 0) {
        # dtree.exe uses stderr for TUI and stdout for output path
        # We capture only stdout (path) but let stderr (TUI) display normally
        $result = & dtree.exe
        if ($LASTEXITCODE -eq 0 -and $result -and (Test-Path $result)) {
            Set-Location $result
            $env:DTREE_PREV_DIR = $prevDir
        }
        return
    }

    # Handle dt - (return to previous directory)
    if ($Arguments.Count -eq 1 -and $Arguments[0] -eq "-") {
        if ($env:DTREE_PREV_DIR -and (Test-Path $env:DTREE_PREV_DIR)) {
            Set-Location $env:DTREE_PREV_DIR
            $env:DTREE_PREV_DIR = $prevDir
        } else {
            Write-Error "dt: no previous directory"
        }
        return
    }

    # Handle flags that should run dtree directly without cd
    if ($Arguments.Count -gt 0) {
        switch ($Arguments[0]) {
            {$_ -in "-h", "--help", "--version"} {
                & dtree.exe $Arguments
                return
            }
            "-bm" {
                & dtree.exe $Arguments
                return
            }
            "-v" {
                # View mode: need to resolve relative path to absolute
                if ($Arguments.Count -ge 2) {
                    $filePath = $Arguments[1]

                    # Convert relative path to absolute
                    if (-not [System.IO.Path]::IsPathRooted($filePath)) {
                        $filePath = Join-Path $PWD.Path $filePath
                    }

                    # Run dtree -v with absolute path
                    # Don't capture stderr - let TUI display
                    $result = & dtree.exe "-v" $filePath
                    $exitCode = $LASTEXITCODE

                    if ($exitCode -ne 0) {
                        return
                    }

                    # dtree may return a directory path to cd into
                    if ($result -and (Test-Path $result) -and (Test-Path $result -PathType Container)) {
                        Set-Location $result
                        $env:DTREE_PREV_DIR = $prevDir
                    }
                } else {
                    # No file specified, just pass through
                    & dtree.exe $Arguments
                }
                return
            }
        }
    }

    # Navigation mode: capture stdout (path) separately from stderr (errors)
    $result = & dtree.exe $Arguments 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -ne 0) {
        # Error occurred, dtree already printed to stderr
        return
    }

    $result = $result | Out-String
    $result = $result.Trim()

    # Only cd if result is a valid directory
    if ($result -and (Test-Path $result)) {
        Set-Location $result
        $env:DTREE_PREV_DIR = $prevDir
    }
}

# Auto-add to profile
$profilePath = $PROFILE
if (-not (Test-Path $profilePath)) {
    New-Item -Path $profilePath -ItemType File -Force | Out-Null
}

# Read this script to get the function definition
$functionCode = @"
function dt {
    param(
        [switch]`$v,  # -v flag for view mode
        [Parameter(ValueFromRemainingArguments=`$true)]
        [string[]]`$Arguments
    )

    `$prevDir = `$PWD.Path

    # Handle -v flag: rebuild arguments array to include it
    if (`$v) {
        `$Arguments = @('-v') + `$Arguments
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
            "-v" {
                # View mode: need to resolve relative path to absolute
                if (`$Arguments.Count -ge 2) {
                    `$filePath = `$Arguments[1]

                    # Convert relative path to absolute
                    if (-not [System.IO.Path]::IsPathRooted(`$filePath)) {
                        `$filePath = Join-Path `$PWD.Path `$filePath
                    }

                    # Run dtree -v with absolute path
                    # Don't capture stderr - let TUI display
                    `$result = & dtree.exe "-v" `$filePath
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
"@

if (-not (Select-String -Path $profilePath -Pattern "function dt" -Quiet)) {
    Write-Host "Adding 'dt' function to your PowerShell profile..." -ForegroundColor Green
    Add-Content -Path $profilePath -Value "`n# dtree wrapper`n$functionCode"
    Write-Host "Done! Restart PowerShell or run: . `$PROFILE" -ForegroundColor Yellow
} else {
    Write-Host "'dt' function already exists in profile" -ForegroundColor Cyan
}