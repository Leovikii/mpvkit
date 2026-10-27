# v0.1.9 验证证据

日期：2026-10-27（Asia/Hong_Kong）。基线 63945b3，用户授权取消两端专属媒体打开方式管理，先完成 Windows 验证。只构建单 EXE，bundle.active=false；不执行 CI、安装包构建或历史开发版迁移，不改实际用户配置、默认应用或媒体登记。

## 实现范围

- 删除 integration 转发层、平台登记事务、Windows 写注册表后端/Offreg 专项和 Linux 专属媒体 desktop 后端；删除状态、意图、确认、恢复、IPC 和相应调用。配置/组件/卸载回到单一文件事务，保留锁、过期复核、手改保护及当前事务失败恢复。
- settings.json 移除 integrationEnabled；不读旧字段，不自动迁移或注销真实旧记录。用户测试须使用新隔离根。删除仅服务专属媒体入口的 --open CLI 和 Linux 媒体图标副本；--save-playback helper 及其许可继续部署，系统 MPV 直接读取原生配置。
- Linux 管理器 io.github.mpvkit.desktop 及其独立图标/归属保护保留。desktop Exec 转义原样移至实际使用的 manager_entry 模块。
- 默认应用快捷按钮移至 MPV 核心行，与检查更新/卸载同组，普通次要样式；没有核心时禁用，有核心但未装增强时可用。只打开系统设置，不参与配置草稿、保存或默认应用写入。高级项保留目录和来源。中英产品与独立预览同步。

## Windows 软件包的媒体登记核验

