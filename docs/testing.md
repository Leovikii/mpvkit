# 本地开发与测试入口

当前源码及公开预发布 v0.2.0-beta.1，2026-10-31。双端CI与发布核验已完成，真实交互边界见[beta.1交接](v0.2.0-beta.1/plan.md)。先读[AGENTS](../AGENTS.md)及[目录边界](current.md#当前目录)，本版Linux/Windows证据分别记录，不用历史版本结果代替。

## 工具与最小检查

固定 Rust 1.98.1（rust-toolchain.toml）、Node 24.19.0、pnpm 11.25.0、Mantine core/hooks 9.7.3；不为接手升级依赖。本机已验证工具与研究资产位于忽略目录 .local，可 `source .local/linux-v018/env.sh`；新主机自行安装固定工具与 `pnpm install --frozen-lockfile`，不要假定 .local 已存在。Linux 原生构建依赖见 [v0.1.7 构建记录](v0.1.7/plan.md#本地开发)。

在仓库根运行；缓存齐备时 cargo 可加 --offline：

```sh
pnpm build
pnpm test:frontend
node --test scripts/release.test.mjs
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle --config '{"build":{"beforeBuildCommand":""}}'
```

上面的 `pnpm build` 已生成 dist，所以 Tauri 命令显式跳过重复的前端构建；单独构建时仍用默认 `pnpm tauri build --debug --no-bundle`。Linux 本轮另跑一次 `node scripts/collect-app-notices.mjs --linux --check`。CI 分类测试仅修改 CI/分类脚本时运行；来源复采、独立预览及其他设备专项按影响面选择，不重复全跑。

产物：src-tauri/target/debug/mpvkit.exe（Windows）或 mpvkit（Linux），bundle.active=false。默认 Rust 测试在临时根运行，部分测试需回环端口；隔离策略阻止绑定时，应明确说明并申请运行所需权限，不改成成功占位。纯文档修改只检查链接/格式/Git diff；逻辑、构建配置或共享样式变更按影响面验证，不无故重跑所有设备测试。

两端的 `updates::download_tests` 使用官方 Tauri mock runtime 和仅回环 HTTP，实际调用 updater 的 check/download：验证有效签名、损坏内容、非法签名、两类 HTTP 500 与截断下载；不调用 install。夹具仅含公开公钥、文本与签名。运行 `cargo test --manifest-path src-tauri/Cargo.toml --locked plugin_download_verifies` 可单独复验；生产端点仍强制 HTTPS。

## 单实例与 Linux 入口

仅启动/桌面入口/单实例变更时运行：

```sh
python3 scripts/test-single-instance.py
desktop-file-validate packaging/io.github.mpvkit.desktop
```

脚本自行创建私有 D-Bus 会话与全新临时根：两轮各并发启动六次，确认只有原始 GUI、重复启动退出 0、退出后能重新启动、播放期 helper 不被拦截、无会话总线时 GUI 失败而 helper 仍能处理。日志与 result.json 留在打印的 /tmp 根，不改真实配置。窗口激活仍受 Wayland 合成器策略约束，Windows 实际激活需 Windows 原生验证。

## Linux 原生隔离回归

只验证启动窗口策略时，在仓库根运行以下无窗口测试；它使用实际 Lua 策略和确定性时钟，不读取用户配置：

```sh
mpv --no-config --load-scripts=no --vo=null --ao=null --idle=yes \
  --script=src-tauri/src/linux_tests/window-policy.lua
```

真实窗口回缩仍须运行下面的扩展入口，比较稳定后的尺寸与正确落屏的原生参考；无窗口测试不能证明合成器时序。

仅本机支持范围 Arch/CachyOS x86_64、KDE Wayland、系统 /usr/bin/mpv 0.41.0 与可用 Vulkan 驱动。此套件需要图形会话/会话总线、ffmpeg、desktop-file-validate、固定增强资产和本机合成视频，先构建当前 ELF。

```sh
# 必须使用一个尚不存在的新绝对目录；不要复用上次根或指向真实配置。
GDK_BACKEND=wayland MPVKIT_TEST_MODE=1 \
MPVKIT_TEST_ROOT=/tmp/mpvkit-alpha1-new-run \
MPVKIT_TEST_ASSETS="$PWD/.local/linux-v018/assets" \
MPVKIT_TEST_EXECUTABLE="$PWD/src-tauri/target/debug/mpvkit" \
MPVKIT_TEST_VIDEO="$PWD/.local/linux-v018/synthetic.mp4" \
cargo test --manifest-path src-tauri/Cargo.toml --locked \
arch_wayland_install_playback_and_uninstall -- --ignored --nocapture

GDK_BACKEND=wayland cargo test --manifest-path src-tauri/Cargo.toml --locked \
kde_session_preferences_are_available -- --ignored --nocapture
```

资产与视频路径是本机夹具；缺失时先按 [原生准备记录](v0.1.8/evidence.md#复验入口)准备并校验来源，不从任意 URL 下载后直接执行。套件在测试根内部署资源、运行系统 MPV，验证组件/配置部署、原生目录加载、播放期 helper、连播/字幕/续播、缩略图、uosc、代表性窗口/渲染和卸载/清理。手改/过期/路径/锁/恢复等安全边界由默认 Linux Rust 测试完整覆盖，不在每个 GUI 场景重复；包管理器实际写操作被阻断。系统管理器入口由软件包提供，按文末独立进行包验收。脚本注入指针不等于物理鼠标或完整 WebKit GUI 验收。

默认原生入口保留 1200/640 两档 uosc 宽度、4K 首次 autofit、手动缩放后切集保持、geometry/window-scale 保护；画质/HDR 是三档画质 × Auto，加均衡 On/Off，共五组合。同一进程内的十二种字幕目录大小写及自然队列覆盖继续保留。`MPVKIT_TEST_EXECUTABLE` 仅在 Rust 测试编译时选择实际 helper，生产程序始终记录自身路径。

仅修改窗口/Lua/渲染配置、组件版本，或相关默认测试失败时运行扩展入口。该入口使用独立新根，自行部署及生成音频样本，不依赖必要链产物：

```sh
GDK_BACKEND=wayland MPVKIT_TEST_MODE=1 \
MPVKIT_TEST_ROOT=/tmp/mpvkit-alpha1-extended-new-run \
MPVKIT_TEST_ASSETS="$PWD/.local/linux-v018/assets" \
MPVKIT_TEST_EXECUTABLE="$PWD/src-tauri/target/debug/mpvkit" \
MPVKIT_TEST_VIDEO="$PWD/.local/linux-v018/synthetic.mp4" \
cargo test --manifest-path src-tauri/Cargo.toml --locked \
arch_wayland_extended_window_and_render_matrix -- --ignored --nocapture
```

扩展覆盖五档宽度、1080p/4K/竖屏/超宽、自定义 autofit、音频切视频、最大化/全屏启动及完整九组合。两个图形入口分别运行，不用 `--ignored` 一次启用所有在线/设备验收。失败保留根和日志，修复后只重跑受影响入口。

Linux GUI 短验收使用当前 debug 程序、新隔离数据根和固定更新 JSON：两页基本操作、语言/深浅色、启动仅检查一次、预览默认关闭与切换后重查、错误重试、纯文本长日志、下载/包事务预览/授权取消及完成待自动重启；不再验收已删除的 AUR 提示。IPC mock 不执行真实 Pacman。Windows 原生 Node 套件使用 WebView2 和 WinGet 夹具，不在 Linux 强行移植整套。alpha.1 历史短验收见[原始证据](v0.2.0-alpha.1/evidence.md#linux-必要回归与软件包验收2026-10-27)，alpha.3 的历史环境及未测范围见[交接](v0.2.0-alpha.3/plan.md)。alpha.5 当前结果及限制见本版交接。

Linux 图标历史取证见 [v0.1.8 证据](v0.1.8/manager-icon-evidence.json)，不证明 v0.2.0 的包入口已通过。当前不再由管理器生成用户 applications 或刷新旧入口；常规验证不得修改真实桌面/默认应用，Arch 包入口需在隔离系统安装后另验。

原生 GTK/WebKit UI：先构建上述独立 debug ELF，再运行 `node scripts/test-linux-ui.mjs`；固定资产默认取本机 `.local/linux-v018/assets`，可由 MPVKIT_TEST_ASSETS 指定，MPVKIT_TEST_EXECUTABLE 可指定最终测试 ELF。测试创建私有根，记录调整前首启尺寸，并覆盖窄窗、系统偏好、草稿/取消/统一安装/独立保存/核心失败仍完整删除配置、深浅主题、主页/设置页原生窗口截图及 KWin 服务端装饰协商。需要 Node、Spectacle、qdbus6 和当前 KDE 会话；测试用按 PID 限定的临时 KWin 脚本激活自己的窗口，随即卸载，不修改合成器设置。请串行运行图形测试，保留测试窗口前台；焦点丢失应判测试失败，不能使用其他窗口的截图。`GTK_THEME=HighContrast node scripts/test-linux-ui.mjs` 复验无模糊实色路径。截图不能证明多屏 DPI 或物理拖动已通过。若宿主窗口恢复插件覆盖默认尺寸，可显式设置 `MPVKIT_TEST_RESIZE_WINDOW=1`，仅调整本测试 PID；结果会标记 layoutExplicitlyResized，此时只能证明调整后的布局，不能当作无干预首启通过。最终测试包未启用该变量的结果见 beta.1 交接。

## Windows 原生回归

在 Windows x64 上构建单 EXE，运行 `pnpm test:e2e`；快速两页检查为 `pnpm test:e2e:smoke`。脚本虽名 native-v016.test.mjs，仍是当前持续维护的原生套件，不是废弃文件。

beta.1 已在 Windows x64 上执行构建及隔离回归，包含原生560×760首启、192 DPI、单实例唤回、helper分派及固定MPV播放。命令、首轮失败和定向复验见[Windows证据](v0.2.0-beta.1/evidence.md#windows-自动化续验2026-10-31)。当前本机隔离测试入口 `.local/beta1-windows/app/run.cmd` 不执行真实WinGet或NSIS安装；不能用它替代VM中的包管理器问答和真实更新重启。

完整套件须显式提供 MPVKIT_TEST_ASSETS、MPVKIT_CORE_SOURCE（已经核验并提取的 WinGet MPV 包目录）与 MPVKIT_FFMPEG（固定摘要的研究用生成器）；可用 MPVKIT_TEST_EXECUTABLE 选择当前 EXE。脚本创建私有临时根，以本地 Rust fixture 模拟 WinGet，绝不执行真实安装器。缺少这些资源应记录未测，不能替换为任意系统工具绕过校验。来源、包版本/实际提交差异及边界见 [WinGet 证据](v0.1.8/evidence.md#winget-核心与控制台2026-10-26当前批)。

本版不再包含媒体登记/Offreg 专项。包自带登记机制及 12 种常见扩展名的只读核验见 [v0.1.9 证据](v0.1.9/evidence.md#windows-软件包的媒体登记核验)；未执行主机安装器，不能等同真实 UAC/登记验收。历史完整原生结果见 [alpha.1 交接](v0.2.0-alpha.1/plan.md)，本版未完成的 Windows 原生项目及接手顺序见 [beta.1 Windows 交接](v0.2.0-beta.1/plan.md#windows-测试交接)。

当前 Windows NSIS 已采用可见一键安装，默认位置固定、无向导；构建命令和来源见[安装器说明](../packaging/windows/README.md)。本地脱离系统写入的页面探针不是正式安装验收，不得直接在开发者真实环境执行正式安装器来代替隔离 VM/用户验证。

scripts/test-process.ps1、test-media-preferences.mjs、make-media-fixtures.mjs 为现有套件辅助；test-window.ps1 为独立原生窗口/DPI/截图探针，使用时先核对精确 PID/映像。截图仅为测试取证，不是已删除的产品截图功能。

## 当前离线 UI 预览

源码位于 docs/v0.1.6/preview，渲染当前实际两页 App 并模拟状态；旧样机已删除。它不调用真实包管理器，不代替原生测试。

```sh
node docs/v0.1.6/preview/build.mjs
# 本机 Linux 用已安装的 Chromium；Windows 不指定时默认 Edge。
MPVKIT_BROWSER="$PWD/.local/playwright/chromium-1234/chrome-linux64/chrome" \
node docs/v0.1.6/preview/check.mjs
```

输出 .local/ui-v016/preview.html；需要交互服务时 build.mjs 加 --serve，地址为 127.0.0.1:1516，仅手动启动/停止。[预览说明](v0.1.6/preview/README.md)。

## 来源与分发材料

修改固定组件/依赖/字体/Lua 扩展时，按 [来源政策](v0.1.0/sources.md)及相关 evidence 重新校验；没有变更时不重新采集或格式化第三方原文。

- `node scripts/collect-app-notices.mjs --check` 与 `--linux --check`：两平台应用依赖材料一致性。
- scripts/verify-uosc-sources.py：固定本地源码归档；verify-uosc-icons.mjs：固定字体映射；verify-scroll-bar-license.py、collect-go-notices.py：对应来源/许可。先读参数及原证据，不把这些脚本删成“无人调用”。
- scripts/rebuild-uosc-helper.py、inspect-uosc-buildinfo.mjs、compare-uosc-links.go：固定 helper 的构建/核验链；用途和未完成发布审阅在 [验收台账](v0.1.0/acceptance.md)。

最新一次通过范围统一记在 [当前交接](v0.2.0-beta.1/plan.md)，本版命令和日志路径亦见交接；历史产物摘要见 [alpha.1 证据](v0.2.0-alpha.1/evidence.md)。测试日志和合成媒体放 .local/ 或 /tmp，不提交 ELF、个人配置、凭据或完整桌面截图。

## 更新与分发专项

Windows 原生套件覆盖启动一次、预览偏好、无稳定版、失败重试、版本发现竞态、日志纯文本、草稿保护和测试安装阻断。隔离根 test-tools/updates 使用 releases-N.json、v<version>.json 及 v<version>-SHA256SUMS，包含六项资产名/真实摘要。Windows 与 Arch 的进度、预览、授权取消/失败/完成由原生 WebView 中的 IPC mock 演示，不运行真实安装器或 Pacman；官方插件下载验签在两端 Rust 回环测试实际执行。

布局专项覆盖 WinUI 3 对齐、480/560/720 宽、200% 字号、深浅色、长日志滚动/固定操作区与焦点锁，截图只留 .local。独立预览同步当前界面，不能代替系统包或实际更新安装验收。

分发脚本命令与工作流见 [CI/CD 审阅说明](ci/README.md)。前轮 Windows 环境只运行 release.test.mjs 的资源/配方契约与 Rust 包参数测试，不能作为 Arch 实测。历史 alpha.1 在 CachyOS/KDE Wayland 完成的编译、makepkg、原生 GUI/播放和隔离 pacman 验收见[当前证据](v0.2.0-alpha.1/evidence.md#linux-必要回归与软件包验收2026-10-27)。

Arch 包验收独立于默认单测：已有 dist 时，用 `pnpm tauri build --no-bundle --config '{"build":{"beforeBuildCommand":""}}'` 构建一次 release，再以非 root 用户运行 `node scripts/release.mjs arch <新输出目录>`。makepkg 仅封装，不二次编译 Rust。核验 vercmp、desktop-file-validate、readelf、namcap 与许可/资源清单；namcap 警告逐项说明，不只看退出码。debug 隔离测试和 release 包验收各自有用途，不用测试覆写替代 release 的原生用户目录行为。

真正的 pacman 安装、同版重装、卸载及包入口验证在全新隔离系统根进行；先只装运行依赖，确认缺 MPV 仍可启动，再加入 MPV 与所需 Vulkan 驱动核验实际播放期保存。重装/卸载保留配置、未知文件和系统 MPV，不改变主机默认应用。隔离根引导所需的系统依赖缓存须验证来源与签名；初始化钩子/显示会话受容器限制的情况单独记录。无真实后继版时，同版重装不算跨版本升级；升级到下一 alpha 时再验。真实 polkit 授权和设备矩阵不以契约测试冒充通过。

alpha.3：PR 的 Arch 容器运行 packaged_metadata_matches_pacman 显式测试，以及 scripts/test-arch-package.sh 的新隔离根载荷验收；后者 -dd/--noscriptlet 不验证依赖拉取、桌面会话或授权代理。该段为 alpha.3 当时环境；本机 alpha.5 已具备 KDE Wayland，当前包与原生测试结果见 alpha.5 交接，实际系统授权/跨版本重启仍待隔离 VM。
