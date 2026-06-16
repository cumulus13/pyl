@echo off
setlocal

echo [pyl] Building...

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust/Cargo not found.
    echo Download from: https://www.rust-lang.org/tools/install
    exit /b 1
)

cargo build --release
if %errorlevel% neq 0 (
    echo ERROR: Build failed.
    exit /b 1
)

echo.
echo  Build OK:  target\release\pyl.exe
echo.
echo  Quick start:
echo    copy target\release\pyl.exe C:\Windows\pyl.exe
echo.
echo  Usage examples:
echo    pyl -pypy -V
echo    pyl -pypy3.11 script.py
echo    pyl -3.13 -m pip install rich
echo    pyl alias set pp PyPy/3.11
echo    pyl -pp script.py
echo    pyl -0
echo.
