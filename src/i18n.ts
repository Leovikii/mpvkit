export const messages = {
  oldUnavailable: ["原目录不可用，将保留其文件和打开方式。重新登记前可能需要接回原目录并移除旧登记。", "The old folder is unavailable. Its files and Open with entry will be preserved. Reconnect it to remove an old registration if needed."],
  working: ["正在处理，请稍候…", "Working, please wait…"], displayUnavailable: ["所选屏幕不可用", "Selected display is unavailable"],
  addLanguage: ["添加语言","Add language"], moveUp: ["上移","Move up"], moveDown: ["下移","Move down"],
  status: ["状态", "Status"], location: ["安装位置", "Location"], components: ["安装组件", "Install components"],
  playback: ["播放设置", "Playback settings"], integration: ["系统集成", "System integration"],
  updates: ["组件与更新", "Components & updates"], maintenance: ["维护", "Maintenance"], settings: ["设置", "Settings"],
  welcome: ["选择安装位置", "Choose an install location"], locationHint: ["播放器与配置保存在同一安装目录内。", "Your player and configuration share one install folder."],
  useLocation: ["继续", "Continue"], browse: ["浏览…", "Browse…"], back: ["上一步", "Back"], next: ["继续", "Continue"],
  change: ["更改", "Change"], directory: ["安装目录", "Install folder"], configDirectory: ["配置目录", "Configuration folder"],
  advanced: ["高级选项", "Advanced options"], customDirectory: ["自定义安装目录", "Custom install folder"],
  recommendedDefaults: ["默认设置适合大多数情况。仅在需要时调整。", "The defaults suit most setups. Adjust only if needed."],
  external: ["使用已有 MPV", "Use an existing MPV"], externalPath: ["MPV 程序路径", "MPV executable"],
  trustPlayer: ["仅选择你信任的 mpv.exe。检查版本时会运行它。", "Choose a trusted mpv.exe. Version checks will run it."],
  discover: ["查找本地 MPV", "Find local MPV"], choose: ["选择", "Select"], noCandidates: ["未找到候选，可手动选择。", "No candidates found. You can choose a file."],
  player: ["播放器", "Player"], extras: ["播放增强", "Playback extras"], extrasHint: ["播放界面和缩略图", "Player controls and thumbnails"],
  ready: ["已就绪", "Ready"], missing: ["需要修复", "Needs repair"], notInstalled: ["未安装", "Not installed"], waiting: ["待安装", "Not installed"],
  download: ["下载组件", "Download components"], install: ["安装", "Install"], downloadHint: ["下载并校验固定版本。确认安装后才更改播放环境。", "Download and verify pinned versions. Your player environment changes only after you confirm installation."],
  replace: ["允许备份并替换已有文件", "Back up and replace existing files"], replaceHint: ["替换范围将在确认前列出。已修改的受管理文件不会覆盖。", "Review the files before confirming. Changed managed files will not be overwritten."],
  details: ["查看详情", "View details"], cancel: ["取消", "Cancel"], close: ["关闭", "Close"], retry: ["重试", "Retry"],
  preview: ["确认变更", "Review changes"], confirm: ["确认", "Confirm"], destination: ["目标位置", "Destination"], source: ["当前位置", "Current location"],
  conflict: ["文件已被修改，未覆盖。请先处理冲突。", "Files have changed and will not be overwritten. Resolve the conflicts first."],
  file: ["文件", "File"], add: ["新增", "Add"], update: ["替换（保留备份）", "Replace (keep backup)"], remove: ["移除", "Remove"],
  noChanges: ["无需更改文件。", "No file changes needed."], done: ["操作完成。", "Done."],
  needPlayer: ["请先安装播放器。", "Install the player to continue."], damaged: ["请检查或修复安装。", "Check or repair this installation."],
  downloadBusy: ["正在准备组件", "Preparing components"], downloadCancelled: ["已取消，安装未改变。", "Cancelled. Your installation was not changed."],
  screen: ["播放屏幕", "Playback display"], auto: ["自动", "Automatic"], manual: ["手动分辨率", "Custom resolution"],
  width: ["宽度", "Width"], height: ["高度", "Height"], resolutionHint: ["仅用于推荐，不更改系统分辨率。", "Used for recommendations. Does not change your display resolution."],
  quality: ["画质档位", "Quality"], balanced: ["均衡", "Balanced"], powerSaving: ["省电", "Power saving"], highQuality: ["高画质", "High quality"],
  recommendation: ["建议：{profile}", "Recommended: {profile}"], conservative: ["根据供电与显示信息，采用保守设置。高画质可手动选择。", "A conservative starting point based on power and display information. High quality is available manually."],
  batteryReason: ["电池或节电模式：优先降低处理成本。", "Battery or power saver: reduce processing cost."],
  unknown: ["未知", "Unknown"], hdr: ["HDR 输出", "HDR output"], hdrOn: ["开启", "On"], hdrOff: ["关闭（转换为 SDR）", "Off (map to SDR)"],
  hdrHint: ["不切换系统 HDR。开启前需确认所选屏幕已启用 HDR。", "Does not switch system HDR. Turn HDR on for the selected display first."],
  audio: ["音轨语言顺序", "Preferred audio languages"], subtitles: ["字幕语言顺序", "Preferred subtitle languages"],
  mediaDefault: ["遵循媒体默认轨道", "Use the media default"], languageHint: ["填写语言代码，用逗号分隔。例如 zh,en,ja。留空使用媒体默认。", "Enter language codes separated by commas, e.g. en,zh,ja. Leave empty for media defaults."],
  detect: ["重新检测", "Detect again"], openConfig: ["打开配置目录", "Open configuration folder"], userConf: ["自定义请编辑 user.conf。它可以覆盖这些设置。", "Put custom settings in user.conf. They can override these settings."],
  saveContinue: ["保存并继续", "Save and continue"], save: ["保存", "Save"], restartPlayback: ["下次启动播放器时生效。", "Applies the next time the player starts."],
  addOpenWith: ["添加到打开方式", "Add to Open with"], integrationHint: ["让 Windows 使用此环境打开媒体文件。", "Let Windows open media files with this environment."],
  registered: ["已添加到打开方式", "Added to Open with"], notRegistered: ["尚未添加", "Not added"], changedRegistration: ["打开方式已失效", "Open with needs repair"],
  defaultApps: ["设置默认应用", "Open Default apps"], defaultHint: ["是否设为默认播放器，由你在 Windows 中选择。", "Choose your default player in Windows."],
  unregister: ["移除打开方式", "Remove from Open with"], finish: ["完成", "Finish"], recover: ["恢复未完成操作", "Recover interrupted operation"],
  recovery: ["需要恢复", "Recovery required"], recoveryHint: ["上次操作未完成。恢复后继续。", "The previous operation was interrupted. Recover it to continue."],
  openVideo: ["打开视频", "Open media"], media: ["媒体文件", "Media file"], play: ["播放", "Play"],
  checkAgain: ["重新检查", "Check again"], environment: ["播放环境", "Player environment"], dailyHint: ["安装、设置和更新集中在这里。", "Manage your player, settings and updates here."],
  restore: ["恢复上一版本", "Restore previous version"], uninstall: ["移除播放环境", "Remove player environment"],
  cleanup: ["清理下载与暂存", "Clean downloads and staging"], cleanLogs: ["清理旧日志", "Clean old logs"], diagnostics: ["导出诊断", "Export diagnostics"],
  exportPath: ["导出文件路径", "Export file path"], exportHint: ["先预览脱敏内容，再保存到本机。", "Review the redacted report before saving it locally."],
  dangerHint: ["仅处理记录中属于 MpvKit 的文件。用户文件和备份保留。", "Only files recorded as owned by MpvKit are affected. User files and backups are preserved."],
  language: ["语言", "Language"], uiLanguage: ["界面语言", "Display language"], system: ["跟随系统", "Use system setting"],
  appearance: ["外观", "Appearance"], theme: ["明暗模式", "Appearance mode"], light: ["浅色", "Light"], dark: ["深色", "Dark"],
  accent: ["强调色", "Accent color"], blue: ["蓝", "Blue"], teal: ["青", "Teal"], violet: ["紫", "Purple"], orange: ["橙", "Orange"],
  about: ["关于", "About"], version: ["版本", "Version"], copyright: ["版权与许可", "Copyright & license"], thirdParty: ["第三方声明", "Third-party notices"],
  repository: ["仓库", "Repository"], repositoryPending: ["仓库地址尚未配置", "Repository address is not configured"],
  releaseStatus: ["开发测试版；完整系统、设备与来源验收尚未完成。", "Development build. Full system, device and source verification is still pending."],
  webOnly: ["网页模式无法操作本机，请使用桌面应用。", "Local operations require the desktop app."], loading: ["正在检查…", "Checking…"],
  operationFailed: ["操作未完成。请查看详情后重试。", "The operation could not complete. Check the details and try again."],
  networkError: ["下载失败。请检查网络后重试。", "Download failed. Check your connection and try again."],
  stale: ["状态已变化，请重新预览。", "The state changed. Review a new preview."], invalidPath: ["无法使用此路径，请选择本地文件夹。", "This path is unavailable. Choose a local folder."],
  appSaveFailed: ["应用设置未保存，请重试。", "App settings were not saved. Try again."], hdrUnavailable: ["此屏幕的系统 HDR 尚未确认开启。请选择自动模式或启用系统 HDR 后重试。", "System HDR is not confirmed active on this display. Use Automatic, or enable system HDR and retry."],
  selected: ["已选择", "Selected"], setup: ["首次设置", "Setup"], progress: ["第 {current} / {total} 步", "Step {current} of {total}"],
  pathsPreserved: ["切换目录不移动或删除原有文件。", "Switching folders does not move or delete existing files."],
  coreCheck: ["组件状态", "Component status"], effectiveProfile: ["播放时保存的档位可能覆盖基线。", "A profile saved during playback may override this baseline."],
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
  if (code === "cancelled") return "downloadCancelled";
  if (code.includes("stale") || code.includes("changed")) return "stale";
  if (code.includes("conflict")) return "conflict";
  if (code === "components_required" || code === "not_installed") return "needPlayer";
  if (code === "hdr_unavailable") return "hdrUnavailable";
  if (code.includes("download") || code.includes("network")) return "networkError";
  if (code.includes("path") || code.includes("target_root")) return "invalidPath";
  return "operationFailed";
}
