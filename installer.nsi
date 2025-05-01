; System Dashboard NSIS Installer Script
; Modern UI
!include "MUI2.nsh"

; General settings
Name "System Dashboard"
OutFile "SysDashboard-Installer.exe"
InstallDir "$PROGRAMFILES\System Dashboard"
InstallDirRegKey HKLM "Software\System Dashboard" "Install_Dir"
RequestExecutionLevel admin ; Request admin privileges

; Interface settings
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE "English"

; Installation section
Section "Install"
  SetOutPath $INSTDIR
  
  ; Add the main program files
  File "target\release\sysinfo_dashboard.exe"
  File "win_sysinfo.dll"
  
  ; Visual C++ redistributable check - uncomment if needed
  ; Call vcredist_check
  
  ; Create start menu shortcuts
  CreateDirectory "$SMPROGRAMS\System Dashboard"
  CreateShortcut "$SMPROGRAMS\System Dashboard\System Dashboard.lnk" "$INSTDIR\sysinfo_dashboard.exe"
  CreateShortcut "$SMPROGRAMS\System Dashboard\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  
  ; Create desktop shortcut
  CreateShortcut "$DESKTOP\System Dashboard.lnk" "$INSTDIR\sysinfo_dashboard.exe"
  
  ; Write registry keys for uninstall
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\SystemDashboard" "DisplayName" "System Dashboard"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\SystemDashboard" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\SystemDashboard" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\SystemDashboard" "NoRepair" 1
  
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; Uninstall section
Section "Uninstall"
  ; Remove program files
  Delete "$INSTDIR\sysinfo_dashboard.exe"
  Delete "$INSTDIR\win_sysinfo.dll"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  
  ; Remove shortcuts
  Delete "$SMPROGRAMS\System Dashboard\System Dashboard.lnk"
  Delete "$SMPROGRAMS\System Dashboard\Uninstall.lnk"
  Delete "$DESKTOP\System Dashboard.lnk"
  RMDir "$SMPROGRAMS\System Dashboard"
  
  ; Remove registry keys
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\SystemDashboard"
  DeleteRegKey HKLM "Software\System Dashboard"
SectionEnd

; Function to check for Visual C++ redistributable - uncomment if needed
; Function vcredist_check
;   ReadRegStr $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
;   StrCmp $0 "1" vcredist_done
;   MessageBox MB_YESNO "This application requires the Visual C++ 2015-2019 Redistributable. Would you like to download and install it?" IDNO vcredist_done
;   ExecShell "open" "https://aka.ms/vs/16/release/vc_redist.x64.exe"
;   vcredist_done:
; FunctionEnd