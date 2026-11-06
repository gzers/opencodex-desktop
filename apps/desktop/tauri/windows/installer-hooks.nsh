; Program files belong to I; user data belongs to the current SID's U/desktop.
; L0 supplies the NSIS/WiX default-I template and offline runtime deployment.
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
