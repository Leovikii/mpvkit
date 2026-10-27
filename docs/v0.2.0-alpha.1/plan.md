# v0.2.0-alpha.1 实施与交接

更新：2026-10-27。源码已进入 **0.2.0-alpha.1**，从 22dc5e4 开发，当前任务为公开现有 alpha.1。用户已明确解除旧开发测试阶段对 CI/CD 和安装包的禁令，随后授权清理、提交并推送当前整批更新；现已明确要求更新文档、公开已有草稿，并同步 dev 与 main。**当前仍为第一个 alpha 预览版，0.2.0 是后续首个正式版目标，不能写成稳定版发布。** 更新策略：**正常 GUI 启动检查一次；接收预览版默认关闭；不提供自动检查开关或周期轮询。**

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

正式密钥/Secrets 已配置，双平台产物和正式验签已通过；此前版本仍为草稿、updates 元数据尚未公开，最新授权与实际发布结果见末尾。未发布元数据前普通网络检查会失败，这是发布前提，不是假定已有公共服务。debug 的 MPVKIT_TEST_MODE=1 只读取隔离根 test-tools/updates 固定 JSON，且强制禁止执行安装器；release 不接受测试覆写。

## 分发与流水线

- Windows 仅 NSIS currentUser，配置在 src-tauri/tauri.windows.release.conf.json；不增加 MSI。按用户补充要求实现可见一键安装：启动即显示进度，固定 `%LOCALAPPDATA%\Programs\MpvKit`，成功自动关闭，无语言/确认/路径/完成向导，忽略 `/D` 与旧自定义路径。重装/升级原位覆盖，不先卸载，保留配置；禁止降级，保留异常和系统提示。模板来源及维护见[安装器说明](../../packaging/windows/README.md)。默认开发仍是单程序。
- Arch/CachyOS 当前从 Release 下载原生 **mpvkit-bin** 软件包，用 `pacman -U` 手动安装/升级；AUR 注册暂时关闭，不再把尚未上架的 yay/paru 命令作为当前安装方法。scripts/release.mjs 以一次 native ELF 构建整理版本 tar.zst，生成固定 URL/SHA-256 的 PKGBUILD/.SRCINFO，再用 makepkg 生成 .pkg.tar.zst。不会二次编译 Rust；mpv 为 optdepends，保留无核心启动和独立卸载核心。WebKitGTK/GTK/xdg-utils 为运行依赖，polkit 为包操作可选依赖。
- 包提供系统 desktop/icon、/usr/lib/mpvkit 资源、完整随包许可材料；不自行媒体登记，不改默认应用。实际依赖仍须在 Arch 用 readelf、namcap 和干净环境启动核验。
- 版本 0.2.0-alpha.1 映射 pkgver=0.2.0alpha1；Arch vercmp、实际 makepkg 和隔离同版重装验证已通过；真实跨版本升级待下一版。普通 pacman -Syu 不管理 AUR，用户用已选择的 helper 或审阅配方更新；应用不猜 helper、不执行 AUR 脚本。
- 暂不提供 AppImage、自建 pacman 仓库或其他发行版支持。AUR 包名未正式占用/发布；若未来确需并行稳定/预览包，再使用冲突的 mpvkit-preview-bin。

前一轮自动审批依据旧 AGENTS 禁令拒绝过可执行 Actions 和 NSIS 重建，原始记录保留在 evidence。**用户本轮明确解除这两项禁令**，AGENTS/入口文档已先更新，已启用 .github/workflows/ci.yml 与 release.yml，并删除重复惰性模板。本地完整签名 NSIS 打包成功，正式安装器未执行。当前 [流程说明](../ci/README.md) 与实际验证分别更新；用户随后授权提交/推送本批，最新又明确授权公开现有 alpha.1；不把静态检查写成远端通过。

工作流包含日常检查和 Release 两条流程：日常前端只构建一次、Rust 两端并行，文档跳过重型任务；Node/pnpm/Rust 固定，Action 固定核验的 commit，Arch 固定容器摘要，Cargo 缓存含实际系统 ABI 摘要。main 推送（含 PR 合并）自动检查版本，无对应 tag 则创建并构建、校验后自动公开发布；已有 tag 跳过，手动入口仅用于失败重试。发布两端只编译一次 release，Windows 按精确版本收集安装器与签名，避免缓存旧包混入；资产收口、SHA256SUMS/签名核验后公开 Release 并更新 channels.json，不公开半套产物，不覆盖不可变版本资产，不自动发布 AUR。alpha 明确标为 prerelease、latest=false，只推广 preview。冷/热缓存耗时未测，不承诺运行分钟数。

