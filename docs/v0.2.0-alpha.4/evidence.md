# alpha.4 本机调查与证据

日期：2026-10-28。源码基线 dev / `0499a4a` / 0.2.0-alpha.3；接手时 `git status --short` 无输出。前半记录 alpha.3 调查事实，后半记录 alpha.4 实施与验证；[plan.md](plan.md)记录交接。

## 用户实测问题

用户截图显示：

```json
{
  "code": "mpv_ambiguous",
  "message": "发现多个 MPV 安装位置，请整理后重试；文件未修改。",
  "phase": "system-mpv",
  "retryable": true
}
```

首层提示是通用的“操作未完成”，具体原因藏在“查看详情”。用户同时报告更新确认多、看不到安装/下载反馈；随后要求参考 Resona 同窗进度，取消按钮背景进度并恢复正常按钮尺寸。截图作为故障及设计参考，不代表产品代码已改，也不提供另一个项目的执行授权。临时剪贴板图片未加入仓库。

## Windows 路径误判：已复现

只读查询固定卸载登记 `Software\Microsoft\Windows\CurrentVersion\Uninstall\MPV_Player_Automated_is1` 的 HKLM/HKCU、64/32 位视图，本次仅 HKLM 64 位有记录：

- InstallLocation：`C:\Program Files\MPV Player\`
- DisplayVersion：`v0.41.0`（登记版本，不证明 MPV 内部精确提交）
- 目标 `mpv.exe` 存在，文件长度 120308736，文件属性 Archive。

`src-tauri/src/platform/windows_mpv.rs` 从登记目录 `.join("mpv.exe")`，默认候选则从 Program Files `.join("MPV Player/mpv.exe")` 构造。`select_player` 对 `as_os_str()` 做 `eq_ignore_ascii_case`，只忽略 ASCII 大小写，没有统一路径分隔符。

用固定 Rust 1.98.1 的 Windows 标准库、实际登记值和 Program Files 路径执行独立最小探针，沿用源码中的 join 及比较表达式。输出：

```text
registered="C:\\Program Files\\MPV Player\\mpv.exe"
fallback="C:\\Program Files\\MPV Player/mpv.exe"
registered_exists=true fallback_exists=true
current_raw_dedup_equal=false path_components_equal=true canonical_equal=true
source_select_player_would_return=mpv_ambiguous
```

探针断言上述文件存在、原始比较不等、Windows Path 比较相等；编译和运行退出码均为 0。源文件、可执行文件及原始输出保留在忽略目录 `.local/alpha4-plan/path-probe.rs`、`path-probe.exe`、`path-probe.txt`，不作为产品修复提交。

结论：本次固定候选是**同一文件的两种分隔符表示**，不能据此要求用户删除“第二个 MPV”。这是检测去重错误；并非本次调查发现真实重复安装。canonicalize 仅用来只读佐证，不是建议取消重解析点保护。调查未扫描其他磁盘或所有已安装程序。

影响调用点：`player_path` 被 Windows `config_location`、配置/player 状态查询及核心 prepare/check_update/execute 共用；`app_root` 获取配置根也会调用它，因此可能在业务操作之前阻断。默认 Rust `cfg(test)` 与 Debug 隔离模式会直接返回 fixture 路径，现有完全相同字符串的去重测试没有覆盖混合分隔符，不能用此前通过的隔离测试排除此故障。

调查时尚未做：修复后产品启动复验、真实核心更新或安装。不清理注册表、真实 MPV 配置、用户安装或默认应用。用户原始操作入口和失败前后的安装状态没有完整日志，不能断言截图对应哪个更新阶段。

## 当前交互的代码依据

- `src/App.tsx`：`installOnly → download → preview_core（缺核心时）→ downloadEnhancements → preview_install → apply_deployment`，配置入口还可继续到 `preview_settings`，造成连续应用确认。
- 同文件：`busy && !progress` 的状态在主页面，预览 Modal 执行中继续打开，操作按钮只有 disabled；组件进度位于播放器页，缺少同窗阶段反馈。
- 同文件：错误同时写在主页面与 Modal，通用 `errorKey` 后技术 JSON 才提供部分真实原因。
- `src/app-updates.tsx`：Arch `prepare_app_update` 结束后须再点击 `install_app_update`；运行按钮变为多态 `div role=progressbar`，用 CSS 背景表示百分比，未知阶段挂 Loader。
- `src-tauri/src/updates.rs`：已有 download/verify/prepare/install Channel；Windows 目前在 install 内下载并直接交 NSIS，Arch prepare 已生成实际包事务。双端同窗设计可以复用这些能力。
- `src-tauri/src/platform/linux_core.rs` 的核心执行仍有 900 秒 capture 超时，应用更新模块则明确不强杀 Pacman。alpha.4 若统一核心任务反馈，应核对并修正核心提交期的相同边界，不能只改前端文案。

以上是静态调用审阅，不是对 alpha.4 UI 的运行测试；没有把用户反馈直接写成已修复。

## 退出确认空白色带：结构与样式已定位

用户新增截图显示“放弃未保存的配置？”标题与按钮之间有一条空色带。核对当前源码：

- `src/App.tsx` 的 closing Modal 仅有标题和 `.dialog-actions`，没有正文；该操作区直接作为 Modal Body 的子元素。
- `src/style.css` 两处定义 `.dialog-actions`，后一个仍保留 `margin-top:20px`；对只有按钮的短弹窗，也留下了这一段无内容间距，并叠加顶部分隔线与独立底色。
- 同文件全局 `.mantine-Paper-root` 深色背景为 `#2d2d2d`，操作区为 `#282828`；固定 Mantine 9.7.3 的 ModalBaseContent 使用 Paper，Header 则使用 `--mantine-color-body`。因此页面卡片背景规则也影响 Modal 内容，造成标题、空白带、底栏的分层色差。
- 离线预览同样有 `.dialog-actions` 的 20px 顶部 margin 和分区背景，实施时必须同步核对。

