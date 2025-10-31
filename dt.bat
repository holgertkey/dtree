@echo off
REM dt.bat - Cmd.exe wrapper for dtree
REM Provides cd integration similar to bash dt() function
REM Matches PowerShell dt() function behavior
REM
REM Usage:
REM   dt              - Open interactive TUI
REM   dt path         - Navigate to path or bookmark
REM   dt -            - Return to previous directory
REM   dt --version    - Show version
REM   dt -bm ...      - Bookmark management
REM   dt -v file.txt  - View file

setlocal EnableDelayedExpansion

REM Save current directory
set "PREV_DIR=%CD%"

REM Handle dt without arguments - interactive TUI
if "%~1"=="" (
    REM Run dtree and capture stdout only (stderr displays normally for TUI)
    REM dtree uses stderr for TUI and stdout for output path
    for /f "delims=" %%i in ('dtree.exe') do set "RESULT=%%i"

    REM Check if command succeeded and result is a valid directory
    if !ERRORLEVEL! EQU 0 (
        if defined RESULT (
            if exist "!RESULT!\" (
                endlocal & set "DTREE_PREV_DIR=%CD%" & cd /d "!RESULT!"
                exit /b 0
            )
        )
    )
    endlocal
    exit /b 0
)

REM Handle dt - (return to previous directory)
if "%~1"=="-" (
    if not defined DTREE_PREV_DIR (
        echo dt: no previous directory >&2
        endlocal
        exit /b 1
    )
    if not exist "%DTREE_PREV_DIR%" (
        echo dt: previous directory does not exist >&2
        endlocal
        exit /b 1
    )
    endlocal & set "DTREE_PREV_DIR=%CD%" & cd /d "%DTREE_PREV_DIR%"
    exit /b 0
)

REM Handle flags that should run dtree directly without cd
if "%~1"=="-h" goto :passthrough
if "%~1"=="--help" goto :passthrough
if "%~1"=="--version" goto :passthrough
if "%~1"=="-bm" goto :passthrough

REM Handle -v or --view flags
if "%~1"=="-v" goto :view_mode
if "%~1"=="--view" goto :view_mode

REM Navigation mode - capture stdout (path) separately from stderr (errors)
REM Use temporary file to capture all output (stdout + stderr)
set "TEMP_FILE=%TEMP%\dtree_output_%RANDOM%.txt"
dtree.exe %* > "%TEMP_FILE%" 2>&1
set "EXIT_CODE=!ERRORLEVEL!"

REM Read result from temp file
set "RESULT="
if exist "%TEMP_FILE%" (
    for /f "usebackq delims=" %%i in ("%TEMP_FILE%") do (
        if not defined RESULT set "RESULT=%%i"
    )
    del "%TEMP_FILE%" 2>nul
)

REM Check exit code
if !EXIT_CODE! NEQ 0 (
    endlocal
    exit /b !EXIT_CODE!
)

REM Trim whitespace and check if result is a valid directory
if defined RESULT (
    REM Remove leading/trailing spaces
    for /f "tokens=* delims= " %%a in ("!RESULT!") do set "RESULT=%%a"

    REM Check if path exists and is a directory
    if exist "!RESULT!\" (
        endlocal & set "DTREE_PREV_DIR=%CD%" & cd /d "!RESULT!"
        exit /b 0
    )
)

endlocal
exit /b 0

:passthrough
REM Just run dtree directly and exit
endlocal
dtree.exe %*
exit /b %ERRORLEVEL%

:view_mode
REM View mode: need to resolve relative path to absolute
set "FILE_PATH=%~2"

REM If no file specified, just pass through to dtree
if "%FILE_PATH%"=="" (
    endlocal
    dtree.exe %*
    exit /b !ERRORLEVEL!
)

REM Convert relative path to absolute if needed
if not "%FILE_PATH:~1,1%"==":" (
    REM Not an absolute path (doesn't have drive letter), make it absolute
    set "FILE_PATH=%CD%\%FILE_PATH%"
)

REM Run dtree with absolute path (preserve original flag -v or --view)
REM Don't capture stderr - let TUI display normally
set "TEMP_FILE=%TEMP%\dtree_view_%RANDOM%.txt"
dtree.exe %~1 "!FILE_PATH!" > "%TEMP_FILE%" 2>&1
set "EXIT_CODE=!ERRORLEVEL!"

REM Read result from temp file
set "RESULT="
if exist "%TEMP_FILE%" (
    for /f "usebackq delims=" %%i in ("%TEMP_FILE%") do (
        if not defined RESULT set "RESULT=%%i"
    )
    del "%TEMP_FILE%" 2>nul
)

REM Check exit code
if !EXIT_CODE! NEQ 0 (
    endlocal
    exit /b !EXIT_CODE!
)

REM dtree may return a directory path to cd into
if defined RESULT (
    REM Trim whitespace
    for /f "tokens=* delims= " %%a in ("!RESULT!") do set "RESULT=%%a"

    REM Check if path exists and is a directory (Container)
    if exist "!RESULT!\" (
        endlocal & set "DTREE_PREV_DIR=%CD%" & cd /d "!RESULT!"
        exit /b 0
    )
)

endlocal
exit /b 0
