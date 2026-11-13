; Program files belong to the selected I; user data belongs to this SID's U/D/H.
; 024 has no automatic 022 removal/migration PREINSTALL hook. The former
; ExecWait of an old registered uninstaller could remove unrelated program state
; and is not part of the official024 supply contract. Default/custom I selection
; stays in the two templates; no recursive runtime/data removal is introduced.

; Never use Tauri's independent legacy AppData deletion for this product.
; Run before Tauri removes the fixed, installed v2 cleanup helper.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode = 1
    ; A remembered checkbox must not make an updater delete current/legacy data.
    StrCpy $DeleteAppDataCheckboxState 0
  ${EndIf}
  ${If} $UpdateMode <> 1
    ; Terminal ownership must be revoked before any program file is removed.
    ; A nonzero helper result stops uninstall and retains program/configuration.
    nsExec::ExecToStack '"$INSTDIR\ocxd.exe" --uninstall-terminal-cli --confirm'
    Pop $0
    Pop $1
    ${If} $0 != 0
      MessageBox MB_OK|MB_ICONSTOP "The current user's terminal entry could not be revoked. The application files and registration were retained." /SD IDOK
      SetErrorLevel 1
      Abort
    ${EndIf}
  ${EndIf}
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    nsExec::ExecToStack '"$INSTDIR\ocxd.exe" --uninstall-manager-data --confirm'
    Pop $0
    Pop $1
    ${If} $0 != 0
      MessageBox MB_OK|MB_ICONSTOP "Manager data cleanup could not complete. Close OpenCodeX Desktop and verify its data directories, then try uninstalling again. The application files have been retained." /SD IDOK
      SetErrorLevel 1
      Abort
    ${EndIf}
    ; The v2 helper validates the current user's fixed location and owned roots,
    ; and retains external HOME and unknown files. Product install registration
    ; is left to Tauri; this hook never removes legacy startup registry values.
    StrCpy $DeleteAppDataCheckboxState 0
  ${EndIf}
!macroend
