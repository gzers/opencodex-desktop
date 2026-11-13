Unicode true
RequestExecutionLevel user
SilentInstall silent
!include LogicLib.nsh
!include FileFunc.nsh
!include x64.nsh
!define PRODUCTNAME "OpenCodeX Desktop"
!define INSTALLMODE "currentUser"
!define ARCH "x64"
!define DISPLAYLANGUAGESELECTOR "false"
!define PLACEHOLDER_INSTALL_DIR "placeholder\${PRODUCTNAME}"
Var PassiveMode
Var NoShortcutMode
Var UpdateMode
InstallDir "${PLACEHOLDER_INSTALL_DIR}"
OutFile "C:\Users\15119\.codex\worktrees\024-windows-native\code\test\windows024-native\target\installer-w1\nsis-paths02\path-probe.exe"
!define MANUPRODUCTKEY "Software\OpenCodeX.Native024.InstallerPath.113600"
!macro SetContext
  !if "${INSTALLMODE}" == "currentUser"
    SetShellVarContext current
  !else if "${INSTALLMODE}" == "perMachine"
    SetShellVarContext all
  !endif

  ${If} ${RunningX64}
    !if "${ARCH}" == "x64"
      SetRegView 64
    !else if "${ARCH}" == "arm64"
      SetRegView 64
    !else
      SetRegView 32
    !endif
  ${EndIf}
!macroend
Function .onInit
  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/NS" $NoShortcutMode
  ${IfNot} ${Errors}
    StrCpy $NoShortcutMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}

  !if "${DISPLAYLANGUAGESELECTOR}" == "true"
    !insertmacro MUI_LANGDLL_DISPLAY
  !endif

  !insertmacro SetContext

  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    ; Set default install location
    !if "${INSTALLMODE}" == "perMachine"
      ${If} ${RunningX64}
        !if "${ARCH}" == "x64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else if "${ARCH}" == "arm64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else
          StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
        !endif
      ${Else}
        StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
      ${EndIf}
    !else if "${INSTALLMODE}" == "currentUser"
      StrCpy $INSTDIR "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
    !endif

    Call RestorePreviousInstallLocation
  ${EndIf}


  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_INIT
  !endif
FunctionEnd
Function RestorePreviousInstallLocation
  ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
  ; An old default is not an explicit custom choice. Leave the new Programs
  ; default selected. 024 never invokes a legacy uninstaller from PREINSTALL.
  ${If} $4 != ""
  ${AndIf} $4 != "$LOCALAPPDATA\${PRODUCTNAME}"
    StrCpy $INSTDIR $4
  ${EndIf}
FunctionEnd
Section
FileOpen $0 "C:\Users\15119\.codex\worktrees\024-windows-native\code\test\windows024-native\target\installer-w1\nsis-paths02\resolved.txt" w
FileWriteUTF16LE $0 "$INSTDIR"
FileClose $0
SetErrorLevel 0
SectionEnd
