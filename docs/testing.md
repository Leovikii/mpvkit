# 本地开发与测试入口

当前 v0.2.0-alpha.1，更新：2026-10-27。先读 [AGENTS](../AGENTS.md)、[当前交接](v0.2.0-alpha.1/plan.md) 与 [目录边界](current.md#当前目录)。本页不授权真实用户配置/默认应用/包管理器写入；用户已允许本版 CI/CD、NSIS 和 Arch 包构建；分发配置与隔离验证边界见交接。无历史版兼容/迁移，不执行历史 NSIS 测试。

## 工具与最小检查

固定 Rust 1.98.1（rust-toolchain.toml）、Node 24.19.0、pnpm 11.25.0、Mantine core/hooks 9.7.3；不为接手升级依赖。本机已验证工具与研究资产位于忽略目录 .local，可 `source .local/linux-v018/env.sh`；新主机自行安装固定工具与 `pnpm install --frozen-lockfile`，不要假定 .local 已存在。Linux 原生构建依赖见 [v0.1.7 构建记录](v0.1.7/plan.md#本地开发)。

在仓库根运行；缓存齐备时 cargo 可加 --offline：

```sh
pnpm build
pnpm test:frontend
node --test scripts/release.test.mjs
node --test scripts/ci-changes.test.mjs
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
```

产物：src-tauri/target/debug/mpvkit.exe（Windows）或 mpvkit（Linux），bundle.active=false。默认 Rust 测试在临时根运行，部分测试需回环端口；隔离策略阻止绑定时，应明确说明并申请运行所需权限，不改成成功占位。纯文档修改只检查链接/格式/Git diff；逻辑、构建配置或共享样式变更按影响面验证，不无故重跑所有设备测试。

Windows 的 `updates::download_tests` 使用官方 Tauri mock runtime 和仅回环 HTTP，实际调用 updater 的 check/download：验证有效签名、损坏内容、非法签名、两类 HTTP 500 与截断下载；不调用 install。夹具仅含公开公钥、文本与签名。运行 `cargo test --manifest-path src-tauri/Cargo.toml --locked plugin_download_verifies` 可单独复验；生产端点仍强制 HTTPS。

## Linux 原生隔离回归

仅本机支持范围 Arch/CachyOS x86_64、KDE Wayland、系统 /usr/bin/mpv 0.41.0 与可用 Vulkan 驱动。此套件需要图形会话/会话总线、ffmpeg、desktop-file-validate、固定增强资产和本机合成视频，先构建当前 ELF。

```sh
# 必须使用一个尚不存在的新绝对目录；不要复用上次根或指向真实配置。
GDK_BACKEND=wayland MPVKIT_TEST_MODE=1 \
MPVKIT_TEST_ROOT=/tmp/mpvkit-v018-new-run \
MPVKIT_TEST_ASSETS="$PWD/.local/linux-v018/assets" \
MPVKIT_TEST_EXECUTABLE="$PWD/src-tauri/target/debug/mpvkit" \
MPVKIT_TEST_VIDEO="$PWD/.local/linux-v018/synthetic.mp4" \
cargo test --manifest-path src-tauri/Cargo.toml --locked \
arch_wayland_install_playback_and_uninstall -- --ignored --nocapture

GDK_BACKEND=wayland cargo test --manifest-path src-tauri/Cargo.toml --locked \
kde_session_preferences_are_available -- --ignored --nocapture
```

资产与视频路径是本机夹具；缺失时先按 [原生准备记录](v0.1.8/evidence.md#复验入口)准备并校验来源，不从任意 URL 下载后直接执行。套件在测试根内部署资源、运行系统 MPV，验证预览/手改/过期、连播/字幕/续播、uosc/窗口、重置/卸载/清理；包管理器实际写操作被阻断。系统管理器入口现由软件包提供，需单独包装验收。脚本注入指针不等于物理鼠标或完整 WebKit GUI 验收。

旧启动预览的 Rust 真实进程测试已删除；直接系统 MPV 启动由当前原生套件验证，不运行历史 launch/real_tests.rs。

已知时序边界：uosc 暂停按钮即时断言曾失败一次，新根复验通过；IPC 可早于 time-pos 就绪，目前续播测试已等待属性生成。遇失败保留日志与根，不放宽断言、不把失败写成通过。

Linux 图标历史取证见 [v0.1.8 证据](v0.1.8/manager-icon-evidence.json)，不证明 v0.2.0 的包入口已通过。当前不再由管理器生成用户 applications 或刷新旧入口；常规验证不得修改真实桌面/默认应用，Arch 包入口需在隔离系统安装后另验。

## Windows 原生回归

在 Windows x64 上构建单 EXE，运行 `pnpm test:e2e`；快速侧栏检查为 `pnpm test:e2e:smoke`。脚本虽名 native-v016.test.mjs，仍是当前持续维护的原生套件，不是废弃文件。

完整套件须显式提供 MPVKIT_TEST_ASSETS、MPVKIT_CORE_SOURCE（已经核验并提取的 WinGet MPV 包目录）与 MPVKIT_FFMPEG（固定摘要的研究用生成器）；可用 MPVKIT_TEST_EXECUTABLE 选择当前 EXE。脚本创建私有临时根，以本地 Rust fixture 模拟 WinGet，绝不执行真实安装器。缺少这些资源应记录未测，不能替换为任意系统工具绕过校验。来源、包版本/实际提交差异及边界见 [WinGet 证据](v0.1.8/evidence.md#winget-核心与控制台2026-10-26当前批)。

v0.1.8 的 2026-10-27 本机回归已完成，见 [历史 Windows 证据](v0.1.8/evidence.md#windows-当前批实机回归2026-10-27)。本机 Cargo 需将 `$env:USERPROFILE + '\.cargo\bin'` 加入 PATH；普通沙箱阻止回环连接/文件替换时，应在获准进程复验，不能把批量 replace_failed 判作产品故障或改写替换保护。列表回绕后的下一步 seek 需同时等 playlist-pos 和 time-pos 就绪；固定品牌色与偏好字段断言已纳入侧栏专项。

本版不再包含媒体登记/Offreg 专项。包自带登记机制及 12 种常见扩展名的只读核验见 [v0.1.9 证据](v0.1.9/evidence.md#windows-软件包的媒体登记核验)；未执行主机安装器，不能等同真实 UAC/登记验收。当前完整原生结果以 [本版交接](v0.2.0-alpha.1/plan.md)为准。

当前 Windows NSIS 已采用可见一键安装，默认位置固定、无向导；构建命令和来源见[安装器说明](../packaging/windows/README.md)。本地脱离系统写入的页面探针不是正式安装验收，不得直接在开发者真实环境执行正式安装器来代替隔离 VM/用户验证。

scripts/test-process.ps1、test-media-preferences.mjs、make-media-fixtures.mjs 为现有套件辅助；test-window.ps1 为独立原生窗口/DPI/截图探针，使用时先核对精确 PID/映像。截图仅为测试取证，不是已删除的产品截图功能。

## 当前离线 UI 预览

源码位于 docs/v0.1.6/preview，持续同步当前两页/品牌/组件动作；旧三页样机已删除。它只模拟状态，不调用 Tauri 或包管理器，不代替原生测试。

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

最新一次通过范围统一记在 [交接](v0.1.9/plan.md)，原始命令/日志路径/产物摘要记在 [证据](v0.1.9/evidence.md)。测试日志和合成媒体放 .local/ 或 /tmp，不提交 ELF、个人配置、凭据或完整桌面截图。

## v0.2.0-alpha.1 更新与分发检查

Windows 当前原生套件新增应用更新测试：默认不收预览版、启动一次、无稳定版、失败重试、预览偏好、渠道竞态、日志纯文本、稍后/重开、草稿保护及禁止测试安装器。元数据放新隔离根 test-tools/updates/channels.json 与 v<version>.json；不会接触公共频道。真实安装必须另有隔离 Windows 用户/VM，普通开发测试不写当前用户注册表。

布局专项覆盖 WinUI 3 对齐、640/1040 宽、200% 字号、深浅色、长日志滚动/固定操作区与焦点锁，截图只留 .local。独立预览同步当前界面，不能代替系统包或实际更新安装验收。

分发脚本命令与工作流见 [CI/CD 审阅说明](ci/README.md)。本机没有 Linux 执行环境，release.test.mjs 使用实际资源映射验证 Arch stage/配方/哈希/版本完整性，Rust linux_core_contract 验证参数；这些都不算 Arch 编译、makepkg、pacman 或 Wayland 实测。
