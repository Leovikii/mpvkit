# 当前落盘结构与精简建议

核对日期：2026-10-22；代码基线 v0.1.2。本文为只读架构盘点与建议，尚未实施新的目录迁移或移除 user.conf 加载行为。以代码生成器、部署事务、目标选择、启动参数、固定组件内容及本机隔离安装清单交叉核对。没有扫描或修改用户真实安装。

## 根目录关系

- A（应用全局数据）：Windows KnownFolder LocalAppData/MpvKit。应用语言/主题、当前受管理目标在这里；选择自定义目录不会迁走这些全局设置。
- R（受管理安装）：默认 R=A，也可以是用户选择的自定义目录。播放器、播放器配置、归属和恢复数据保存在这里。
- P（管理器软件安装）：NSIS 当前用户安装默认同样为 LocalAppData/MpvKit，安装器可选择其他位置；便携 EXE 则为用户实际放置位置。默认安装时 P、A、R 可能重合，这是应整理的职责边界。

A/.mpvkit 按操作需要产生 owner.json、operation.lock、selection.lock、app-settings.json、target.json。应用设置首次保存才生成 app-settings.json；没有切换目录时 target.json 可以不存在，代码使用默认目标。

## R 中固定部署文件（完整）

下列为受管理 MPV 全套新安装的 81 个归属文件，不包含运行期状态、用户自建文件与安装器文件。第三方 uosc 的所有模块列出，避免把依赖目录误认为多余配置。

~~~text
<R>/
├── config/
│   ├── fonts/
│   │   ├── LXGWWenKaiScreen.ttf
│   │   ├── uosc_icons.otf
│   │   └── uosc_textures.ttf
│   ├── input.conf
│   ├── mpv.conf
│   ├── mpvkit-context.json
│   ├── mpvkit-profiles.json
│   ├── script-opts/
│   │   └── thumbfast.conf
│   └── scripts/
│       ├── mpvkit.lua
│       ├── thumbfast.lua
│       └── uosc/
│           ├── bin/
│           │   └── ziggy-windows.exe
│           ├── char-conv/
│           │   └── zh.json
│           ├── elements/
│           │   ├── BufferingIndicator.lua
│           │   ├── Button.lua
│           │   ├── Controls.lua
│           │   ├── Curtain.lua
│           │   ├── CycleButton.lua
│           │   ├── Element.lua
│           │   ├── Elements.lua
│           │   ├── ManagedButton.lua
│           │   ├── Menu.lua
│           │   ├── PauseIndicator.lua
│           │   ├── Speed.lua
│           │   ├── Timeline.lua
│           │   ├── TopBar.lua
│           │   ├── Updater.lua
│           │   ├── Volume.lua
│           │   └── WindowBorder.lua
│           ├── intl/
│           │   ├── de.json
│           │   ├── es.json
│           │   ├── fr.json
│           │   ├── it.json
│           │   ├── pl.json
│           │   ├── pt.json
│           │   ├── ro.json
│           │   ├── ru.json
│           │   ├── tr.json
│           │   ├── uk.json
│           │   ├── zh-HK.json
│           │   └── zh-hans.json
│           ├── lib/
│           │   ├── ass.lua
│           │   ├── buttons.lua
│           │   ├── char_conv.lua
│           │   ├── cursor.lua
│           │   ├── fzy.lua
│           │   ├── intl.lua
│           │   ├── menus.lua
│           │   ├── std.lua
│           │   ├── text.lua
│           │   └── utils.lua
│           └── main.lua
├── launcher/
│   ├── LICENSE.txt
│   ├── PROJECT-LICENSE.md
│   ├── mpvkit.exe
│   └── third-party/
│       ├── APP-DEPENDENCY-NOTICES.txt
│       ├── RUST-COPYRIGHT-library.html
│       ├── WEBVIEW2-SDK-LICENSE.txt
│       ├── WEBVIEW2-SDK-NOTICE.txt
│       ├── app-dependency-inventory.json
│       └── app-distribution-materials.json
├── licenses/
│   ├── LXGWWenKaiScreen-OFL.txt
│   ├── THIRD-PARTY.txt
│   ├── catalog.json
│   ├── ffmpeg-GPL-3.txt
│   ├── material-icons-Apache-2.txt
│   ├── mpv-Copyright.txt
│   ├── mpv-GPL-2.txt
│   ├── mpv-LGPL-2.1.txt
│   ├── source-materials.json
│   ├── thumbfast-MPL-2.txt
│   ├── uosc-LGPL-2.1.txt
│   ├── uosc-browser-BSD-2.txt
│   ├── uosc-clipboard-BSD-3.txt
│   ├── uosc-go-BSD-3.txt
│   ├── uosc-go-NOTICES.txt
│   ├── uosc-go-PATENTS.txt
│   └── uosc-sys-BSD-3.txt
├── runtime/
│   ├── d3dcompiler_43.dll
│   ├── mpv.exe
│   └── mpv/
│       └── fonts.conf
└── state/
    └── settings.json