结论来自用户截图与当前 CSS/组件实现的对照，本轮没有重新启动 GUI 量测截图中的实际物理像素，调查阶段没有修改产品样式；本版实施和截图复核见下文。修正方案是紧凑的标题/短影响正文/按钮布局、统一内容底色、限定卡片样式作用范围，并核对所有使用共享操作区的弹窗；不靠增加占位文案或最小高度填充空白。

## 设计依据核对

- 复核 [Microsoft Dialog controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs)：短标题/具体动作、安全取消、就地错误；项目使用 Mantine 实现这些原则。
- 复核 [Microsoft Progress controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls)：确定与未知进度区分，进度控件本身只读；下载完成与整个安装完成分开表达。
- 已读仓库 Mantine 官方 LLM 索引及 Modal、Button、Loader、Alert 正文；Progress 的 LLM URL 本轮浏览工具因 Markdown content-type 读取失败，改读 [官方 Progress 页面](https://mantine.dev/core/progress/)。线上页面为 9.7.4，项目继续固定 9.7.3，并核对本地声明与实现：`Progress.value`、Section 的 aria 默认行为及 Modal 的焦点/关闭 props。
- 固定 9.7.3 的 `animated` 表示条纹动画，不等于未知总量模式。计划用 Progress 表示确定值，Loader 表示未知阶段，避免误播 100%。实施新增 Progress 官方快照时保存原始字节与来源摘要，不改写其他官方快照。
- 已读取用户指定的 Resona 聊天“制定 Alpha.3 开发方案”作为参考上下文；本版设计以用户本轮截图和明确要求为准，不引用其任务输出作为本仓库测试证据。

## 本轮文档检查

已核对 8 份本轮文档的 104 个相对链接、alpha.4 新增锚点、UTF-8 文本及文件末尾换行，新文档为 LF；`git diff --check` 和新增文档空白检查无问题。版本核对确认 package.json、Tauri 配置及 Cargo.toml 仍为 0.2.0-alpha.3。检查脚本位于忽略目录 `.local/alpha4-plan/check-docs.mjs`，运行退出码 0。

已审阅 Git diff，改动范围仅文档；既有 alpha.3 发布证据和源码事实继续保留，新增设计均标待审/未实施。未运行产品构建/单测/GUI、WinGet/Pacman 写操作或发布；最小 Rust 探针只证明原路径比较问题。

## alpha.4 实施与本地验证

本版实现 Windows Path.components + ASCII 大小写去重；保留本地绝对路径与重解析点校验。两个不同 MPV 文件仍返回 mpv_ambiguous，技术详情列出两个路径；首层用简中/英文说明具体原因。AppError 仅将 message 改为 String，JSON 字段契约不变。

同一 Modal 承接准备、下载、预览、唯一写入确认、执行、错误和结果。组件和配置保存独立；双端应用更新统一先下载验签再确认安装，Windows 暂存官方 updater 与字节，Arch 继续暂存现有包计划。关闭已准备更新释放候选；旧 Channel 回调不能覆盖下一阶段。标准 Button 与独立 Mantine Progress/Loader 取代按钮背景。

短退出弹窗用标题、后果正文与正常按钮；页面 Paper 背景限定在 page-surface，删除重复操作区样式和空白色带。长任务正文滚动，进度/操作区固定；准备时正文可聚焦，避免禁用按钮使初始焦点留在外部。未引入任务框架或新运行时依赖。

已执行的本地验证：

| 检查 | 结果与证据 |
| --- | --- |
| TypeScript / Vite | 固定 Node 24.19.0，tsc --noEmit 与 Vite build 通过 |
| 前端/发布契约 | 20/20 通过；.local/alpha4-js-final.log |
| Windows Rust | 45 通过 / 5 环境专项默认忽略；fmt/clippy 通过；.local/alpha4-rust-final.log、alpha4-clippy-final.log |
| 官方下载验签 | 包含在默认 Rust 测试：真实插件下载已签名非可执行文本；覆盖损坏/非法签名/HTTP 500/截断；未调用安装器 |
| 固定增强归档 | 显式专项通过：60 文件 / 33257710 字节，实际核对摘要/解包/隔离部署/卸载；.local/alpha4-assets.log |
| 本机只读检测 | 显式专项通过，但固定登记与真实 MPV 文件当前不存在，输出 installed=false 与预期 fallback；.local/alpha4-host-final.log。不能据此声称已安装主机复验通过 |
| Windows 原生 | 初轮全套 7/9，通过 52 项真实 MPV 音轨组合；两处旧测试未等待准备结束，已修正；受影响更新/配置/诊断/草稿四项复验 4/4 通过（97982ms），最终完整套件 **9/9 通过**（274207ms），.local/alpha4-native-verified.log，包含新诊断及退出布局检查 |
| 离线预览 | build/check 通过；首次预编辑、独立安装/保存、失败重试、mtime、手改、焦点/Esc、640/900/1200、200%、双语/主题/动效、强制颜色模拟及 Arch 更新；无 JS 错误/网络请求；.local/alpha4-preview-final.log |
| 许可材料 | Windows 331 包 / 587 材料、Linux 401 包 / 704 材料；collector --check 均通过，无缺失原文。版本/输入摘要更新，不把收集当完整人工许可审阅 |

已目视核对更新 42% / 未知进度 / 长日志 / 640 宽 200% 及退出浅深色 / 200% 截图：普通按钮文字完整、短弹窗无三色空带、独立进度可见。截图仅留 .local/e2e-artifacts/ 与 .local/ui-v016/，不提交用户桌面图。

普通验证均使用新临时根、已核对的 MPV/ffmpeg 与 WinGet fixture；未运行真实 WinGet、NSIS 或 Pacman 写入，不修改真实配置、注册表或默认播放器。初次纯 cargo build 未带 Tauri custom-protocol，导致测试无法找到内嵌页；已改用文档规定的 Tauri build --debug --no-bundle，并补启动失败清理。异步准备中的禁用按钮焦点问题已真实发现并修复；旧用例的等待错误与产品死锁分别记录，不把失败输出删成通过。

Linux 当前主机没有发行版或桌面会话；Rust/clippy、构建、makepkg、真实包元数据与隔离 Pacman 载荷由 Arch CI 验证。Windows NSIS 跨版本安装/重开及 Arch Polkit/KDE Wayland/首次依赖获取/跨版本重开仍缺隔离环境，不能用 IPC mock 或容器冒充。

强制颜色模拟已由 Edge 离线预览补验并目视复核（.local/ui-v016/forced-colors.png）：边界、文字及关闭按钮可辨。此项不等于 Windows 设置开关、真实读屏或物理高 DPI 多显示器验收。

## 双平台 CI 与合并

[PR #10](https://github.com/Leovikii/mpvkit/pull/10) 的 [Checks](https://github.com/Leovikii/mpvkit/actions/runs/40635264476) 全部通过，提交 e7ed20c。Windows Rust 45 通过 / 5 环境专项忽略；Arch Rust 50 通过 / 5 环境专项忽略，包元数据专项另 1 通过。Arch makepkg 与全新隔离根中的 Pacman 安装、同版重装、资源与卸载全部通过；日志 .local/alpha4-ci-windows.log、alpha4-ci-arch.log。隔离包脚本使用 -dd/--noscriptlet，不代表首次依赖拉取或图形授权代理。

PR 以极简英文 Merge dev 合并，提交 e9d41f0；源码 dev/main 已快进同步。本版 tag 指向该源码提交，不以交接文档后续补录移动 tag。自动 Release [运行](https://github.com/Leovikii/mpvkit/actions/runs/40635983418) 全部成功，公开结果见下文。

正式 Arch 包也通过 Rust、真实包元数据专项、makepkg、隔离安装/同版重装/资源/卸载、desktop-file-validate、readelf 和 vercmp。已下载当前 release-Linux 诊断产物核对 namcap：无错误；警告为 !strip 保留符号、ELF loader 的 unused、由已声明依赖传递满足的 libsoup3/cairo/glib2/hicolor-icon-theme/glibc/dbus/libgcc/gdk-pixbuf2，以及外部命令调用的 xdg-utils/polkit 未被链接分析识别。保留相应运行时依赖。完整日志和诊断在 .local/alpha4-release-arch.log 与 .local/alpha4-release-linux/。

## 公开发布核验

[v0.2.0-alpha.4](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-alpha.4) 于 2026-10-28 05:15:24 UTC 从创建起公开，draft=false / prerelease=true；创建请求 make_latest=false，匿名 releases/latest 返回 404，未成为稳定 Latest。tag 已通过 GitHub ref API 核对，固定指向 e9d41f0dcffe64c1a404196e7682a959ec131d83。

六项资产均已从公开下载地址匿名获取，核对文件大小、GitHub SHA-256 digest、SHA256SUMS、双平台 manifest URL/签名字段与 notes 契约。发布日志确认两个正式 Minisign 的 Signature and comment signature verified；未执行公开安装器。日志 .local/alpha4-release-publish.log、alpha4-release-windows.log；下载与校验脚本 .local/verify-public-alpha4.mjs，文件留在 .local/public-alpha4/。

| 公开文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| MpvKit_0.2.0-alpha.4_x64-setup.exe | 4110578 | 4b8b2f83cfe1fe77e9fd5b7972f820c37d02bdb19d410cd5ea8e22f7bb9b7392 |
| mpvkit-bin-0.2.0alpha4-1-x86_64.pkg.tar.zst | 6937063 | 71ba698831c8ccda27223e790c3e84d14bd1a581c426511f70e10a29ef069bba |
| latest.json | 1642 | c17089bcb2cd63728eeb48b9cb09c08f76c254be6e7055568cb0a54bcea8fecb |

另外三项为两平台签名及 SHA256SUMS。公开下载成功不等于跨版本安装验收；Windows NSIS、Linux Polkit/KDE Wayland 和真实首次依赖/重开仍保留上述未测范围。
