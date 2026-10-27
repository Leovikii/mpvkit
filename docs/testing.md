# 本地开发与测试入口

当前 v0.1.9，更新：2026-10-27。先读 [AGENTS](../AGENTS.md)、[当前交接](v0.1.9/plan.md) 与 [目录边界](current.md#当前目录)。本页为命令索引，不授予真实配置/默认应用/包管理器写入权限。无 CI、历史版兼容/迁移或安装包，不执行旧 NSIS 测试。

## 工具与最小检查

固定 Rust 1.98.1（rust-toolchain.toml）、Node 24.19.0、pnpm 11.25.0、Mantine core/hooks 9.7.3；不为接手升级依赖。本机已验证工具与研究资产位于忽略目录 .local，可 `source .local/linux-v018/env.sh`；新主机自行安装固定工具与 `pnpm install --frozen-lockfile`，不要假定 .local 已存在。Linux 原生构建依赖见 [v0.1.7 构建记录](v0.1.7/plan.md#本地开发)。

在仓库根运行；缓存齐备时 cargo 可加 --offline：

```sh
pnpm build
pnpm test:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
```

产物：src-tauri/target/debug/mpvkit.exe（Windows）或 mpvkit（Linux），bundle.active=false。默认 Rust 测试在临时根运行，部分测试需回环端口；隔离策略阻止绑定时，应明确说明并申请运行所需权限，不改成成功占位。纯文档修改只检查链接/格式/Git diff；逻辑、构建配置或共享样式变更按影响面验证，不无故重跑所有设备测试。

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

资产与视频路径是本机夹具；缺失时先按 [原生准备记录](v0.1.8/evidence.md#复验入口)准备并校验来源，不从任意 URL 下载后直接执行。套件会在测试根内部署资源、运行系统 MPV，验证预览/手改/过期、连播/字幕/续播、uosc/窗口、重置/卸载/清理及独立管理器入口，包管理器实际写操作被阻断。脚本注入指针不等于物理鼠标或完整 WebKit GUI 验收。

默认忽略的 real_mpv_contract 另需明确指定 MPVKIT_REAL_MPV / MPVKIT_REAL_TEST_ROOT，见 src-tauri/src/launch/real_tests.rs；不要用一个全局 `--ignored` 运行所有真实进程测试。

已知时序边界：uosc 暂停按钮即时断言曾失败一次，新根复验通过；IPC 可早于 time-pos 就绪，目前续播测试已等待属性生成。遇失败保留日志与根，不放宽断言、不把失败写成通过。

Linux 图标专项：隔离模式只写私有 applications，不足以证明真实 KWin 图标查找。用户已授权完成过真实入口登记与标题栏/面板取证，见 [证据](v0.1.8/manager-icon-evidence.json)。后续常规测试不得为复现而自动修改实际桌面/默认应用；已存在的本机管理器入口由正常启动维护。

## Windows 原生回归

在 Windows x64 上构建单 EXE，运行 `pnpm test:e2e`；快速侧栏检查为 `pnpm test:e2e:smoke`。脚本虽名 native-v016.test.mjs，仍是当前持续维护的原生套件，不是废弃文件。

完整套件须显式提供 MPVKIT_TEST_ASSETS、MPVKIT_CORE_SOURCE（已经核验并提取的 WinGet MPV 包目录）与 MPVKIT_FFMPEG（固定摘要的研究用生成器）；可用 MPVKIT_TEST_EXECUTABLE 选择当前 EXE。脚本创建私有临时根，以本地 Rust fixture 模拟 WinGet，绝不执行真实安装器。缺少这些资源应记录未测，不能替换为任意系统工具绕过校验。来源、包版本/实际提交差异及边界见 [WinGet 证据](v0.1.8/evidence.md#winget-核心与控制台2026-10-26当前批)。

v0.1.8 的 2026-10-27 本机回归已完成，见 [历史 Windows 证据](v0.1.8/evidence.md#windows-当前批实机回归2026-10-27)。本机 Cargo 需将 `$env:USERPROFILE + '\.cargo\bin'` 加入 PATH；普通沙箱阻止回环连接/文件替换时，应在获准进程复验，不能把批量 replace_failed 判作产品故障或改写替换保护。列表回绕后的下一步 seek 需同时等 playlist-pos 和 time-pos 就绪；固定品牌色与偏好字段断言已纳入侧栏专项。

本版不再包含媒体登记/Offreg 专项。包自带登记机制及 12 种常见扩展名的只读核验见 [v0.1.9 证据](v0.1.9/evidence.md#windows-软件包的媒体登记核验)；未执行主机安装器，不能等同真实 UAC/登记验收。当前完整原生结果以 [本版交接](v0.1.9/plan.md)为准。

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