~~~

config/mpv.conf 是 MPV 主配置；input.conf 是键位及 uosc 菜单动作；script-opts/thumbfast.conf 指定播放器路径等缩略图参数。mpvkit-context.json 为 Lua 提供根路径、保存助手路径、初始档位、截图是否含字幕；mpvkit-profiles.json 是三档画质的生成数据，不是 MPV 原生 profiles.conf。

state/settings.json 是 GUI 的结构化播放器设置，当前 schema 3。launcher/mpvkit.exe 由复制正在运行的完整 MpvKit EXE 得到，不是一个独立精简启动器；它用于稳定的打开方式入口以及 GUI 关闭后的档位保存。launcher 下的声明与随该副本分发的代码有关，licenses 下的声明对应下载组件，两者职责不同。

当前不会自动生成 user.conf、profiles.conf 或 script-opts/uosc.conf。user.conf 只在用户自行创建或已有时被额外加载。默认生成的 mpv.conf 仍建议使用它，但用户已明确希望改为直接编辑主配置；对应启动校验和覆盖保护尚待调整。

## R 中按操作产生的文件

~~~text
<R>/
├── .mpvkit/
│   ├── owner.json                    # 此控制目录的归属标记
│   ├── operation.lock                # 安装/更新/保存操作的跨进程锁
│   ├── install.json                  # 文件哈希、原始备份、组件版本和安装身份
│   ├── setup.json                    # 引导完成状态及已确认设置的摘要
│   ├── playback.json                 # 在播放器菜单中保存的全局画质档位；不是播放进度
│   ├── associations.json             # Windows 打开方式登记及恢复原值的信息
│   ├── association-pending.json       # 系统集成事务进行中或待恢复
│   ├── pending.json                  # 文件部署事务进行中或待恢复
│   ├── previous.json                 # 上一成功操作的恢复记录
│   ├── backups/
│   │   └── <操作 ID>/
│   │       ├── <数字编号>            # 被替换文件的原始字节
│   │       └── previous.json         # 被替换的上一成功操作记录，按需生成
│   ├── staging/
│   │   └── <操作 ID>/<数字编号>       # 新文件的已校验暂存字节
│   └── receipts/<操作 ID>.json       # 暂存/备份的归属与摘要，供安全清理
├── cache/
│   └── sessions/<会话 ID>/
│       ├── session.json              # 会话归属、ID、MPV 版本
│       └── session.log               # 最多三行启动事件，不是完整 MPV 输出日志
└── config/
    ├── user.conf                     # 仅用户已有/自建时存在；不会由默认安装创建
    └── watch_later/
        └── mpvkit-position/<媒体标识> # MPV 退出续播记录，只保存 start
~~~

默认 R=A 时，全局 app-settings.json、target.json、selection.lock 也会出现在上述 .mpvkit 中。自定义 R 时，它们仍留在 A；自定义 R 不因是安装目标就复制一份应用设置。

pending.json 与 association-pending.json 在正常完成后移除，异常中断时保留用于恢复。锁文件可能留在磁盘，文件存在不代表仍被锁定。原子写入时，目标文件同目录会短暂出现 .mpvkit-<操作 ID>.tmp。

组件下载当前读取进内存后校验、解包，没有固定 downloads/ 或持久化 .zip/.7z 缓存目录。staging 存的是部署字节，不能与下载归档混称。暂存和过期备份通过维护预览清理，保护原始备份及当前恢复引用；并非一律在安装结束时自动删除。

日志清理目前仅删除旧 session.log，保留最近 20 份，不删除 session.json 或会话目录。日志数量达到 1024 会要求清理；目录项另有 8192 上限。长期累积的会话目录是需要改善的已确认问题。

## P 与其他位置

管理器通过 NSIS 安装时，P 下包含 mpvkit.exe、uninstall.exe、LICENSE.txt、PROJECT-LICENSE.md 与 third-party/ 下六份应用声明/清单（名称与上述 launcher/third-party 相同）。默认 P=A=R 时，这些文件与受管理目录位于同一根目录。便携运行不自动产生 uninstall.exe。安装器还写入 Windows 安装登记、开始菜单等系统位置，它们不属于播放器配置目录。

