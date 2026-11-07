; Program files belong to I; user data belongs to the current SID's U/desktop.
; CLI 2.11.4 templates nsis023.nsi / wix023.wxs select the contract default I.
; Silent NSIS skips the reinstall page. A 022 -> 023 default-root upgrade
; therefore removes the exact registered old app before writing the new app.
; This is program replacement only: no recursive data removal or migration.
!macro NSIS_HOOK_PREINSTALL
  Push $0
  Push $4
  Push $5
  Push $6
  ReadRegStr $4 HKCU "${UNINSTKEY}" "DisplayVersion"
  ${If} $4 == "0.2.22"
    ReadRegStr $5 HKCU "${UNINSTKEY}" "InstallLocation"
    ${If} $5 == '$\"$LOCALAPPDATA\${PRODUCTNAME}$\"'
    ${OrIf} $5 == "$LOCALAPPDATA\${PRODUCTNAME}"
      ReadRegStr $6 HKCU "${UNINSTKEY}" "UninstallString"
      ${If} $6 != '$\"$LOCALAPPDATA\${PRODUCTNAME}\uninstall.exe$\"'
        SetErrorLevel 1
        Abort "The registered 0.2.22 uninstaller identity differs. No data was migrated."
      ${EndIf}
      ClearErrors
      ExecWait '$6 /S /UPDATE _?=$LOCALAPPDATA\${PRODUCTNAME}' $0
      ${If} ${Errors}
      ${OrIf} $0 != 0
      ${OrIf} ${FileExists} "$LOCALAPPDATA\${PRODUCTNAME}\${MAINBINARYNAME}.exe"
        SetErrorLevel 1
        Abort "The old 0.2.22 program could not be removed. Retained data and installer state require review."
      ${EndIf}
    ${EndIf}
  ${EndIf}
  Pop $6
  Pop $5
  Pop $4
  Pop $0
!macroend

; Never use Tauri's independent legacy AppData deletion for this product.
; Run before Tauri removes the fixed, installed v2 cleanup helper.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode = 1
    ; A remembered checkbox must not make an updater delete current/legacy data.
    StrCpy $DeleteAppDataCheckboxState 0
  ${EndIf}
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    nsExec::ExecToStack '"$INSTDIR\ocxd.exe" --uninstall-manager-data --confirm'
    Pop $0
    Pop $1
    ${If} $0 != 0
      MessageBox MB_OK|MB_ICONSTOP "Manager data cleanup could not complete. Close OpenCodeX Desktop and verify its data directories, then try uninstalling again. The application files have been retained."
      SetErrorLevel 1
      Abort
    ${EndIf}
    ; The v2 helper validates the current user's fixed location and owned roots,
    ; and retains external HOME and unknown files. Product install registration
    ; is left to Tauri; this hook never removes legacy startup registry values.
    StrCpy $DeleteAppDataCheckboxState 0
  ${EndIf}
!macroend
