Unicode true
!include "MUI2.nsh"

!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!define ROOT "..\..\.."

Name "AetherCodex"
OutFile "${ROOT}\dist\windows\AetherCodex-${VERSION}-windows-x64-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\AetherCodex"
InstallDirRegKey HKCU "Software\AetherCodex" "InstallDir"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!define MUI_ICON "${ROOT}\apps\aethercodex-manager\src-tauri\icons\icon.ico"
!define MUI_UNICON "${ROOT}\apps\aethercodex-manager\src-tauri\icons\icon.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"

!macro RemoveLegacyEntrypointsBody
  Delete "$DESKTOP\Codex++.lnk"
  Delete "$DESKTOP\Codex++ 管理工具.lnk"
  Delete "$DESKTOP\Codex++ 绠＄悊宸ュ叿.lnk"
  Delete "$SMPROGRAMS\Codex++\Codex++.lnk"
  Delete "$SMPROGRAMS\Codex++\Codex++ 管理工具.lnk"
  Delete "$SMPROGRAMS\Codex++\Codex++ 绠＄悊宸ュ叿.lnk"
  Delete "$SMPROGRAMS\Codex++\卸载 Codex++.lnk"
  RMDir "$SMPROGRAMS\Codex++"
  Delete "$LOCALAPPDATA\Programs\CodexPlusPlus\codex-plus-plus.exe"
  Delete "$LOCALAPPDATA\Programs\CodexPlusPlus\codex-plus-plus-manager.exe"
  Delete "$LOCALAPPDATA\Programs\CodexPlusPlus\uninstall.exe"
  RMDir "$LOCALAPPDATA\Programs\CodexPlusPlus"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexPlusPlus"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Codex++"
  DeleteRegKey HKCU "Software\CodexPlusPlus"
  DeleteRegKey HKCU "Software\Codex++"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "CodexPlusPlusWatcher"
!macroend

Function RemoveLegacyEntrypoints
  !insertmacro RemoveLegacyEntrypointsBody
FunctionEnd

Function un.RemoveLegacyEntrypoints
  !insertmacro RemoveLegacyEntrypointsBody
FunctionEnd

Section "Install"
  SetOutPath "$INSTDIR"

  nsExec::ExecToLog 'taskkill /IM aethercodex.exe /F'
  Pop $0
  nsExec::ExecToLog 'taskkill /IM aethercodex-manager.exe /F'
  Pop $0
  nsExec::ExecToLog 'taskkill /IM codex-plus-plus.exe /F'
  Pop $0
  nsExec::ExecToLog 'taskkill /IM codex-plus-plus-manager.exe /F'
  Pop $0

  File "${ROOT}\dist\windows\app\aethercodex.exe"
  File "${ROOT}\dist\windows\app\aethercodex-manager.exe"

  ; Remove what releases under the previous product name left behind, so an
  ; upgrade does not leave a second set of shortcuts and uninstall entries.
  ; The mojibake name comes from a historic build that wrote the shortcut in
  ; the wrong encoding; the bytes on disk are what matter here.
  Call RemoveLegacyEntrypoints

  CreateShortcut "$DESKTOP\AetherCodex.lnk" "$INSTDIR\aethercodex.exe" "" "$INSTDIR\aethercodex.exe"
  CreateShortcut "$DESKTOP\AetherCodex 管理工具.lnk" "$INSTDIR\aethercodex-manager.exe" "" "$INSTDIR\aethercodex-manager.exe"
  CreateDirectory "$SMPROGRAMS\AetherCodex"
  CreateShortcut "$SMPROGRAMS\AetherCodex\AetherCodex.lnk" "$INSTDIR\aethercodex.exe" "" "$INSTDIR\aethercodex.exe"
  CreateShortcut "$SMPROGRAMS\AetherCodex\AetherCodex 管理工具.lnk" "$INSTDIR\aethercodex-manager.exe" "" "$INSTDIR\aethercodex-manager.exe"
  CreateShortcut "$SMPROGRAMS\AetherCodex\卸载 AetherCodex.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\aethercodex-manager.exe"

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\AetherCodex" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex" "DisplayName" "AetherCodex"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex" "Publisher" "Archai"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex" "DisplayIcon" "$INSTDIR\aethercodex-manager.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex" "UninstallString" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  nsExec::ExecToLog 'taskkill /IM aethercodex.exe /F'
  Pop $0
  nsExec::ExecToLog 'taskkill /IM aethercodex-manager.exe /F'
  Pop $0

  Delete "$DESKTOP\AetherCodex.lnk"
  Delete "$DESKTOP\AetherCodex 管理工具.lnk"
  Delete "$SMPROGRAMS\AetherCodex\AetherCodex.lnk"
  Delete "$SMPROGRAMS\AetherCodex\AetherCodex 管理工具.lnk"
  Delete "$SMPROGRAMS\AetherCodex\卸载 AetherCodex.lnk"
  Call un.RemoveLegacyEntrypoints
  RMDir "$SMPROGRAMS\AetherCodex"

  Delete "$INSTDIR\aethercodex.exe"
  Delete "$INSTDIR\aethercodex-manager.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex"
  DeleteRegKey HKCU "Software\AetherCodex"
SectionEnd
