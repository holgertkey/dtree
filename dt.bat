@echo off
REM dt.bat - Cmd.exe wrapper for dtree
REM Provides cd integration similar to bash dt() function
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
    REM Run dtree and capture output
    for /f "delims=" %%i in ('dtree.exe 2^>nul') do set "RESULT=%%i"

    if !ERRORLEVEL! EQU 0 (
        if defined RESULT (
            if exist "!RESULT!" (
                endlocal & set "DTREE_PREV_DIR=%CD%" & cd /d "!RESULT!"
                exit /b 0
            )
        )
    )
    endlocal
    exit /b 1
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

REM Navigation mode - pass all arguments to dtree
for /f "delims=" %%i in ('dtree.exe %* 2^>nul') do set "RESULT=%%i"

if !ERRORLEVEL! NEQ 0 (
    endlocal
    exit /b 1
)

if defined RESULT (
    if exist "!RESULT!" (
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
REM View mode: convert relative path to absolute if needed
set "FILE_PATH=%~2"

if "%FILE_PATH%"=="" (
    echo dt: -v requires a file path >&2
    endlocal
    exit /b 1
)

REM Convert to absolute path if relative
if not "%FILE_PATH:~1,1%"==":" (
    set "FILE_PATH=%CD%\%FILE_PATH%"
)

REM Run dtree in view mode and capture possible directory output
for /f "delims=" %%i in ('dtree.exe %~1 "!FILE_PATH!" 2^>nul') do set "RESULT=%%i"

if !ERRORLEVEL! NEQ 0 (
    endlocal
    exit /b 1
)

REM If dtree returned a directory, cd into it
if defined RESULT (
    if exist "!RESULT!\" (
        endlocal & set "DTREE_PREV_DIR=%CD%" & cd /d "!RESULT!"
        exit /b 0
    )
)

endlocal
exit /b 0