发布推广必须通过现有许可 gate。新增 updater 依赖原文已补齐，两个 inventory 无 missingMaterials；**releaseReady=false 仍保留**，完整原生库与适用源码义务未完成审阅，不能靠改一个布尔值宣称可发布。

## 验证与边界

详见 [本轮证据](evidence.md) 与 [测试入口](../testing.md)。已完成的局部检查和最终回归分开记录，禁止把 v0.1.9 或中间产物结果套用于当前包。

Windows 完整原生 9/9（另含 52 项选轨）、Rust 43 项（4 ignored）、前端 3 项、分发契约 2 项通过；fmt/clippy、离线预览和 actionlint 通过。最后清理后额外复验应用更新与中文/空格程序位置的真实播放保存，均通过。截图核对深浅色和 640 宽/200% 字号，无更新设置横向溢出，长日志和固定操作区正常。证据含实际产物摘要与初次失败的修复记录。

本轮补充后 Rust **44 passed / 4 ignored**，fmt/clippy、前端/分发测试、两端许可材料一致性与实际工作流 actionlint 通过。官方 Tauri mock runtime 运行真实 updater 插件 check/download：有效签名、损坏内容、非法签名、下载 HTTP 500、截断响应、元数据 HTTP 500 六种情况均符合预期，不调用 install。固定签名夹具只有公开公钥、非可执行文本和签名，不含私钥。

一键 NSIS 包已成功构建并验签，坏内容和被篡改签名注释均拒绝。保留真实生成页面/初始化、替换全部三个系统写入 Section 的本地探针已实际运行：检测到可见进度窗口，免点击完成并关闭，`/D` 无法改变默认位置，没有创建真实安装目录。此探针只证明流程控制，不能替代正式包安装/卸载验收；包摘要及证据见 evidence。

前轮 Windows 环境无安装的 WSL、Linux VM/容器，当时只完成平台无关契约；这不代表当前 Linux 主机状态。**本轮 Linux 编译、makepkg、隔离 pacman 安装/同版重装/卸载及 Wayland GUI/播放器已通过，见下节；跨版本升级仍待真实下一版。** 当前 Windows 系统真实 WinGet 写操作、NSIS 安装/卸载、alpha.1 → alpha.2 插件安装升级、UAC/物理输入/HDR 亦未验收；未改真实用户配置、注册表、默认应用或桌面入口。

本批收尾将日常 CI 改为仅 main PR/merge_group，取消 push 重复构建和整个工作流的路径过滤。固定 `CI` 汇总检查始终返回，文档变更只走轻量检测与汇总；前端一次构建、两端并行，失败/取消/意外跳过均阻止通过。ruleset 配置见[流程说明](../ci/README.md)；本机只读 main 规则 API 返回空列表，未改远端合并政策。

