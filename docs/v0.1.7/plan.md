# v0.1.7：Arch Linux x86_64 / Wayland

2026-10-26 接续：已 fetch 确认最新 main 为本版提交 `e41cfda`。用户已批准 v0.1.8 uosc 布局方案及实现测试，Windows 优先，本机无 Linux 环境。共享经典控制栏、信息按钮和四模式菜单已实现，源码递增为 v0.1.8；当前交接与实际验证边界见 [v0.1.8 计划](../v0.1.8/plan.md) / [证据](../v0.1.8/evidence.md)。本页其余内容保留 v0.1.7 历史事实，不作为本轮 Linux 实测记录。

2026-10-25。用户授权开发下一版本，并进一步明确：只适配本机这类 x86_64 Arch Linux（当前 CachyOS）与 Wayland；配置按 Linux 实际能力生成，不照搬 Windows。保留 Windows x64 分支，不扩展 ARM、X11、其他发行版、包管理器或后台服务。无 CI/CD、历史开发版迁移或安装包。用户随后明确授权将本批更新提交并推送 GitHub；这不代表正式发布。

## 最新源码基线

本版先 fetch 并快进到远端 v0.1.6 `13939c1`（feat: unify player settings and configuration drafts for v0.1.6），再接续 Linux 改动，版本递增为 v0.1.7。原本地 Linux 工作已在 `/tmp/mpvkit-linux-rebase-20261025-EAwth0` 完整备份，并保留对应 Git stash。提交前再次 fetch，确认远端 main 仍为该基线；本批按用户授权提交并推送至 main，不创建正式发布或安装包。远端 [v0.1.6 方案](../v0.1.6/plan.md) 和 Windows 验证原文保留。

保留远端两页侧栏、首次预编辑、切页草稿、关闭放弃提示、固定预览/保存底栏，以及“仅安装组件”和“安装并保存配置”两条路径。只读预览复用 Rust 生成器，不要求组件存在；实际保存仍校验全部必需组件、fingerprint 与保留文件摘要，沿用实际 mtime 和未变化文件不重写。Linux 不恢复旧三页界面。系统 MPV 已安装行独立显示，组件安装只下载受管理增强。

## 实施与边界

- Linux 使用固定 `/usr/bin/mpv`，由系统包管理器安装/升级。不下载、复制、覆盖或卸载系统核心，不调用 sudo/pacman。界面提示必要依赖，安装组件只涉及播放增强。按实际选项与 Wayland/Vulkan context 检查能力；本机系统包 `mpv 1:0.41.0-6.1`，不把滚动发行版包冒称固定哈希资产。
- 系统 MPV 的已安装状态和实际版本独立于 MpvKit 归属记录查询，空安装和增强卸载后仍显示“已安装（系统）”。增强及配置是否可用单独判断，系统已有 MPV 不会误报增强已安装。
- uosc 5.13.0、thumbfast 固定提交及文楷 1.530 保持现有 HTTPS 来源、大小、SHA-256 与归档边界。只选 `ziggy-linux`（amd64 ELF）；不部署 `ziggy-windows.exe`、MPV Windows 归档、DLL 或其 fonts.conf。Linux 独立来源清单在 `resources/catalog-linux.json`，Windows 清单保留。
- `platform/linux.rs` 实现本地绝对路径与符号链接保护、Unix 有界管道读取、原子替换与目录同步、执行权限及进程存活检查；Windows API 和注册表仅在 Windows 编译。
- 标准库文件锁在 guard 结束时显式解锁，避免 Unix 并发 spawn 暂时继承文件描述符而延长已完成操作的锁；保留跨进程互斥并增加确定性回归测试。
- 用户级 desktop entry 后端复用原有登记预览、过期核对、锁、归属、事务日志和失败恢复。只管理 `$XDG_DATA_HOME/applications/io.github.mpvkit.desktop`，更新派生 MIME 缓存；不写 mimeapps.list 或接管默认应用。原值冲突阻断；外部手改取消登记时保留。Exec 使用 desktop-entry 双层转义，不经过 shell。
- GTK 原生文件选择器在主线程运行，支持 Wayland/portal；窗口使用不透明主题背景，无 Mica。目录打开使用 xdg-open；本机 KDE 默认应用入口使用 kcmshell6。系统强调色暂用蓝色，用户仍可手动选择主题与强调色。
- 跟随系统语言优先读取本机 KDE `plasma-localerc` 的 `Translations/LANGUAGE`，未指定则使用 `Formats/LANG`，再回退进程非空 locale；避免空 LANGUAGE 或启动器 C locale 覆盖桌面界面语言。明暗优先读取 portal Settings `ReadOne(org.freedesktop.appearance, color-scheme)`，无偏好/接口不可用时回退 GTK；原生背景和前端使用同一来源，保留手动偏好。继续通过聚焦和 15 秒轮询更新；不修改 KDE/GTK 系统配置，不新增依赖。
- 同一 crate、前端与两页导航。Linux 单独 Tauri 配置关闭透明背景，`bundle.active=false`；图标由既有 SVG 本地渲染 PNG。工具链仍 Rust 1.98.1、Mantine 9.7.3；删除强制给所有主机安装 MSVC target 的要求。Linux 直接依赖已有锁定版本 gtk 0.18.2（原生选择器/外观）与 libc 0.2.191（管道/进程检查），没有升级依赖。

