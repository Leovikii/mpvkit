; New installations separate application binaries from player data. Preserve /D and existing installs.
!define MUI_CUSTOMFUNCTION_GUIINIT MpvKitDefaultDirectory
Function MpvKitDefaultDirectory
  Push $0
  Push $1
  ClearErrors
  ; NSIS omits its /D argument from $CMDLINE; read the original command line.
  System::Call 'kernel32::GetCommandLineW() w.r0'
  ${GetOptions} $0 "/D=" $0
  IfErrors 0 mpvkit_directory_done
  ReadRegStr $1 HKCU "Software\github\MpvKit" ""
  StrCmp $1 "" 0 mpvkit_directory_done
  StrCmp $INSTDIR "$LOCALAPPDATA\MpvKit" 0 mpvkit_directory_done
  StrCpy $INSTDIR "$LOCALAPPDATA\Programs\MpvKit"
  mpvkit_directory_done:
  Pop $1
  Pop $0
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  IfSilent 0 +2
  Call MpvKitDefaultDirectory
  SetOutPath $INSTDIR
!macroend

; App uninstall resolves the selected target through the same stable CLI.
!macro NSIS_HOOK_PREUNINSTALL
  ClearErrors
  ExecWait '"$INSTDIR\mpvkit.exe" --check-uninstall' $0
  IfErrors mpvkit_uninstall_blocked
  IntCmp $0 0 mpvkit_uninstall_ready mpvkit_uninstall_blocked mpvkit_uninstall_blocked
  mpvkit_uninstall_blocked:
    MessageBox MB_OK|MB_ICONEXCLAMATION "请先打开 MpvKit，恢复未完成的系统集成操作并移除打开方式登记，然后重试卸载。目标状态不可读取时也会停止。播放器、配置与备份不会被此卸载器删除。" /SD IDOK
    Abort
  mpvkit_uninstall_ready:
!macroend
