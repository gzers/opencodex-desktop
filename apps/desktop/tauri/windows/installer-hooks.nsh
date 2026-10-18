; Default uninstall and updater replacement preserve every data root.
; Run before Tauri removes ocxd.exe or the main executable.
!macro NSIS_HOOK_PREUNINSTALL
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
    DeleteRegKey SHCTX "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty SHCTX "${MANUKEY}"
    DeleteRegValue HKCU "${MANUPRODUCTKEY}" "Installer Language"
    DeleteRegKey /ifempty HKCU "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty HKCU "${MANUKEY}"
    ; The helper removes only validated manager roots and the matching startup
    ; pointer. Do not run Tauri's independent recursive AppData deletion after it.
    StrCpy $DeleteAppDataCheckboxState 0
  ${EndIf}
!macroend