## Linux 播放配置

| 项目 | Linux 当前选择与依据 |
| --- | --- |
| 视频输出 | `vo=gpu-next`、`gpu-api=vulkan`、`gpu-context=waylandvk`；只支持当前 Wayland/Vulkan 环境，需系统驱动 |
| 硬解 | `hwdec=auto-safe`，由本机 MPV 选择；禁用时为 `no`，不复制 Windows D3D11VA 参数、不按 GPU 名称推荐 |
| HDR 自动 | `target-colorspace-hint=auto`、`target-trc=auto`、`target-prim=auto`；由 MPV 与 compositor 的实际色彩协议处理 |
| HDR 强制开启 | `yes` / `pq` / `bt.2020`；不做硬件预检，不承诺屏幕已开启 HDR |
| HDR 关闭 | `target-colorspace-hint=yes`、`target-trc=srgb`、`target-prim=bt.709`；输出 SDR 且向 Wayland compositor 明确标记色彩。Windows 保持原 bt.1886/no 分支 |
| 字体 | 明确 `sub-fonts-dir=~~/fonts` 与 `osd-fonts-dir=~~/fonts`；从受管理字体目录加载，不安装系统字体、不复制 Windows fonts.conf |
| 路径与脚本 | 普通系统 MPV 默认读取检测目录；稳定 helper 显式使用同一目录。thumbfast 使用 `/usr/bin/mpv` 与上游 Unix socket，保存 helper 为无 `.exe` 的 Linux 程序 |
| 共用内容 | 原生画质 profiles、音轨/字幕顺序、音量 ±10、默认位置续播、截图和最小快捷键属于 MPV 公共能力；保留手改保护，使用本机 MPV 检查实际参数 |

依据为系统包附带 `/usr/share/man/man1/mpv.1.gz`（mpv 0.41.0）的 gpu-context、target-colorspace-hint、sub-fonts-dir、osd-fonts-dir、FILES 章节及实际 `--list-options` / `--gpu-context=help` 输出。手册明确 portable_config 自动发现只适用于 Windows；Linux 从不依赖它自动发现。

## 目录与数据

Linux 目录检测与安装已获用户授权并实施，详见 [配置接入](linux-config-routing.md)。配置根按 MPV_HOME → XDG_CONFIG_HOME/mpv 或 ~/.config/mpv → 标准目录不存在时的原生 ~/.mpv 规则判断；最后一项是 MPV 的原生搜索规则，不是 MpvKit 历史开发版兼容。相对覆盖路径、符号链接、非目录和重复用户配置阻断。界面显示检测结果与来源，不提供自选播放器或全盘扫描。

配置和增强直接写入检测根，mpv.conf、input.conf、scripts/、script-opts/、fonts/ 与 .mpvkit/、licenses/ 同根。为继续复用单目录事务，归属、备份、恢复和稳定 helper 位于配置根的隐藏 .mpvkit 下；仅应用外观偏好位于 $XDG_DATA_HOME/mpvkit。Windows 的 portable_config 和数据目录布局保留。

系统 MPV 的普通启动会自动读取；Linux 默认不额外登记打开方式，可显式启用已有可选登记。稳定入口 .mpvkit/launcher/mpvkit，脚本/helper 路径、健康检查、预览、真实 mtime、更新和卸载使用同一平台相对路径。Lua 在原生配置根查找保存记录；Linux 启动不再以日志目录重设 MPV_HOME，避免播放期保存定位错误。仍保留完整组件健康、fingerprint/摘要复核、手改保护和失败恢复。

普通新文件默认 0600，helper 设置用户执行权限。截图用 xdg-user-dir PICTURES/MpvKit。卸载只处理归属且未改变的增强、入口和可选登记，保留配置、watch_later、未知文件与系统 MPV。不读取、转换、迁移或删除此前开发版实际数据。

