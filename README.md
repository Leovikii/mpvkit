# MpvKit

MpvKit 是 MPV 安装与配置管理器。当前 **v0.1.7 开发测试版，Windows x64 与本机 Arch Linux x86_64 / KDE Wayland**，仓库：[Leovikii/mpvkit](https://github.com/Leovikii/mpvkit)。技术栈为 Tauri 2、React、TypeScript、Mantine 9.7.3 和 Rust；MPV 保持独立进程。

## 使用流程

- **播放器设置**：首次即可编辑画质/HDR 并预览完整配置；可以仅安装组件，也可以安装后继续保存草稿。已有组件时直接保存新配置，显示 mpv.conf 实际修改时间。组件明细、维护和目录入口收在高级选项。
- **应用设置**：界面语言、明暗主题、强调色、关于、许可和诊断。

Windows 安装位置固定为本地应用数据目录下的 MpvKit/player。Linux 使用系统 `/usr/bin/mpv`，自动检测其用户配置目录：优先 MPV_HOME，其次 XDG_CONFIG_HOME/mpv（通常 `~/.config/mpv`），按 MPV 规则识别原生 `~/.mpv`。相对路径、符号链接或多个用户配置目录并行时阻止安装。完整配置、脚本、字体及归属/恢复记录写入检测目录；不修改系统核心或默认应用。Linux 独立生成 Wayland/Vulkan、auto-safe 硬解和 HDR 参数。

Linux 保存后，原来的系统 MPV 打开方式及终端启动在下一次播放时直接读取配置，无需更换默认应用；附加 MpvKit 打开方式默认关闭。应用偏好留在 `$XDG_DATA_HOME/mpvkit`。不读取或迁移此前开发版私有目录，未知文件保留。

主配置为 Windows 的 portable_config/mpv.conf、Linux 检测目录中的 mpv.conf；快捷键在对应 input.conf。不设计 user.conf。保留默认位置续播、音量 ±10、文楷字体、原生画质 profiles 和 uosc。核心/增强安装与配置保存分开，GUI 保留预览、手改保护、备份和失败恢复；实际文件修改时间独立显示。


## 开发测试约束

用户明确宣布正式版之前：**无 GitHub Actions/CI/CD、无历史开发版兼容或迁移、无安装包。** 未知用户数据保留，测试从隔离新环境开始。版本号变化、测试通过或 GitHub 推送不解除约束。

~~~powershell
pnpm build
pnpm test:frontend
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
~~~

Windows 单 EXE：src-tauri/target/debug/mpvkit.exe；Linux 单 ELF：src-tauri/target/debug/mpvkit，不构建安装包。Linux 开发依赖及隔离验证见 [v0.1.7 计划](docs/v0.1.7/plan.md)。Windows 原生测试见 scripts/native-v016.test.mjs，使用显式隔离目录、固定组件与合成媒体，不改变实际默认应用或配置；真实播放测试需设置 MPVKIT_FFMPEG 指向经过校验的研究用生成器。

## 开发文档

后续 AI agent 从 [AGENTS.md](AGENTS.md) 开始，阅读[文档入口](docs/README.md)、[当前行为和完整目录](docs/current.md)、[v0.1.7 实施与验证](docs/v0.1.7/plan.md)。专项规范：[前端与 Mantine](docs/v0.1.0/frontend.md)、[来源政策](docs/v0.1.0/sources.md)、[未完成验收](docs/v0.1.0/acceptance.md)。

本版基于远端 v0.1.6 的 `13939c1`，保留两页导航、预编辑和完整组件健康校验。Linux 仅覆盖当前本机，不扩展 ARM、X11 或其他发行版。实体高刷/HDR/功耗、干净系统和完整来源/许可验收尚未完成，不把本机隔离测试等同正式发布条件。

## 许可证

MpvKit 使用 GPL-3.0-or-later，见[项目许可声明](COPYING.md)和[完整文本](LICENSE)。第三方组件、依赖、字体分别保留其许可证与署名。历史材料保留用于追溯，不作为执行旧路径、迁移或安装器流程的授权。
