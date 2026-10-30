@echo off
setlocal
echo Avvio installazione User-Level di Accent Markdown Editor per Windows 11...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-user.ps1"
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Si e' verificato un errore durante l'installazione.
    pause
    exit /b %ERRORLEVEL%
)
echo.
echo Installazione completata. Premi un tasto per chiudere.
pause >nul