Debug 测试需要 `MPVKIT_TEST_MODE=1` 与绝对 `MPVKIT_TEST_ROOT`，配置根使用 MPVKIT_TEST_ROOT/config/mpv；desktop entry 写入隔离目录内部；Release 忽略测试覆写。可选实际播放测试的 `MPVKIT_TEST_EXECUTABLE` 只在 `cfg(test)` 存在，用已经构建的程序验证稳定启动/播放期保存，不增加产品 CLI 或 IPC。

## 本地开发

必要系统包（由开发者自行安装）：`base-devel webkit2gtk-4.1 gtk3 librsvg libappindicator-gtk3 patchelf mpv vulkan-icd-loader xdg-utils desktop-file-utils`，以及与本机 GPU 对应的 Vulkan 驱动；不替用户选择或安装驱动。本轮最终检查使用固定 Rust 1.98.1、pnpm 11.25.0、Node 24.19.0。

```sh
pnpm build
pnpm test:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
node scripts/collect-app-notices.mjs --linux --check
```

Linux 产物 `src-tauri/target/debug/mpvkit`（ELF），Windows 产物仍为 mpvkit.exe。不构建 AppImage、deb、rpm、NSIS、MSI，不添加工作流。

可选真实播放测试需要先用系统 ffmpeg 生成本地合成视频，准备逐字节校验的固定资产（uosc-5.13.0.zip、thumbfast-thumbfast.lua、LXGWWenKaiScreen.ttf），再提供全新的隔离根：

```sh
GDK_BACKEND=wayland MPVKIT_TEST_MODE=1 \
MPVKIT_TEST_ROOT=/tmp/mpvkit-new-run \
MPVKIT_TEST_ASSETS=/tmp/mpvkit-linux-assets \
MPVKIT_TEST_EXECUTABLE="$PWD/src-tauri/target/debug/mpvkit" \
MPVKIT_TEST_VIDEO=/tmp/mpvkit-linux-synthetic.mp4 \
cargo test --manifest-path src-tauri/Cargo.toml --locked \
  arch_wayland_install_playback_and_uninstall -- --ignored --nocapture
```

不执行旧 Windows 原生脚本作为 Linux 验收；Windows 测试入口仍保留，Win32 锁定/联接/注册表测试只在 Windows 编译。共享部署测试继续在 Linux 运行，新增符号链接、输出超时、desktop 转义/冲突/恢复和 Linux 配置隔离检查。

## 验证与交接

已完成 Linux 实施与本机隔离验证：45 项默认 Rust 测试、前端检查、Clippy、两平台依赖清单检查和单 ELF 构建通过；真实 KDE 系统偏好和 Wayland 集成显式分别通过，包含系统 MPV 首次显示/卸载后保留、退出后续播与九组画质/HDR 配置。当前结果、产物摘要及未完成项见 [evidence.md](evidence.md)。真实 HDR/功耗、Windows 原生回归、其他电脑、干净系统和完整来源许可审阅仍保留缺口；不能以强制参数被接受、截图或 GPU 使用率证明画质或 HDR 成功。下一任务优先为本机两页原生交互/辅助技术与长时间真实媒体验收，不扩张平台范围。

2026-10-25 双击未生效的根因是原私有目录不在系统 MPV 默认搜索范围。用户随后明确授权目录检测与安装；当前已改为原生目录，先前“切换打开方式”的提示已替换。隔离实测普通系统 MPV 在无 config-dir、无额外登记的情况下加载完整配置，并成功播放期保存；登记仍单独验收。当前证据见 [evidence.md](evidence.md)。

## 播放信息快捷键（2026-10-25）

用户授权将 i / Shift+i 加入 Windows 和 Linux 共同预设。resources/input.conf 增加 `i script-binding stats/display-stats`（短暂显示）与 `I script-binding stats/display-stats-toggle`（持续显示/关闭）。两平台配置生成器均直接嵌入这一份文件，独立预览也直接导入，不另建副本。已存在 input.conf 保留手改保护，不做自动追加或开发版迁移。

本轮相关 5 项配置测试、实际系统 MPV 绑定读取、TypeScript/Vite 与单 ELF 重建通过。Windows 原生按键交互未在本机验收，详见 [证据](evidence.md)。

2026-10-26 补充：当前 v0.1.8 已实施 Windows WinGet 核心/原生配置根与双端控制台，现行路径和测试结果见 [v0.1.8](../v0.1.8/plan.md)；本版历史 portable_config 等记录不作为现行 Windows 路径。

2026-10-26 后续：自动连播开关、整个队列结束行为及字幕相对目录/语种后缀规则同步实现，仍使用 MPV 原生参数。Linux 生成器和专用断言已同步，当前 Windows 实测与 Linux 待测边界记入 [v0.1.8 证据](../v0.1.8/evidence.md)，不沿用本版历史 ELF 作为新行为的验收。