WinGet 的职责是启动包提供的安装器，登记动作由该包完成。固定 shinchiro.mpv 0.41.0 的 [WinGet 清单](https://github.com/microsoft/winget-pkgs/blob/master/manifests/s/shinchiro/mpv/0.41.0/shinchiro.mpv.installer.yaml) 指向 0GMou/mpv2winget v0.41.0。已有本地安装包 SHA-256 **159441c52fd74755cb74c830f661fcd9517c77056887c27ac0569876b600f0c9** 与清单一致，未执行安装器。

本轮读取该 tag 对应提交 **d41f49759acd340e37a3737cf48c899b64692d91** 的 [build_mpv.iss](https://github.com/0GMou/mpv2winget/blob/d41f49759acd340e37a3737cf48c899b64692d91/build_mpv.iss)，本地源文件 SHA-256 **327ff405c81dddb36da6ccec0ad1094d6df788f610fd74093322947d897668cf**。核验实际执行条件及写入目标：

1. 静态段登记 mpv.file、Applications/mpv.exe 的打开命令和 MPV Player 的 RegisteredApplications/Capabilities。
2. Pascal [Code] 的 CurStepChanged 在 ssPostInstall 调用 RegisterPerceivedTypes。它枚举 HKCR 已有扩展名，读取 PerceivedType；分类为 audio/video 时，写入 Capabilities/FileAssociations 的 extension → mpv.file，并写入对应扩展名的 OpenWithProgids/mpv.file。
3. 没有写用户 UserChoice，也不会通过这段逻辑把 MPV 强制设为默认应用。

此前只查看解包后的静态 [Registry] 段，曾判断“未完整声明逐扩展名关联”；该提取物没有显示编译的 Pascal 扫描逻辑。**本轮固定 tag 的完整上游源码补齐并修正此判断**，不能再把静态段缺少扩展名当作包没有关联能力。

只读查询本机 12 个已知 HKCR 扩展名（没有扫描磁盘或写注册表）：

| 视频：PerceivedType=video | 音频：PerceivedType=audio |
| --- | --- |
| .mkv、.mp4、.webm、.avi、.mov、.m4v、.ts | .mp3、.flac、.wav、.m4a、.ogg |

全部满足上游登记条件，因此此包可以为这些常见格式提供系统打开方式及默认应用候选。证据是固定来源代码与本机分类的核验，**不是本机已实际安装后逐格式点击验收**。没有 PerceivedType 或分类不同的扩展名会被上游跳过，需要用户在系统“打开方式”中选择 mpv.exe；MpvKit 不补写登记。机器摘要见 [package-association-evidence.json](package-association-evidence.json)，完整源码与查询结果保存在忽略目录 .local/windows-v019/。

Linux 的 [Arch 官方包文件清单](https://archlinux.org/packages/extra/x86_64/mpv/files/) 含 usr/share/applications/mpv.desktop；[MPV v0.41.0 原生入口](https://github.com/mpv-player/mpv/blob/v0.41.0/etc/mpv.desktop) 已声明音视频 MIME 与系统 MPV 启动命令。本版删除重复媒体入口，不修改包资产；本轮没有 Linux 实机环境。

## Windows 当前程序验证

Rust 默认最终 **60 通过 / 5 默认忽略，7.08 秒**；fmt、严格 clippy all-targets -D warnings、前端 **3/3**、TypeScript/Vite、单 EXE --no-bundle 构建通过。删除登记专属测试后数量变化不代表扩大设备覆盖；保留并补充文件卸载手改/续播保护与过期 helper 拒绝检查。Linux 管理器的 Exec 转义边界测试随实现移入 manager_entry，但本机 Windows 不编译该模块，需 Linux 补验。

离线预览完整交互检查通过，包含草稿、失败重试、焦点、双语主题、640/900/1200、200% 文字和减少动态效果，无运行异常或网络请求。普通沙箱启动 Edge 因临时文件替换受限失败，在获准的独立浏览器进程重跑通过。两端依赖材料检查均通过：Windows 319 包 / 565 份原文，Linux 356 包 / 618 份原文，无缺失材料；本轮仅同步 v0.1.9 版本标题及输入摘要，不改变依赖和许可原文，不代表完整许可审阅已完成。

原生测试修正记录：首轮新增按钮断言误用 Set default apps，真实标签为 Open Default apps，导致 2 项定位失败；第二轮 7 项通过，侧栏新增的 IPC 监测失败。固定 Tauri 2.12.4 的 core.js 用 Object.defineProperty 定义不可写的 invoke，替换监测没有生效。改为点击真实按钮并直接调用实际 IPC，Debug 隔离模式继续阻止打开主机设置；未为测试放宽产品行为。最终 EXE 的侧栏专项已通过 **1/1，4.99 秒**，并留存顶部 MPV 管理区截图，按钮可用且高级项关闭。

Windows 最终完整隔离原生回归 **8/8 通过，269.83 秒**，另含 **52 个真实 MPV 轨道子用例**，是本版最终 EXE 的新结果。覆盖核心安装/失败/取消/超时/重试/升级/卸载及归属边界、启动窗口及手调保护、uosc/倍速/播放模式/stats、自然连播/字幕、安装/手改/原生 profiles/保存、双语主题/默认应用按钮/删除 IPC、快捷键/控制台/续播/字体，以及草稿/文件时间/过期计划/640–1200 布局。包管理器使用隔离本地进程夹具，实际播放器来自已校验的 WinGet 包；系统默认应用不变。

单 EXE：src-tauri/target/debug/mpvkit.exe，**30,242,304 字节**，SHA-256 **420bfaa61275be3814cd56987c5d7d5b21c6a10659a697a72788c716480b04bf**。同步依赖材料后已重新构建，最终完整原生回归使用此产物。构建仅出现链接器输出创建 .lib/.exp 的提示，严格 clippy 通过。

本机忽略目录 .local/windows-v019/ 保留：unit-final.log、clippy-final.log、build-final.log、preview.log、sidebar-final.log、native-release-check.log（名称不代表发布构建，实际为上述 Debug 单 EXE）。前两轮失败日志 native.log 和 native-final.log 同样保留。侧栏专项截图位于 .local/e2e-artifacts/mpvkit-v016-ui-kFdDSV/mpv-default-apps.png，已查看并确认按钮位于 MPV 核心行、高级项关闭且组件未安装时可用。

完整回归证据均在 .local/e2e-artifacts/：

| 项目 | 本机证据目录 |
| --- | --- |
| 核心管理 | mpvkit-winget-Ahkyqc |
| 窗口尺寸 | mpvkit-v018-window-M3VJfx |
| uosc 与模式 | mpvkit-v018-psmq2V |
| 连播/字幕 | mpvkit-v018-autoplay-1PJvEd |
| 安装/配置/profiles/轨道 | mpvkit-v016-aHTTcv |
| 侧栏/主题/默认应用 | mpvkit-v016-ui-JqUxY8 |
| 草稿/过期/布局 | mpvkit-v016-draft-owFXcq |

精确用例、耗时、轨道临时目录、产物摘要及未测边界汇总于 [windows-native-evidence.json](windows-native-evidence.json)。目录名沿用已有测试标记，不代表引用旧版结果。

## 未测边界

真实 WinGet 安装/UAC/升级/卸载及上游动态登记的完整 Shell 行为仍需可回收 Windows 环境；本轮按用户许可用固定上游代码作简单核验，完整产品原生套件使用隔离 WinGet 进程夹具和真实 MPV。Linux 实机、物理鼠标/完整辅助技术、多屏/混合 DPI、HDR/高刷/功耗、干净系统与完整来源许可验收没有新增结论。旧实际开发版数据保持原样，未增加旧格式解析、迁移或自动清理。