下一任务：[PR #1](https://github.com/Leovikii/mpvkit/pull/1) 与 [PR #2](https://github.com/Leovikii/mpvkit/pull/2) 已合并，main 自动版本检测 / 建 tag / 双端草稿已走通，正式更新密钥签名与独立验签通过；本轮修复草稿校验清单上传遗漏。维护者仍需配置发布环境保护、设置 main 必需检查 `CI`，完成许可审阅及实际安装升级验收；真实下一 alpha 升级、polkit 授权与设备专项独立跟踪。公开现有 alpha.1 已获明确授权，不再沿用旧的未授权限制。

## Linux 必要测试精简与验收（2026-10-27，已完成）

用户先审阅方案，随后授权实施、完成 Linux 必要测试并推送 dev，由用户手动 PR 到 main。起点 ae815c2；测试环境为当前 CachyOS x86_64 / KDE Wayland，不沿用历史通过结果。版本保持 0.2.0-alpha.1。

- `docs/testing.md` 将默认必要检查、显式扩展回归和 Arch 包验收分开；前端 dist 只生成一次，debug/release 各自复用，CI 分类/离线预览/来源复采按变更触发，不重跑 Windows 套件，不修改双平台 PR CI 政策。
- 必要原生链保留部署、原生目录加载、helper 保存、缩略图、队列/字幕/续播及卸载保护，uosc 采用 1200/640 宽、4K 首次尺寸、手动缩放切集和 geometry/window-scale 保护；画质/HDR 从九组合缩为五个代表组合。字幕大小写等低成本覆盖和默认安全测试继续保留。
- 完整五档宽度、纵横比、音频切视频、启动状态和九组合移至 `arch_wayland_extended_window_and_render_matrix`；独立新根和音频夹具，不依赖必要链产物。本轮因移动覆盖而额外验证一次，以后按需运行。
- 修复测试执行器被误写入 Lua helper 的问题：`MPVKIT_TEST_EXECUTABLE` 仅在 `cfg(test)` 下生效，实际 debug/release 产品仍记录自身路径。删除前轮清理后遗留、无调用且会阻断 Linux clippy 的 `show_error`。
- 最终 Linux 默认 Rust **47 passed / 4 ignored**；fmt/clippy、前端 3 项、分发契约 2 项、Linux 许可材料一致性通过。必要 Wayland 链 **16.53 秒**，独立扩展 **17.50 秒**，KDE 语言/主题专项通过；不以历史耗时计算节省比例。
- 原生 WebKitGTK 短验收通过：启动一次、默认不收预览、保存后重查、失败重试、纯文本长日志、稍后/重开、AUR 已发布/未发布提示、无安装调用及深浅色偏好。没有移植整套 Windows 自动化或新增依赖。
- release ELF 构建后真实 makepkg 封装成功，收尾一致性重建的 release 摘要未变；vercmp、desktop、readelf、namcap 核验完成。在仅装运行依赖的全新隔离根验证无 MPV 启动；补 MPV/Vulkan 后验证实际 `/usr/bin/mpvkit` helper、desktop 启动与 app_id。真实 pacman 同版重装/卸载保留全部 **71 个配置文件**，系统 MPV 哈希不变，自有程序/入口/资源移除。

本轮命令、产物摘要、初次失败与修复、namcap 警告和环境限制见[当前证据](evidence.md#linux-必要回归与软件包验收2026-10-27)。测试根和脚本留在忽略目录或 /tmp，没有改变主机包数据库、默认应用或真实 MPV 配置。软件包验收使用新的用户空间根和主机 Wayland 会话，不冒称完整虚拟机或真实 polkit 交互验收；跨版本升级、物理输入/HDR/混合 DPI、公开 AUR/Release 与许可 gate 未由本轮关闭。

## 首次 CI 失败修复（2026-10-27）

- 用户提供的两个日志 ZIP 内容相同。Windows Rust **44 passed / 4 ignored**；Arch Rust **39 passed / 8 failed / 4 ignored**。根因是 Arch 容器的 pacman 依赖列表遗漏 `mpv`，本地已有系统 MPV 因而未暴露；Node 20 弃用警告是独立问题。
- CI 和 Release 同时补齐系统 MPV 测试依赖，保留全部默认安全测试；分发包仍将 MPV 作为可选依赖。5 个 Actions 升到官方核验的最新 Node 24 版本并固定完整 SHA，具体版本及兼容性见[工作流说明](../ci/README.md#工作流依赖与版本)。Arch 镜像仍是官方当前摘要，应用工具链未变。
- 在同一个只读 Bubblewrap 环境隐藏 `/usr/bin/mpv`，精确复现同样 **8 个失败**；恢复系统 MPV 后 **47 passed / 0 failed / 4 ignored**。实际 `cargo test --offline --locked` 同样通过；actionlint 1.7.12、CI 分类与分发契约 **3/3** 通过。仅改工作流依赖与文档，无需重复图形、安装包或 Windows 本地完整套件。
- 本地证据见[当前记录](evidence.md#首次-ci-失败定位与-workflow-组件更新2026-10-27)，远端结果以 [PR 检查](https://github.com/Leovikii/mpvkit/pull/1/checks) 当前提交为准；手动 Release 流程和公开发布不属于本次触发范围。

## 本机旧测试配置清理（2026-10-27）

- 用户明确要求完整清理本机不兼容旧配置。本机已安装 `mpvkit-bin 0.2.0alpha1-1`，但原生 MPV 根仍有旧 launcher 与含 `integrationEnabled` 的设置；当前严格反序列化不接受该字段。旧用户菜单链接仍指向仓库 debug 程序，并遮蔽系统包入口。
- 生成 79 项删除预览，核对旧清单中的 73 个 MPV 文件和 2 个桌面资产全部摘要匹配；持有两个既有操作锁、复核状态后删除旧配置、增强、launcher、旧应用偏好/清单和用户菜单链接。仅移除空目录，保留根及 `owner.json` / `operation.lock`。
- 删除约 290 MiB；11 条续播记录和系统程序、包资源、默认应用设置等共 27 个保护文件复核不变，KDE 菜单缓存刷新成功。当前应用数据根没有待清理的偏好文件，其 WebKit 缓存保留。未启动 GUI 或重新安装增强；下次打开新版后需重新安装增强并保存配置。详见[清理证据](evidence.md#本机旧测试配置清理2026-10-27)。
- 此为用户本轮授权的一次性本机操作，不增加产品迁移或自动清理逻辑，不提交或推送。

## main 自动 CD 与用户文档（2026-10-27）

- Release 增加 main push 触发，每次核对源码版本与本版日志，查询 `v<版本>`；仅真实 404 才创建 tag，绑定本次事件的精确 SHA，并在同一工作流继续双端构建、签名与草稿收集。已有 tag 正常跳过；网络 / 权限失败停止，并发建同一版本只允许成功创建者进入构建。不同 main 提交互不取消，公开推广任务仍串行。
- `build-draft` 手动入口保留，用于已有 tag 的失败重试；`promote` 仍需显式手动触发及原有许可 / 签名校验。预览草稿明确 prerelease、latest=false。没有依赖 GITHUB_TOKEN 建 tag 再触发工作流，也没有引入 PAT。
- 当前 notes 改为版本标题与四条英文变更，仍由同一文件供 Release 和 updater.json 使用；后续版本沿用此格式。README 按用户补充要求使用英文，精简为用户功能、两端安装和许可证，另增[预设配置设计](../presets.md)，agent 与开发入口留在 AGENTS / docs 索引。
- 针对性 Node 测试 **10/10**（含 CI 分类、分发及 API 反例子用例）；actionlint 1.7.12、JS 语法、英文日志 / 本地链接 / diff 检查通过。未改 Rust 或前端行为，不重复设备、图形和安装包套件。证据见[本轮记录](evidence.md#main-自动-cd-与用户文档2026-10-27)。
- 合并后的首次自动 CD 已创建 `v0.2.0-alpha.1` tag，Arch 构建通过，Windows 因缺少公钥停在签名打包前。`release-build` / `release-publish` 现已创建并配置同一正式公钥；无需改源码空公钥，打包脚本会注入。私钥须放入 build 环境的 Secret，而非 Variable；无密码时不添加密码 Secret。签名重试结果见后续证据，环境保护、公开推广与许可 gate 仍独立验收。

## 正式签名重试与草稿验收（2026-10-27）

- 用户将私钥配置到 build 环境 Secret，误存的同名 Variable 已删除；仅核对名称，不读取私钥内容。公钥在两环境中与用户提供文件一致，开发配置和测试夹具无需替换公钥。
- 对 [Release #40502127238](https://github.com/Leovikii/mpvkit/actions/runs/40502127238) 执行仅失败任务重试，第二次执行成功；Windows NSIS / updater 签名及 draft 通过，复用第一次成功的 Arch 结果，不重建 tag、不公开推广。
- 实际成品验收发现上传 glob `*.*` 漏掉 `SHA256SUMS`。新增真实 Bash 展开回归先复现缺失，再显式补上传文件参数；本地分发 / CI 分类 **11/11**、actionlint 与 diff / 文档链接检查通过。
- 当前草稿原有 9 个资产先与 GitHub 上传摘要逐一对照，再按原算法重建并补传遗漏清单，未替换原有资产。全部 9 项 SHA-256、updater 元数据及独立 Minisign 安装器 / 注释签名通过；草稿保持 prerelease、未公开。证据见[本轮记录](evidence.md#正式签名重试与草稿验收2026-10-27)。
- README 的 Arch 安装按用户要求优先给出 `yay -S mpvkit-bin` / `paru -S mpvkit-bin`；只读 AUR RPC 确认尚未公开，保留简短可用性说明。本次文档不升版、不改 tag；用户授权推送 dev、CI 通过后合并 main，再将 dev 快进同步 main，采用极简英文 commit。main 的版本检查应因已有 tag 跳过 CD 构建，结果以实际运行核对。

## 核验来源

- [WinUI 3 设置](https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings) / [对话框](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs)：右侧动作、分组、主要操作与安全关闭顺序。
- [Mantine Switch 官方正文](https://mantine.dev/llms/core-switch.md)、仓库 Modal/Button 正文与 9.7.3 类型声明：受控开关、可访问名称、焦点和弹窗 API。
- [Tauri updater](https://v2.tauri.app/plugin/updater/) / [Windows](https://v2.tauri.app/distribute/windows-installer/) / [AUR](https://v2.tauri.app/distribute/aur/)；实际 API 和资源路径核对本地锁定 crate，不能拿主线模板替代版本验证。
- [Arch PKGBUILD](https://man.archlinux.org/man/PKGBUILD.5.en)、[vercmp](https://man.archlinux.org/man/vercmp.8.en)、[打包规范](https://manual.archlinux.page/package-guidelines/)。
- [GitHub latest 语义](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)：不使用 latest 发现预览版；插件 endpoints 是 fallback，不是跨渠道取最大值。

## 自动公开发布修正（2026-10-27）

- 用户明确要求公开当前 alpha，并纠正“后续自动发布仍只留下草稿”的错误流程。Release 移除 draft 任务，package 只保存 Actions 资产；publish 在两端成功后自动核验许可 gate / SHA-256 / Minisign / updater，再公开版本和更新渠道。公开 alpha 保留 prerelease，已有 tag 的 main 推送继续跳过。
- 手动模式改为 `release`（重新构建并发布）和 `publish`（复用已有完整资产发布）；版本源码保持原 tag，执行流程脚本来自工作流提交，修复流程无需移动不可变 tag。首次 metadata Release 不存在时自动初始化；已有索引缺失、HTTP 错误、损坏或倒退均失败，不重置。现有 Release 的全部资产必须与本地哈希一致，发布重试不覆盖安装包。
- 发布回归与 CI 分类 **20/20** 通过，actionlint 1.7.12 / JS 语法 / diff 检查通过；现有 9 项清单、updater 与正式 Minisign 签名重新验证通过。不改应用源码或安装包，不重复原生播放/设备测试。
- 实际公开现有版本的请求被自动审批拒绝：仓库要求真实公开前完成许可审阅，四份分发/依赖记录仍为 `releaseReady=false`。没有绕过拒绝或把审阅结果改真；远端仍为草稿，公开更新索引仍未建立。工作流修正随后经 [PR #4](https://github.com/Leovikii/mpvkit/pull/4) 合并 main，Windows / Linux / 前端及 CI 汇总全通过；dev 已快进同步 main。合并后的 [Release 运行](https://github.com/Leovikii/mpvkit/actions/runs/40515916111) prepare 成功，因已有 tag 跳过 build/package/publish，没有创建新草稿；不能写成当前版本已公开发布。下一步解决这项明确前提后再公开现有资产，并验证更新地址 HTTP 200。

## 公开现有 alpha.1 的文档与授权更新（2026-10-27）

- 用户在获知前次自动审批拒绝的原因后，明确要求更新相关文档、公开已有草稿，确认版本仍为 `0.2.0-alpha.1`，保留预览标记；不升版、不改已有 tag 或安装包。完成后推送 dev，经 PR 合并 main 再同步 dev；以后不采用草稿发布流程。
- AGENTS 合并冲突的授权记录，删除“用户要求草稿流程”的错误归纳与“未获公开授权”的过时限制。README 保持英文，Arch 改为实际可用的原生包下载 / `pacman -U` 安装和更新，AUR 延后；开发入口、基线和 CI 说明同步。
- 许可原文与四份 `releaseReady=false` 记录保持原样，不把更新授权改写成完成审阅；当前公开结果待操作及匿名访问核验后填写。
- 用户进一步强调：新版本从创建时就必须公开，不能先建草稿再转公开。核对 GitHub CLI 2.103.2 对应官方实现后，发现携带资产的 `gh release create` 即使无 `--draft` 仍内部创建草稿；发布脚本改为 REST 明确 `draft=false` 创建，资产单独上传，索引最后更新。只有历史 alpha.1 草稿保留一次性转公开处理；新版本不接受草稿。回归扩展为 **21/21**，加入公开上传中断只补缺失资产的测试，actionlint / 语法 / diff 检查通过。
