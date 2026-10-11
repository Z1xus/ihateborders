@echo off
rem pins the msvc build tools

set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "COMPONENT=Microsoft.VisualStudio.Component.VC.14.44.17.14.x86.x64"
for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires %COMPONENT% -property installationPath`) do set "VS=%%i"
if not defined VS exit /b 1

call "%VS%\VC\Auxiliary\Build\vcvarsall.bat" x64 10.0.26100.0 -vcvars_ver=14.44.35207
if errorlevel 1 exit /b 1
if not "%VCToolsVersion%"=="14.44.35207" exit /b 1

echo PATH=%PATH%>> "%GITHUB_ENV%"
echo LIB=%LIB%>> "%GITHUB_ENV%"
echo INCLUDE=%INCLUDE%>> "%GITHUB_ENV%"
echo VCINSTALLDIR=%VCINSTALLDIR%>> "%GITHUB_ENV%"
echo VCToolsVersion=%VCToolsVersion%>> "%GITHUB_ENV%"
echo VCToolsInstallDir=%VCToolsInstallDir%>> "%GITHUB_ENV%"
echo WindowsSdkDir=%WindowsSdkDir%>> "%GITHUB_ENV%"
echo WindowsSDKVersion=%WindowsSDKVersion%>> "%GITHUB_ENV%"
