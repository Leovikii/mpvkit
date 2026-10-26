# MpvKit

MpvKit 是 MPV 安装与配置管理器。当前 **v0.1.8 开发测试版，Windows x64 与本机 Arch Linux x86_64 / KDE Wayland**，仓库：[Leovikii/mpvkit](https://github.com/Leovikii/mpvkit)。技术栈为 Tauri 2、React、TypeScript、Mantine 9.7.3 和 Rust；MPV 保持独立进程。

## 使用流程

- **播放器设置**：首次即可编辑画质、HDR、自动连播和全部播放结束行为并预览完整配置；可以仅安装组件，也可以安装后继续保存草稿。已有组件时直接保存新配置，显示 mpv.conf 实际修改时间。顶部统一管理核心、播放增强和配置：安装/更新/卸载核心、安装/移除增强、重置预设；目录入口保留在高级选项。
- **应用设置**：界面语言、明暗主题、关于、许可和诊断；界面固定使用图标同源的紫色品牌色（#A84BD6）。

Windows 通过 WinGet 的 `shinchiro.mpv` 包安装和主动更新核心；已有健康核心直接复用，不再自行下载每日构建。安装前展示包来源与目标路径，安装后重新检查实际版本及能力。缺少 WinGet 时提供 Microsoft 应用安装程序入口。配置目录按 MPV 原生规则检测：MPV_HOME、已存在的相邻 portable_config，否则 `%APPDATA%/mpv`；存在冲突、相对路径或重解析点时阻止写入。移除增强会按归属删除未修改的预设及生成项，保留手改内容、续播和系统 MPV。核心卸载由 WinGet 处理，保留用户配置。

Linux 使用系统 `/usr/bin/mpv`，Arch/CachyOS 可经 pkexec 调用 pacman 安装、更新或卸载固定 mpv 包；保留依赖检查，使用当前同步数据库，不单独刷新数据库或递归卸载依赖。自动检测其用户配置目录：优先 MPV_HOME，其次 XDG_CONFIG_HOME/mpv（通常 `~/.config/mpv`），按 MPV 规则识别原生 `~/.mpv`。相对路径、符号链接或多个用户配置目录并行时阻止安装。完整配置、脚本、字体及归属/恢复记录写入检测目录，配置部署不修改系统核心或默认应用。Linux 独立生成 Wayland/Vulkan、auto-safe 硬解和 HDR 参数。

Linux 保存后，原来的系统 MPV 打开方式及终端启动在下一次播放时直接读取配置，无需更换默认应用；附加 MpvKit 打开方式默认关闭。管理器启动时独立维护用户级 `MpvKit` 桌面入口和品牌图标；该入口打开管理界面，不依赖 MPV 或播放增强。可选媒体入口名为 `MPV (MpvKit)`，仅用于打开媒体。应用偏好留在 `$XDG_DATA_HOME/mpvkit`。不读取或迁移此前开发版私有目录，未知文件保留。

两端主配置均为检测目录中的 mpv.conf；快捷键在对应 input.conf。不设计 user.conf。保留默认位置续播、音量 ±10、文楷字体与原生画质 profiles。uosc 控制栏左侧仅上下集与播放/暂停，右侧分组提供轨道、倍速列表、播放模式、列表、信息、更多与全屏；主控放大，次要按钮透明，统一渐隐衬底；四种播放模式合并为原生菜单，信息按钮与 Shift+i 共用 MPV stats，窄窗优先收起次要入口。核心/增强安装与配置保存分开，GUI 保留预览、手改保护和当前操作失败恢复，删除历史备份/回退；实际文件修改时间独立显示。

安装、保存和失败恢复完成后自动清理当前操作暂存及恢复副本；卸载同时清理不再使用的自有日志和空目录。重置恢复全部默认预设，覆盖相应手改且不保存历史备份。详见[组件管理与清理](docs/v0.1.8/maintenance.md)。

键盘反引号（Esc 下方的 ` 键）打开 MPV 原生控制台，Esc 关闭；播放器右键菜单也提供“控制台”，与“播放信息”并列。截图功能已移除；进度条缩略图保留。

播放器默认以屏幕宽高 75% 范围启动，保持视频比例；切集保持当前窗口大小，手动缩放与全屏仍可用。使用 MPV 原生配置，不按视频像素尺寸撑满屏幕；Linux Wayland 在首次实际落屏后按当前 autofit 重算一次，显式窗口尺寸及手动调整受保护，切集不再重算。

自动连播默认开启：同文件夹音视频按文件名自然顺序组成播放列表，从当前所选文件继续，不扫描子目录。关闭开关只停止自动载入文件夹，明确打开的播放列表仍会连播。关闭播放器/保留窗口作用于整个队列结束。字幕自动搜索媒体同目录以及 sub、subs、subtitle、subtitles 子目录，支持 MPV 原生语种后缀匹配；Linux 兼顾常见大小写目录名。


## 开发测试约束

用户明确宣布正式版之前：**无 GitHub Actions/CI/CD、无历史开发版兼容或迁移、无安装包。** 未知用户数据保留，测试从隔离新环境开始。版本号变化、测试通过或 GitHub 推送不解除约束。

~~~powershell
pnpm build
pnpm test:frontend
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
~~~

Windows 单 EXE：src-tauri/target/debug/mpvkit.exe；Linux 单 ELF：src-tauri/target/debug/mpvkit，不构建安装包。Linux 开发依赖及隔离验证见 [v0.1.7 计划](docs/v0.1.7/plan.md)。Windows 原生测试见 scripts/native-v016.test.mjs，使用显式隔离目录、固定组件与合成媒体，不改变实际默认应用或配置；真实播放测试需设置 MPVKIT_FFMPEG 指向经过校验的研究用生成器，MPVKIT_CORE_SOURCE 指向明确校验后提取的 WinGet MPV 目录；测试 WinGet 仅为本地原生模拟程序，绝不调用真实安装器。

## 开发文档

后续 AI agent 从 [AGENTS.md](AGENTS.md) 开始，阅读[当前行为和目录](docs/current.md)、[v0.1.8 当前交接](docs/v0.1.8/plan.md)和[本地测试入口](docs/testing.md)；其他材料按[文档索引](docs/README.md)查阅。专项规范：[前端与 Mantine](docs/v0.1.0/frontend.md)、[来源政策](docs/v0.1.0/sources.md)、[未完成验收](docs/v0.1.0/acceptance.md)。

最新功能提交为 `7fa64a2`，用户已确认 uosc/窗口和 Linux 图标修复；Linux 默认 Rust 52 项、完整隔离 Wayland 回归及原生图标检查通过。2026-10-27 补齐 Windows 默认 Rust 63 项、私有 hive 专项、完整原生 8/8（另含 52 轨道子用例）与管理器桌面检查，见 [当前交接](docs/v0.1.8/plan.md)和[本版证据](docs/v0.1.8/evidence.md)。真实包管理器写操作、物理鼠标/完整辅助技术及设备矩阵仍待验收。不扩展 ARM、X11 或其他发行版。实体高刷/HDR/功耗、干净系统和完整来源/许可验收尚未完成，不把本机隔离测试等同正式发布条件。

## 许可证

MpvKit 使用 GPL-3.0-or-later，见[项目许可声明](COPYING.md)和[完整文本](LICENSE)。第三方组件、依赖、字体分别保留其许可证与署名。历史材料保留用于追溯，不作为执行旧路径、迁移或安装器流程的授权。
