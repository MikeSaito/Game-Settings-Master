; Diagnostics for isolated updater fixtures only. The production installer does
; not include these hooks. No hook launches the application being tested.
!macro NSIS_HOOK_POSTINSTALL
  Push $R6
  Push $R7
  Push $R8
  FileOpen $R7 "$INSTDIR\native-installer.log" a
  FileWrite $R7 "command: $CMDLINE$\r$\n"
  ${GetOptions} $CMDLINE "/ARGS" $R8
  FileWrite $R7 "relaunch arguments: $R8$\r$\n"
  nsis_tauri_utils::RunAsUser "$SYSDIR\cmd.exe" '/d /c echo shell-child>"$INSTDIR\native-shell-child.txt"'
  Pop $R6
  FileWrite $R7 "RunAsUser probe: $R6$\r$\n"
  FileClose $R7
  Pop $R8
  Pop $R7
  Pop $R6
  ClearErrors
!macroend
