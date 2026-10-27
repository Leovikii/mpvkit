# v0.2.0-alpha.1 实施与交接

更新：2026-10-27。源码已进入 **0.2.0-alpha.1**，从 22dc5e4 开发，未公开发布。用户已明确解除旧开发测试阶段对 CI/CD 和安装包的禁令，随后授权清理、提交并推送当前整批更新。**当前仍为第一个 alpha 预览版，0.2.0 是后续首个正式版目标，不能写成稳定版发布。** 更新策略：**正常 GUI 启动检查一次；接收预览版默认关闭；不提供自动检查开关或周期轮询。**

## 已实现

- 删除无 UI 入口的管理器启动 MPV 链、相关 IPC、会话日志及日志清理链；诊断导出保留。测试直接启动隔离系统 MPV，不为测试保留生产启动功能。
- 不再复制整个管理器到配置根 .mpvkit/launcher；scripts/mpvkit.lua 内嵌当前管理器绝对路径，以参数数组调用本体 --save-playback，复用 revision/generation、锁与原子事务。路径支持空格、UTF-8、引号及反斜杠，控制字符拒绝。移动程序后需重新预览/保存配置；不做历史版映射或迁移。
- 应用偏好根定为 Windows `%LOCALAPPDATA%/io.github.mpvkit`、Linux `$XDG_DATA_HOME/io.github.mpvkit`（回退 `~/.local/share/io.github.mpvkit`）；与 Windows 标准用户级程序目录分开。不清理或迁移真实旧数据。
- Linux 不再自行创建用户 desktop/ 资产及 applications 链接；Arch 包提供 `/usr/bin/mpvkit`、系统 desktop、同源图标与许可资源，GTK app_id 保持 io.github.mpvkit。移除播放增强不影响管理器软件包。
- 关于区加入应用更新。界面可用后延迟 3 秒检查一次；提前手动检查则合并。检查完成/失败可重试；切换预览开关立即废弃旧结果，偏好保存完成即可重查；旧响应不会覆盖新渠道。
- 预览默认关闭，不因 alpha 构建而开启。关闭只发现稳定版，开启取稳定/预览中 SemVer 最新者，不降级；stable=null 正常显示尚无稳定版。
- 更新弹窗显示版本、当地格式日期及纯文本日志；稍后不重复自动弹，关于区可重开。Windows 主动下载更新；未保存草稿先保存/重置。Linux 保留提醒，明确提示用 yay/paru 等 AUR 助手手动更新 MpvKit 后重启；只打开元数据标明已发布的 AUR 包入口，未发布则说明，不执行 helper 或自写 /usr/bin。预览开关不控制包管理器更新渠道。
- 按用户反馈重点修正 WinUI 3 布局：同一设置行的标题/说明在左、按钮/开关在右；统一边距和分隔线，窄窗检查按钮换行。弹窗固定操作区、日志滚动，主要操作在左、安全关闭在右，保留焦点锁/Esc、双语和深浅色。

## 定稿目录

```text
程序：Windows 用户级 NSIS 目录 / Linux /usr/bin/mpvkit
应用数据根/
  mpvkit.json                  # language/theme/previewUpdates，默认 false
  .mpvkit/                     # 归属、锁、当前事务；WebView 自有缓存由框架管理
MPV 原生配置根/
  mpv.conf, input.conf
  scripts/, script-opts/, fonts/, licenses/, watch_later/
  .mpvkit/settings.json         # 播放设置来源
  .mpvkit/owner.json, install.json, operation.lock
  .mpvkit/pending.json, staging/, recovery/  # 仅当前事务，完成后清理
```

不生成 launcher/、cache/sessions/ 或历史备份。owner/install/锁不可因看起来冗余而删；未知文件、手改与续播继续保留。Lua 直接记录路径，未再增加 script-opts/helper 路径文件。Arch 资源按固定 tauri-utils 的 resource_dir 规则安装到 `/usr/lib/mpvkit`，不是仅分发一个缺资源的 ELF。

## 更新与签名

实现见 src-tauri/src/updates.rs、src/app-updates.tsx。固定通道地址：

`https://github.com/Leovikii/mpvkit/releases/download/updates/channels.json`

```json
{"schemaVersion":1,"stable":null,"preview":{"version":"0.2.0-alpha.1","manifest":"https://github.com/Leovikii/mpvkit/releases/download/v0.2.0-alpha.1/updater.json","archPackage":null}}
```

