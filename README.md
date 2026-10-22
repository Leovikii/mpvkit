# MpvKit

MpvKit 是 MPV 安装与配置管理器。当前 **v0.1.5 开发测试版，Windows x64**，仓库：[Leovikii/mpvkit](https://github.com/Leovikii/mpvkit)。技术栈为 Tauri 2、React、TypeScript、Mantine 9.7.3 和 Rust；MPV 保持独立进程。

## 使用流程

- **播放器**：安装/更新/修复核心和播放增强，查看版本、健康、配置与系统打开方式状态，必要时卸载并保留用户配置。
- **播放设置**：核心可用后显示；选择画质档位，预览并安装配置，后续修改或打开配置目录。
- **应用设置**：界面语言、明暗主题、强调色、关于、许可和诊断。

安装位置固定为 Windows 本地应用数据目录下的 MpvKit/player。核心与播放配置分开安装；默认在配置安装后添加系统打开方式，不接管默认应用。主配置修改时间读取真实文件时间。

直接编辑 portable_config/mpv.conf，快捷键在 input.conf；不设计 user.conf。保留默认位置续播、方向键音量 ±10、文楷字幕字体、原生画质 profile 与 uosc。GUI 只更新受管理范围，预览、手改保护、备份和失败恢复持续有效。系统打开方式通过稳定 helper 使用同一受管理启动；关闭管理界面不会终止播放。

## 开发测试约束

用户明确宣布正式版之前：**无 GitHub Actions/CI/CD、无历史开发版兼容或迁移、无安装包。** 未知用户数据保留，测试从隔离新环境开始。版本号变化、测试通过或 GitHub 推送不解除约束。

~~~powershell
pnpm build
pnpm test:frontend
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
~~~

单 EXE：src-tauri/target/debug/mpvkit.exe。原生测试见 scripts/native-v015.test.mjs，使用显式隔离目录、固定组件与合成媒体，不改变实际默认应用或配置；真实播放测试需设置 MPVKIT_FFMPEG 指向经过校验的研究用生成器。

## 开发文档

后续 AI agent 从 [AGENTS.md](AGENTS.md) 开始，阅读[文档入口](docs/README.md)、[当前行为和完整目录](docs/current.md)、[v0.1.5 实施与验证](docs/v0.1.5/plan.md)。专项规范：[前端与 Mantine](docs/v0.1.0/frontend.md)、[来源政策](docs/v0.1.0/sources.md)、[未完成验收](docs/v0.1.0/acceptance.md)。

Linux 尚未实现；只保留具体平台边界，不预建插件框架、后台服务或空适配器。实体高刷/HDR/功耗、干净系统和完整来源/许可验收尚未完成，不把本机隔离测试等同正式发布条件。

## 许可证

MpvKit 使用 GPL-3.0-or-later，见[项目许可声明](COPYING.md)和[完整文本](LICENSE)。第三方组件、依赖、字体分别保留其许可证与署名。历史材料保留用于追溯，不作为执行旧路径、迁移或安装器流程的授权。
