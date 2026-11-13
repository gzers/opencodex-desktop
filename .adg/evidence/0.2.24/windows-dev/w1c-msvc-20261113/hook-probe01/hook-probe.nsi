OutFile "C:\Users\15119\.codex\worktrees\official299-windows-evidence\docs-governance\.adg\evidence\0.2.24\windows-dev\w1c-msvc-20261113\hook-probe01\hook-probe.exe"
!include "C:\Users\15119\.codex\worktrees\024-windows-native\code\apps\desktop\tauri\windows\installer-hooks.nsh"
Unicode true
RequestExecutionLevel user
SilentInstall silent
Name "OpenCodeX W1-C preuninstall-only probe"
InstallDir "placeholder"
!include LogicLib.nsh
!include FileFunc.nsh
Var UpdateMode
Var DeleteAppDataCheckboxState
Function .onInit
  StrCpy $UpdateMode 0
  StrCpy $DeleteAppDataCheckboxState 0
  ${GetOptions} $CMDLINE "/UPDATE" $0
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}
FunctionEnd
Section
  !insertmacro NSIS_HOOK_PREUNINSTALL
  FileOpen $0 "$INSTDIR\hook-passed.txt" w
  FileWrite $0 "hook permitted continuation; no deletion performed"
  FileClose $0
  SetErrorLevel 0
SectionEnd
