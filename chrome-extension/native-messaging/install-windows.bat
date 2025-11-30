@echo off
REM Install Native Messaging manifest for Windows

set MANIFEST_NAME=com.zoo.downloader.json
set HOST_NAME=com.zoo.downloader

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0

REM Default Chrome registry path
set CHROME_REG_PATH=HKCU\Software\Google\Chrome\NativeMessagingHosts\%HOST_NAME%

REM Prompt for Zoo Downloader executable path
set /p APP_PATH="Enter the full path to zoo-downloader.exe: "

REM Update manifest with correct path
powershell -Command "(gc '%SCRIPT_DIR%%MANIFEST_NAME%') -replace '/path/to/zoo-downloader', '%APP_PATH%' | Out-File -encoding ASCII '%SCRIPT_DIR%%MANIFEST_NAME%.tmp'"
move /y "%SCRIPT_DIR%%MANIFEST_NAME%.tmp" "%SCRIPT_DIR%%MANIFEST_NAME%"

REM Create registry entry
reg add "%CHROME_REG_PATH%" /ve /t REG_SZ /d "%SCRIPT_DIR%%MANIFEST_NAME%" /f

if %errorlevel% equ 0 (
    echo.
    echo Native Messaging host installed successfully!
    echo Manifest location: %SCRIPT_DIR%%MANIFEST_NAME%
    echo.
    echo Next steps:
    echo 1. Install the Chrome extension
    echo 2. Update the manifest with your extension ID
    echo 3. Restart Chrome
) else (
    echo.
    echo Installation failed. Please run this script as Administrator.
)

pause
