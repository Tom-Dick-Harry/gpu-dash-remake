@echo off
REM Package script for System Dashboard
echo Creating distribution package...

REM Create distribution directory
mkdir dist 2>nul
mkdir dist\bin 2>nul

REM Copy the main executable
copy target\release\sysinfo_dashboard.exe dist\bin\ /Y
echo Copied main executable

REM Copy necessary DLL files
copy win_sysinfo.dll dist\bin\ /Y
echo Copied required DLL files

REM Create a launcher script
echo @echo off > dist\SysDashboard.bat
echo cd bin >> dist\SysDashboard.bat
echo start /b sysinfo_dashboard.exe >> dist\SysDashboard.bat

REM Create a README file
echo System Dashboard Application > dist\README.txt
echo =========================== >> dist\README.txt
echo. >> dist\README.txt
echo This application provides real-time system telemetry monitoring. >> dist\README.txt
echo. >> dist\README.txt
echo To start the application: >> dist\README.txt
echo 1. Double-click the SysDashboard.bat file >> dist\README.txt
echo. >> dist\README.txt
echo The application includes: >> dist\README.txt
echo - A background server that collects system information >> dist\README.txt
echo - A GUI dashboard that displays the collected information >> dist\README.txt

echo Distribution package created successfully in the 'dist' folder!
echo Run 'dist\SysDashboard.bat' to start the application