截图默认写入 KnownFolder Pictures/MpvKit，由 MPV 首次截图时创建；高级选项可以指定其他本地目录。诊断导出写入用户明确选择的 JSON 文件，不固定在 R 中。正常运行不自动生成完整 mpv.log；本机证据里的 mpv.log 是测试显式开启的。

thumbfast 在 Windows 临时目录使用 thumbfast.out<PID>、.bgra 及临时 .tmp 文件，并使用命名管道通信；它们属于缩略图进程的运行数据。WebView2 也有框架维护的浏览器运行数据，本产品没有把它当成 MPV 配置或纳入部署清单；其动态内部文件不适合硬编码进产品目录约定。

旧平铺布局 configDirectory='.' 会将上面所有 config/ 子项直接放到 R 根目录，其余目录保持。外部 MPV 模式不部署上述 runtime/ 文件，所选 mpv.exe 始终在外部原位置；该模式也不附带四份 MPV/FFmpeg 组件许可文本。未知文件保留，因此已有安装实际内容可能多于本表。

## 优化建议（未实施）

1. **先调整配置归属，再整理目录。** 允许直接编辑 mpv.conf、input.conf 与脚本选项；启动时检查必需文件与可执行代码，不再因为可编辑配置哈希变化而拒绝播放。GUI 更新需保留未知选项和注释，识别重复键/include/profile 等不确定情况时提示冲突，不能声称支持任意配置的无损智能合并。必须覆盖时保留具体预览和备份。
2. **收拢管理数据。** 将顶层 state/settings.json 移到 .mpvkit/settings.json，缓存也可收拢到 .mpvkit/cache/。用户打开配置目录时，只应接触 MPV 可理解的配置及脚本目录。
3. **减少多重配置来源。** 目前有效画质可能来自 settings.json、生成的 mpv.conf、playback.json 的启动覆盖，随后还有 user.conf。播放器菜单保存与 GUI 应走同一保存逻辑，目标是减少独立 playback.json 带来的优先级层。迁移时必须保留跨进程互斥、代际校验及用户手改保护，不能直接删文件。
4. **处理生成 JSON。** mpvkit-context.json 可改由启动参数/环境或脚本选项传递必要数据；mpvkit-profiles.json 可改为共享源生成的脚本内数据，或在确有需要时生成 MPV 原生 profiles.conf。保持一个档位定义源，不在 Rust/Lua 各手写一份。删除两个 JSON 本身节省空间很少，主要收益是降低用户困惑。
5. **评估原生便携布局。** 若希望直接启动 mpv.exe 也自动读取相同配置，可将 MPV 与 portable_config 放成相邻项，替代当前 runtime/mpv.exe 与独立 config/。当前受管理启动靠 --config-dir 指定配置，直接双击 runtime/mpv.exe 不保证使用这套配置。这是一次明确的迁移任务，不应在更新时静默搬动用户目录。
6. **保留必要的恢复记录。** owner/install/setup/associations、事务日志与备份承担不同的归属、引导与恢复职责，不建议为了文件少而硬合成一个 JSON。优先改进暂存和会话清理：处理已完成且确认归属的会话目录，保护活动进程和未知文件；只保留真正需要的恢复链。
7. **分离管理器安装与播放器环境。** 默认 P、A、R 重合容易让用户分不清卸载和移动范围。管理器软件建议使用常规 Programs 目录；全局偏好和用户选择的播放器环境分清职责。稳定助手仍有实际用途，先保留，后续仅在体积/启动成本确有需要时将保存入口提取为轻量助手；不为减少一个 EXE 破坏脱离 GUI 播放及系统打开方式。

建议先做 1–3 与缓存清理，再决定是否一并迁移为原生 portable_config 布局。许可证据不作为可删杂项，第三方 uosc 目录也不做无依据裁剪。

## 代码依据

- [配置生成](../../src-tauri/src/config.rs)、[组件映射与许可](../../src-tauri/src/components.rs)。
- [部署与恢复](../../src-tauri/src/deploy.rs)、[清理归属](../../src-tauri/src/deploy/cleanup.rs)。
- [目标选择](../../src-tauri/src/targets.rs)、[应用设置](../../src-tauri/src/app_settings.rs)、[引导状态](../../src-tauri/src/setup.rs)。
- [播放器启动](../../src-tauri/src/launch.rs)、[菜单档位保存](../../src-tauri/src/preferences.rs)、[会话清理](../../src-tauri/src/logs.rs)。
- [安装包设置](../../src-tauri/tauri.conf.json)、[安装器钩子](../../src-tauri/installer/hooks.nsh)。