每版 updater.json 使用插件 version/notes/pub_date/platforms 原生格式，Windows 键 windows-x86_64，signature 是签名正文。archPackage 仅允许 null、mpvkit-bin、mpvkit-preview-bin；null 表示 AUR 配方尚未公开。索引和资产只允许本仓库 HTTPS Release 路径，无凭据/query/fragment/编码路径；版本目录必须匹配，网络读取和日志大小有界。两个请求各 8 秒超时，无轮询。检查失败不会伪装成尚无发布。

Windows 使用锁定 tauri-plugin-updater 2.11.0 的 Rust API；安装再次核对候选 ID、30 分钟有效期、当前渠道、版本、URL 和签名。下载由插件验签，安装前持有应用数据/MPV 配置两根锁并拒绝未恢复事务；核心更新、增强部署与应用更新独立。安装会退出并重新启动应用，安装开始后不允许关闭确认窗或切换渠道；失败可重新检查。不提供正在下载安装时的取消按钮。

公钥通过 scripts/build-windows.mjs 将 MPVKIT_UPDATER_PUBLIC_KEY 写入忽略目录的 Tauri 配置，并内嵌到产物；不是仅设置 Rust 环境变量。私钥和密码通过 TAURI_SIGNING_PRIVATE_KEY / TAURI_SIGNING_PRIVATE_KEY_PASSWORD 注入。默认单程序开发配置 bundle.active=false、公钥为空，缺签名配置时明确拒绝安装。不能把本地临时测试钥匙用于公开发布。签名不等于 Authenticode，不承诺消除 SmartScreen。

公开 updates 元数据 Release、版本 Release、正式密钥/Secrets 尚未创建。未发布元数据前普通网络检查会失败，这是发布前提，不是假定已有公共服务。debug 的 MPVKIT_TEST_MODE=1 只读取隔离根 test-tools/updates 固定 JSON，且强制禁止执行安装器；release 不接受测试覆写。

## 分发与流水线

- Windows 仅 NSIS currentUser，配置在 src-tauri/tauri.windows.release.conf.json；不增加 MSI。按用户补充要求实现可见一键安装：启动即显示进度，固定 `%LOCALAPPDATA%\Programs\MpvKit`，成功自动关闭，无语言/确认/路径/完成向导，忽略 `/D` 与旧自定义路径。重装/升级原位覆盖，不先卸载，保留配置；禁止降级，保留异常和系统提示。模板来源及维护见[安装器说明](../../packaging/windows/README.md)。默认开发仍是单程序。
- Arch/CachyOS 用 AUR **mpvkit-bin**。scripts/release.mjs 以一次 native ELF 构建整理版本 tar.zst，生成固定 URL/SHA-256 的 PKGBUILD/.SRCINFO，再用 makepkg 生成 .pkg.tar.zst。不会二次编译 Rust；mpv 为 optdepends，保留无核心启动和独立卸载核心。WebKitGTK/GTK/xdg-utils 为运行依赖，polkit 为包操作可选依赖。
- 包提供系统 desktop/icon、/usr/lib/mpvkit 资源、完整随包许可材料；不自行媒体登记，不改默认应用。实际依赖仍须在 Arch 用 readelf、namcap 和干净环境启动核验。
- 版本 0.2.0-alpha.1 映射 pkgver=0.2.0alpha1；Arch vercmp 与实际 makepkg 升级尚待 Linux 主机验证。普通 pacman -Syu 不管理 AUR，用户用已选择的 helper 或审阅配方更新；应用不猜 helper、不执行 AUR 脚本。
- 暂不提供 AppImage、自建 pacman 仓库或其他发行版支持。AUR 包名未正式占用/发布；若未来确需并行稳定/预览包，再使用冲突的 mpvkit-preview-bin。

前一轮自动审批依据旧 AGENTS 禁令拒绝过可执行 Actions 和 NSIS 重建，原始记录保留在 evidence。**用户本轮明确解除这两项禁令**，AGENTS/入口文档已先更新，已启用 .github/workflows/ci.yml 与 release.yml，并删除重复惰性模板。本地完整签名 NSIS 打包成功，正式安装器未执行。当前 [流程说明](../ci/README.md) 与实际验证分别更新；用户随后授权提交/推送本批，不自行公开发布，也不把静态检查写成远端通过。

工作流包含日常检查和手动发布两条流程：日常前端只构建一次、Rust 两端并行，文档跳过重型任务；Node/pnpm/Rust 固定，Action 固定核验的 commit，Arch 固定容器摘要，Cargo 缓存含实际系统 ABI 摘要。发布两端只编译一次 release，Windows 按精确版本收集安装器与签名，避免缓存旧包混入；草稿收口、SHA256SUMS/签名核验后再推广 channels.json，不公开半套产物，不覆盖不可变版本资产，不自动发布 AUR。alpha 明确标为 prerelease、latest=false，只推广 preview。冷/热缓存耗时未测，不承诺运行分钟数。

