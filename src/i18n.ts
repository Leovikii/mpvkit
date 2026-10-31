export const messages = {
  back: ["返回", "Back"],
  openLogs: ["打开日志文件夹", "Open logs folder"],
  githubRepository: ["在 GitHub 查看 MpvKit 仓库", "View MpvKit on GitHub"],
  logsUnavailable: ["日志写入不可用，本次操作不受影响。", "Logging is unavailable. Operations are unaffected."],
  installWithCore: ["未找到 MPV，将先安装 MPV，再安装播放配置。", "MPV is missing. Install MPV first, then playback configuration."],
  installBundleHint: ["安装播放配置和所需组件。", "Install playback configuration and required components."],
  nativePackageHint: ["在包管理器中完成操作和询问。", "Complete the operation and prompts in the package manager."],
  alsoRemoveCore: ["同时卸载 MPV", "Also uninstall MPV"],
  uninstallAction: ["卸载", "Uninstall"],
  waitingPackageManager: ["等待包管理器…", "Waiting for package manager…"],
  removingConfiguration: ["删除配置中…", "Removing configuration…"],
  coreResult: ["MPV 核心", "MPV core"],
  stageFailed: ["未完成", "Incomplete"],
  exitCode: ["退出码：{code}", "Exit code: {code}"],
  coreStillInstalled: ["检测到 MPV 已安装", "MPV is installed"],
  coreNotDetected: ["未检测到 MPV", "MPV not detected"],
  quickInstall: ["一键安装", "Install all"],
  repairInstallation: ["修复安装", "Repair installation"],
  installingComponents: ["正在安装组件…", "Installing components…"],
  completedItems: ["已完成：{items}", "Completed: {items}"],
  partialInstallation: ["MPV 已安装，组件未完成。请重新准备剩余组件。", "MPV is installed; components are incomplete. Prepare the remaining components again."],
  coreUnusable: ["MPV 核心不可用，请检查版本和系统安装状态。", "MPV is unavailable. Check its version and system installation."],
  prepareAgain: ["重新准备", "Prepare again"],
  exitApp: ["退出 MpvKit？", "Exit MpvKit?"],
  executingTask: ["正在执行，请稍候…", "Working, please wait…"],
  mpvAmbiguous: ["发现多个 MPV 安装位置，请核对安装位置后重试。", "Multiple MPV installations found. Check the installation paths and retry."],
  appUpdate: ["应用更新", "Application updates"],
  releaseNotes: ["更新日志", "Release notes"],
  previewUpdates: ["接收预览版更新", "Include preview releases"],
  checkingUpdate: ["正在检查", "Checking"],
  viewUpdate: ["查看更新", "View update"],
  updateCheckFailed: ["更新检查失败，可重试。", "Update check failed. Try again."],
  noStableRelease: ["尚无稳定版发布", "No stable release published yet"],
  newAppVersion: ["发现新版本 {version}", "Version {version} available"],
  noReleaseNotes: ["此版本未提供更新日志。", "No release notes provided."],
  updateDraft: ["请先保存或重置草稿。", "Save or reset your draft first."],
  updateExit: ["安装完成后将自动重启 MpvKit。", "MpvKit will restart automatically after installation."],
  later: ["稍后", "Later"],
  downloadUpdate: ["下载更新", "Download"],
  installationPreview: ["安装预览", "Installation preview"],
  installUpdate: ["安装", "Install"],
  downloadingUpdate: ["下载中…", "Downloading…"],
  verifyingUpdate: ["校验中…", "Verifying…"],
  preparingUpdate: ["准备中…", "Preparing…"],
  installingUpdate: ["安装中…", "Installing…"],
  authorizingUpdate: ["等待系统授权或安装…", "Waiting for authorization or installation…"],
  updateComplete: ["已更新，正在重启…", "Updated. Restarting…"],
  updateCancelled: ["授权已取消，未安装。", "Authorization cancelled. Nothing installed."],
  updatePackageFailure: ["安装失败，请检查系统授权及包管理器。", "Installation failed. Check authorization and the package manager."],
  updatePackageUnmanaged: ["请先通过 Pacman 安装 MpvKit。", "Install MpvKit through Pacman first."],
  getWinget: ["获取应用安装程序", "Get App Installer"],
  installCore: ["安装", "Install"],
  manageCore: ["管理", "Manage"],
  linuxCoreManaged: ["由 Pacman 管理", "Managed by Pacman"],
  windowsCoreManaged: ["由 WinGet 管理", "Managed by WinGet"],
  upgradeCore: ["更新", "Update"],
  upToDate: ["暂无更新", "No updates available"],
  overwriteHint: ["保存将覆盖管理区域内的手改，无历史备份；区域外内容保留。", "Saving replaces edits in the managed section without backup. Other content is kept."],
  wingetMissing: ["未找到 WinGet，请安装或更新 Microsoft 应用安装程序后重试。", "WinGet is unavailable. Install or update Microsoft App Installer, then retry."],
  wingetFailure: ["WinGet 操作未完成，请检查系统授权、源、网络和安装状态后重试。", "WinGet did not complete. Check administrator approval, sources, network and installation status, then retry."],
  discardDraft: ["未保存的配置将丢失。", "Unsaved configuration will be lost."],
  discardAndClose: ["放弃并退出", "Discard and exit"],
  saveConfig: ["保存配置", "Save configuration"],
  previewConfig: ["预览配置", "Preview configuration"],
  configFile: ["配置文件", "Configuration file"],
  retainedFile: ["保留", "Retained"],
  saved: ["已保存", "Saved"],
  unsaved: ["未保存", "Unsaved"],
  openConfigDirectory: ["打开配置目录", "Open configuration folder"],
  openDirectory: ["打开", "Open"],
  applyChanges: ["应用更改","Apply changes"],
  fileChanges: ["文件变更（{count}）","File changes ({count})"],
  writeHint: ["写入以下位置，请核对文件变更；不保留历史备份。","Review changes at this location. No historical backups are kept."],
  sources: ["组件来源","Component sources"],
  balancedHint: ["推荐日常使用，兼顾画质与性能。","Recommended for everyday use."],
  powerHint: ["降低渲染负担，适合低性能设备。","Lower rendering load for less powerful devices."],
  qualityHint: ["推荐独立显卡使用，卡顿时请降低档位。","A dedicated GPU is recommended; lower quality if playback stutters."],
  configuration: ["播放配置","Configuration"],
  uninstallHint: ["将删除此 MPV 配置目录及全部内容，包括手改、其他脚本和续播记录。", "Delete this MPV configuration folder and all contents, including edits, other scripts and playback positions."],
  installConfig: ["安装配置", "Install configuration"],
  configState: ["播放配置", "Playback configuration"],
  modifiedAt: ["修改时间","Modified"],
  unreadableTime: ["无法读取", "Unavailable"],
  checkUpdates: ["检查更新", "Check for updates"],

  componentDetails: ["组件明细", "Component details"],
  uninstallCore: ["卸载核心", "Uninstall core"],
  uninstallCoreHint: ["通过系统包管理器卸载 MPV，其他使用该核心的程序也会受影响。配置保留；可能需要系统授权。", "Uninstall MPV with the system package manager. Other applications using it are affected. Configuration is retained; system authorization may be required."],
  resetConfig: ["重置配置","Reset configuration"],
  resetConfigHint: ["恢复默认配置、快捷键和脚本选项，覆盖手改，无历史备份。", "Restore default configuration, keys and script options. Edits are replaced without backup."],
  manualConfig: ["配置已有手动修改；此处显示上次保存的设置。", "Configuration was edited manually; these are the last saved settings."],
  beforeConfig: ["修改前", "Before"],
  afterConfig: ["修改后", "After"],
  subtitleFont: ["霞鹜文楷屏幕阅读版", "LXGW WenKai Screen"],

  autoPlaylist: ["自动连播", "Auto continue"],
  autoPlaylistHint: ["按文件名顺序载入同文件夹的音视频并连播。", "Load audio and video from the same folder in filename order."],
  endBehavior: ["全部播放结束后", "After all playback ends"],
  closePlayer: ["关闭播放器", "Close player"],
  keepWindow: ["保留窗口", "Keep window open"],

  working: ["处理中…","Working…"],
  status: ["状态", "Status"],
  playback: ["播放设置", "Playback settings"],
  maintenance: ["维护", "Maintenance"],
  settings: ["设置","Settings"],


  advanced: ["高级选项", "Advanced options"],

  extras: ["播放增强", "Playback extras"],
  ready: ["已就绪", "Ready"],
  missing: ["需要修复", "Needs repair"],
  notInstalled: ["未安装", "Not installed"],
  download: ["下载组件", "Download components"],
  install: ["安装", "Install"],
  details: ["查看详情", "View details"],
  cancel: ["取消", "Cancel"],
  close: ["关闭", "Close"],
  retry: ["重试", "Retry"],
  preview: ["确认变更", "Review changes"],
  conflict: ["文件已被修改，未覆盖。请先处理冲突。", "Files have changed and will not be overwritten. Resolve the conflicts first."],
  add: ["新增", "Add"],
  update: ["替换", "Replace"],
  remove: ["移除", "Remove"],
  done: ["操作完成。", "Done."],
  needPlayer: ["请先安装播放器。", "Install the player to continue."],
  downloadCancelled: ["已取消，安装未改变。", "Cancelled. Your installation was not changed."],
  auto: ["自动（推荐）","Auto (recommended)"],
  quality: ["画质","Quality"],
  balanced: ["均衡", "Balanced"],
  powerSaving: ["节能","Efficient"],
  highQuality: ["高画质", "High quality"],
  hdr: ["HDR 输出", "HDR output"],
  hdrOn: ["强制开启","Force on"],
  hdrOff: ["关闭（SDR）","Off (SDR)"],
  hdrHint: ["需支持 HDR 的屏幕，并在系统中开启 HDR。","Requires an HDR display with system HDR enabled."],
  defaultApps: ["设置默认应用", "Open Default apps"],
  recoverHint: ["恢复上次未完成的操作。", "Recover the unfinished operation."],
  recover: ["恢复操作","Recover"],
  recovery: ["需要恢复", "Recovery required"],
  uninstall: ["一键卸载", "Uninstall all"],
  uiLanguage: ["界面语言", "Display language"],
  system: ["跟随系统", "Use system setting"],
  theme: ["明暗模式", "Appearance mode"],
  light: ["浅色", "Light"],
  dark: ["深色", "Dark"],
  webOnly: ["网页模式无法操作本机，请使用桌面应用。", "Local operations require the desktop app."],
  loading: ["正在检查…", "Checking…"],
  operationFailed: ["操作未完成。请查看详情后重试。", "The operation could not complete. Check the details and try again."],
  networkError: ["下载失败。请检查网络后重试。", "Download failed. Check your connection and try again."],
  stale: ["状态已变化，请重新预览。", "The state changed. Review a new preview."],
  invalidPath: ["无法使用配置目录，请检查路径。", "Configuration path is unavailable. Check the directory."],

} as const;
export type Key = keyof typeof messages;
export type Locale = "zh-Hans" | "en";
export function resolveLocale(choice: string, languages: readonly string[]): Locale {
  if (choice === "zh-Hans" || choice === "en") return choice;
  for (const original of languages) {
    const lang = original.toLowerCase();
    if (lang === "zh" || lang.startsWith("zh-hans") || /^zh-(cn|sg)(-|$)/.test(lang)) return "zh-Hans";
    if (lang === "en" || lang.startsWith("en-")) return "en";
  }
  return "en";
}
export function translator(locale: Locale) {
  return (key: Key, params: Record<string, string | number> = {}) => messages[key][locale === "zh-Hans" ? 0 : 1].replace(/\{(\w+)\}/g, (_, k: string) => String(params[k] ?? ("{" + k + "}")));
}
export function errorKey(error: unknown): Key {
  const code = (error as {code?: string})?.code ?? "";
  if (code === "installation_partial") return "partialInstallation";
  if (code === "core_unusable") return "coreUnusable";
  if (code === "mpv_ambiguous") return "mpvAmbiguous";
  if (code === "winget_missing") return "wingetMissing";
  if (code === "update_authorization_cancelled") return "updateCancelled";
  if (code === "update_package_unmanaged") return "updatePackageUnmanaged";
  if (code.startsWith("update_package_") || code === "update_authorization_failed") return "updatePackageFailure";
  if (code.startsWith("winget_") || code === "mpv_install_incomplete") return "wingetFailure";
  if (code === "cancelled") return "downloadCancelled";
  if (code.includes("stale") || code.includes("changed")) return "stale";
  if (code.includes("conflict")) return "conflict";
  if (code === "components_required" || code === "not_installed") return "needPlayer";
  if (code.includes("download") || code.includes("network")) return "networkError";
  if (code.includes("path") || code.includes("target_root")) return "invalidPath";
  return "operationFailed";
}
