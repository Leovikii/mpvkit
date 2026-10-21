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