发布推广必须通过现有许可 gate。新增 updater 依赖原文已补齐，两个 inventory 无 missingMaterials；**releaseReady=false 仍保留**，完整原生库与适用源码义务未完成审阅，不能靠改一个布尔值宣称可发布。

## 验证与边界

详见 [本轮证据](evidence.md) 与 [测试入口](../testing.md)。已完成的局部检查和最终回归分开记录，禁止把 v0.1.9 或中间产物结果套用于当前包。

Windows 完整原生 9/9（另含 52 项选轨）、Rust 43 项（4 ignored）、前端 3 项、分发契约 2 项通过；fmt/clippy、离线预览和 actionlint 通过。最后清理后额外复验应用更新与中文/空格程序位置的真实播放保存，均通过。截图核对深浅色和 640 宽/200% 字号，无更新设置横向溢出，长日志和固定操作区正常。证据含实际产物摘要与初次失败的修复记录。

本轮补充后 Rust **44 passed / 4 ignored**，fmt/clippy、前端/分发测试、两端许可材料一致性与实际工作流 actionlint 通过。官方 Tauri mock runtime 运行真实 updater 插件 check/download：有效签名、损坏内容、非法签名、下载 HTTP 500、截断响应、元数据 HTTP 500 六种情况均符合预期，不调用 install。固定签名夹具只有公开公钥、非可执行文本和签名，不含私钥。

一键 NSIS 包已成功构建并验签，坏内容和被篡改签名注释均拒绝。保留真实生成页面/初始化、替换全部三个系统写入 Section 的本地探针已实际运行：检测到可见进度窗口，免点击完成并关闭，`/D` 无法改变默认位置，没有创建真实安装目录。此探针只证明流程控制，不能替代正式包安装/卸载验收；包摘要及证据见 evidence。

本机无安装的 WSL、Linux VM/容器环境；可行 Linux 测试为平台无关 Rust 契约、实际资源映射/配方生成、版本与通道防倒退测试。**未完成当前 Linux 编译、makepkg、pacman 安装升级卸载、Wayland GUI/播放器验证。** 当前 Windows 系统真实 WinGet 写操作、NSIS 安装/卸载、alpha.1 → alpha.2 插件安装升级、UAC/物理输入/HDR 亦未验收；未改真实用户配置、注册表、默认应用或桌面入口。

本批收尾将日常 CI 改为仅 main PR/merge_group，取消 push 重复构建和整个工作流的路径过滤。固定 `CI` 汇总检查始终返回，文档变更只走轻量检测与汇总；前端一次构建、两端并行，失败/取消/意外跳过均阻止通过。ruleset 配置见[流程说明](../ci/README.md)；本机只读 main 规则 API 返回空列表，未改远端合并政策。

下一任务：维护者在首次 PR 检查出现后将 `CI` 设为 main ruleset 必需检查，并配置发布受保护环境和正式签名；在隔离 Windows 用户/VM 和 Arch Wayland 主机完成正式包安装、升级、卸载、资源/入口/helper 和配置手改保留验证；首次远端运行后记录冷/热缓存耗时，解决公开分发许可门槛后再发布。本批提交/推送已获明确授权，公开发布未获授权。

## 核验来源

- [WinUI 3 设置](https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings) / [对话框](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs)：右侧动作、分组、主要操作与安全关闭顺序。
- [Mantine Switch 官方正文](https://mantine.dev/llms/core-switch.md)、仓库 Modal/Button 正文与 9.7.3 类型声明：受控开关、可访问名称、焦点和弹窗 API。
- [Tauri updater](https://v2.tauri.app/plugin/updater/) / [Windows](https://v2.tauri.app/distribute/windows-installer/) / [AUR](https://v2.tauri.app/distribute/aur/)；实际 API 和资源路径核对本地锁定 crate，不能拿主线模板替代版本验证。
- [Arch PKGBUILD](https://man.archlinux.org/man/PKGBUILD.5.en)、[vercmp](https://man.archlinux.org/man/vercmp.8.en)、[打包规范](https://manual.archlinux.page/package-guidelines/)。
- [GitHub latest 语义](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)：不使用 latest 发现预览版；插件 endpoints 是 fallback，不是跨渠道取最大值。
