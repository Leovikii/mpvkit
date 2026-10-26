# v0.1.8：uosc 经典控制栏

2026-10-26。用户已批准本方案并授权实施及本地测试：中央播放/暂停单按钮、左右上下集、原生信息按钮和合并播放模式。源码递增为 v0.1.8。共享配置与 Lua 已实现；Windows 优先验证，本机没有 Linux 环境，不能将共用逻辑检查视为 Linux 原生通过。实际证据与限制见 [验证记录](evidence.md)。

## 基线与持续边界

实施前再次 `git fetch origin`，HEAD 与 origin/main 均为 `e41cfda94081cf34508fe0c4f66c9f9e4188ac5d`（v0.1.7 Linux 适配）。保留先前未提交的方案文档，以此最新双端基线实施 v0.1.8。

Windows x64 与 Linux x86_64 Arch/CachyOS、KDE Wayland/Vulkan 同时维护。播放器控制栏、快捷键与 Lua 逻辑共用；路径、helper、系统核心和图形后端仍经平台模块处理。Linux 原生配置目录检测与系统 MPV、Windows 按本轮追加决定使用 WinGet 核心和原生配置目录。

继续无 CI/CD、历史开发版迁移和安装包；仅允许本地单 EXE / ELF 构建。保留具体预览、摘要复核、手改保护、备份、失败恢复；不触碰真实用户配置或默认应用，不自动提交或推送。

## 已实施布局

保留 uosc 原生的“控制栏在进度条上方”结构、时间显示、章节标记和 thumbfast 缩略图。上方标题及原生窗口行为保持当前逻辑；底部控制栏分为媒体选项、直接播放操作、播放方式与视图三组。以下为本轮推荐，替代首轮左右混排的常用布局：

```text
菜单  信息   音轨  字幕       上一集  播放/暂停  下一集       播放模式  播放列表   倍速  全屏
────────────────────────── 进度条 / 缩略图 ──────────────────────────
```

| 位置 | 按钮与行为 |
| --- | --- |
| 正中 | 上一集、播放/暂停、下一集。播放与暂停为同一按钮，随 pause 状态更新；中央组按窗口中心对齐，不受左右组宽度变化影响。上下集复用 uosc/prev、uosc/next：优先播放列表，无列表时切换同目录媒体，不改为章节跳转或快退/快进。 |
| 左侧 | 菜单、信息、音轨、字幕。菜单与信息组成综合入口；音轨与字幕组成媒体轨道小组，组间轻留白。字幕靠近中央，便于看片时切换；仍保留加载外部字幕入口。没有多音轨时可收起音轨按钮，菜单仍可达，其他按钮相对次序不变。 |
| 右侧 | 播放模式、播放列表、原生倍速控件、全屏。模式靠近中央播放组，与其作用的播放列表相邻；倍速位于其后，全屏固定最右。列表复用 uosc/items，同时提供同目录文件列表；不再把列表夹在上下集之间。列表/倍速之间轻留白，避免模式与列表被倍速隔开。 |
| 侧边音量 | 沿用 uosc 右侧原生音量控件及静音入口，不为主栏另造横向音量组件。方向键音量 ±10 保持。 |
| 菜单内 | 打开文件、章节、画质、音频设备、文件位置、配置目录、退出；多视频轨/版本和流清晰度按媒体能力提供，不全部常驻。 |

分组依据是操作对象与固定位置习惯，不能严格称为“左边全是信息、右边全是播放”：音轨和字幕也是控制操作，全屏则属于视图。信息与播放模式对调有助于分组，但只互换会使倍速仍隔在模式与中央播放组之间，且轨道和信息入口次序未理顺，因此同时调整两侧内部次序。

信息使用频率相对低，但用户明确要求独立按钮，保留在左侧靠外位置；音轨和字幕成对，靠中央放置。全屏保持最右符合常见播放器位置习惯，不为追求语义纯粹把它移到信息旁。播放模式保留状态图标/悬停提示，与播放列表图标明确区分；不再增加常驻的“当前模式”文字行。视觉优先级由中央播放/暂停主按钮与组间留白表达，不要求左右严格同宽，也不强求两边按钮数量相等。

本轮只比较“按用途分组”与“原常用布局”，不继续扩展精简等多套候选，也不增加自由编排按钮的设置系统。窄窗沿用同一顺序收起次要入口：先移入信息和倍速，再视实际空间收起轨道等入口；菜单、中央三控和全屏优先保留。示意在极窄宽度采用换行以便讨论，产品仍优先单行收起，不把示意的双行结构当作已决定的 uosc 改造。

信息按钮执行 `script-binding stats/display-stats-toggle`，再次点击关闭，与 Shift+i 共用 MPV 内置 stats 面板；i 保留短暂显示。不制作另一份媒体信息窗口，也不以 info 图标自动承诺高亮能追踪 stats 状态。菜单增加同一入口，窄窗仍可访问。

## 播放模式合并建议

把目前分开的 shuffle、loop-playlist、loop-file 合并为一个“播放模式”按钮。点击打开 uosc 原生菜单，直接选择并标记当前模式；不要求用户反复点击轮换四种状态。按钮图标/提示随状态变化，仅本次播放器进程生效，不自动写入配置或新增持久字段。

| 选择 | uosc 随机 | loop-playlist | loop-file | 说明 |
| --- | --- | --- | --- | --- |
| 顺序播放 | 关 | no | no | 按列表播放，结尾沿用当前播放结束偏好 |
| 列表循环 | 关 | inf | no | 播放列表循环；手动上下集沿用 uosc 边界规则 |
| 单集循环 | 关 | no | inf | 当前文件重复；仍可手动切换上下集 |
| 随机播放 | 开 | no | no | 使用 uosc 自带随机导航，不调用 playlist-shuffle 打乱列表；自动结束和手动上下集的实际行为须实测 |

每次选择显式设置三项，避免单集循环与随机同时残留。MPV 手改/外部脚本设置有限循环次数或随机加列表循环等组合时，显示“自定义”，直到用户明确选择；读取实际状态而非维护另一份独立模式来源。无有效列表、同目录只有一项以及播完停留/退出的行为均需测试，不把单文件同目录导航冒称自动连播。暂不增加随机循环、播完退出等更多主栏模式，后者沿用现有独立偏好。

uosc 随机是其内部状态，不等于 MPV 的同名命令/选项。固定版本没有公开的真实状态读取接口；因此在经校验的原版 main.lua 末尾追加 resources/scripts/uosc-bridge.lua，直接发布 state.shuffle 和实际缩放宽度，并提供严格 yes/no 的状态设置消息。原版字节完整保留为前缀，原随机历史处理继续执行；不复制导航或另造调度器。mpvkit.lua 读取桥接状态和 MPV loop 属性，复用原生菜单及 managed button。两平台记录扩展来源/摘要，部署附带扩展源码和修改说明。

## 核对依据与最小实现入口

- 固定 [uosc 5.13.0 配置](https://github.com/tomasklaen/uosc/blob/d124c2c930d69446448022851373e00ae592390d/src/uosc.conf)、[控制栏实现](https://github.com/tomasklaen/uosc/blob/d124c2c930d69446448022851373e00ae592390d/src/uosc/elements/Controls.lua) 和 [README](https://github.com/tomasklaen/uosc/blob/d124c2c930d69446448022851373e00ae592390d/README.md)：play-pause、prev/next、items、command、button、space 及随机导航可复用。已读取本地固定材料，并核对 release ZIP SHA-256 为 `4be9da3289285300fa374496c3f1bfd7bb20ac08e890d25bd5a06b28eebe4882`，与两平台清单一致。
- 固定 release ZIP 的 Controls.lua 在恰好两个 space 时已按两侧内容宽度计算中间组绝对居中；空间不足时会从中间隐藏控件。桥接按可用宽度 / 实际缩放按钮尺寸决定信息与倍速（20）、轨道（14）、模式与列表（10）的显示门槛，先收起次要按钮，菜单保留全部入口；沿用原生侧边音量和溢出算法。Windows 640/900/1200 与运行时放大实测见证据，不声称无限小窗口仍可全部显示。
- 当前 resources/input.conf 已含 i / Shift+i 两个 stats 绑定；v0.1.7 的 Linux 实测与 Windows 未测边界见 [上一版证据](../v0.1.7/evidence.md)。循环语义参考目标 [MPV 0.41.0 手册 options.rst](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst)，Windows 实际构建为 0.41.0 开发提交，仍须分别实测。
- 新增 MpvKit 自有 script-opts/uosc.conf，只写控制栏布局；通过既有 config::relative! 和事务进入双端配置预览、保存、健康检查、备份恢复、手改与卸载保留规则。上游示例 uosc.conf 仍不部署；ZIP、其他脚本、字体和 helper 未改，main.lua 仅追加已声明扩展。
- resources/scripts/mpvkit.lua 增加模式菜单、真实状态按钮及窄窗倍速菜单入口，resources/input.conf 增加信息、模式、倍速和媒体能力菜单项。独立预览导入同一 uosc.conf，不添加依赖、持久模式字段或管理器设置页。

## 实施后的验证要求

1. 全新隔离根下，两端配置均落在检测根 script-opts/uosc.conf。覆盖预览、过期计划、手改、备份恢复以及未改变文件不重写；不做此前开发版字段补全/迁移。
2. Windows WinGet 实际核心与 Linux 系统 MPV 分别实播：中央按钮与空格切换暂停；视频背景单击忽略，双击仅切换全屏且不触发短暂暂停；上下集、同目录边界、无列表、单文件、播完停留/退出；信息按钮/i/Shift+i 交叉操作；四模式切换和外部组合状态。
3. 640/900/1200 宽、窗口/全屏、字体/DPI 放大及简中/英文，检查中间组居中与存活、菜单不被视频背景点击穿透、字幕/音轨动态变化和缩略图；界面模拟不能替代 MPV 原生截图与交互。
4. 运行变更相关配置/集成测试、格式与静态检查；构建仅 `pnpm tauri build --debug --no-bundle`。本机 Windows 不能替代 Linux Wayland 实测，缺少该主机时明确记录待测，不宣称双端全通过。

## 本轮交接

已完成共享配置、桥接和模式菜单、版本递增及单 EXE 构建。具体检查结果及截图以 [验证记录](evidence.md) 为准。补齐已有 MPV 模拟程序缺少的三个 HDR 能力选项，使启动保护测试与 v0.1.7 能力要求一致，没有放宽产品校验。

2026-10-26 用户追加授权：取消视频背景单击播放/暂停，解决双击全屏时先触发暂停的问题。两平台共享 input.conf 改为 MBTN_LEFT ignore，保留 MBTN_LEFT_DBL 全屏；中央 uosc 按钮、空格、菜单和时间轴继续使用各自绑定。独立预览直接导入同一资源；不增加延时判断或手势设置。

## 追加：统一启动窗口尺寸

2026-10-26 用户批准按建议实施：共用 mpv.conf 默认加入 `autofit=75%x75%` 与 `auto-window-resize=no`。启动窗口在屏幕宽高 75% 范围内按视频比例适配，低分辨率视频也使用统一启动尺度；切换不同分辨率或比例的文件时保留当前窗口大小。手动缩放后切集、进入/退出全屏保持用户调整后的尺寸；切集时的视频通过 MPV 原生保持比例处理，不拉伸。

参数依据为已读取的 [MPV v0.41.0 官方手册](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst)。百分比由 MPV 结合屏幕、任务栏与原生边框计算，不保证客户区恰好等于整块显示器的 75%；完整窗口边框必须位于桌面可用区内。Windows 实测见证据；Wayland 不强制窗口位置，原生尺寸仍待 Linux 设备验证。

实现只修改共用生成器默认区及能力检查；预览示例同步，两项可由用户手改且 GUI 保存时保留。Linux 配置断言同步，没有引入 GUI 字段、跨进程窗口记忆、自定义路径或 Lua 窗口管理器。测试过程的 Win32 窗口矩形查询/调整仅加入既有测试 helper，仍先核对精确 PID 与可执行文件，不进入产品。

下一任务为 Linux 原生构建与 Arch/KDE Wayland 实播、真实多媒体与设备验收。现有 HDR/能耗/干净系统/许可等缺口继续保留；没有提交、推送、CI、安装包或历史开发版迁移。

## 截图功能删除（2026-10-26）

用户明确要求完整删除。已移除截图目录、格式和字幕开关、双语文案、设置字段、配置生成和能力检查、Lua 动作及 uosc 菜单入口；同时删除只供此功能使用的两平台目录选择器和图片目录查询（Linux 不再调用 xdg-user-dir）。不保留隐藏开关或旧格式兼容，不清理用户既有图片。thumbfast 进度条缩略图保留。测试中的界面取证仍用于验证布局，不属于产品截图功能。验证见本版 evidence。

本批单 EXE 已重建；Rust 57 项通过（6 项默认忽略），前端 2 项和 Windows 原生 6 项全部通过，格式/静态检查、离线预览和两平台依赖材料检查通过。Linux 专用代码已同步删除相关测试，尚无原生构建或 Wayland 实播。下一任务仍为可用 Linux 环境下的构建/配置/播放验证及既有真实设备验收；无提交或推送。

## WinGet 核心管理与原生控制台（2026-10-26，已实施）

用户授权继续开发：Windows 使用 WinGet 安装和管理 MPV，并新增控制台快捷键及右键入口，完成可行的 Windows/Linux 检查。本轮 Windows 固定包为 `shinchiro.mpv`，源为 `winget`，按该包的版本发布节奏主动安装/升级；不跟随每日构建，不引入自动升级、额外下载回退或核心卸载。Linux 保留当前 `/usr/bin/mpv` 及原生目录检测，没有新增 pacman 提权安装流程。

缺少健康核心时，安装操作先显示独立 WinGet 预览，列出源包元数据与核心/配置位置，明确包协议、可能的管理员授权和包自己的“打开方式”登记；用户确认后才执行固定参数。核心完成后重新检测实际版本及必需能力，再继续原有增强预览/安装及配置保存。已有健康核心直接复用；高级选项→播放器组件提供独立“安装 / 更新 MPV 核心”。WinGet 缺失提供 Microsoft App Installer 入口。取消、失败、源不可达、输出超限、超时或虚假成功均报错并保留草稿；超时可能仍有包安装子进程，提示检查系统状态后重试。核心独立于配置事务，配置失败保留系统核心；不声称能回滚第三方安装器。

预览有效期五分钟，执行前复核核心长度/修改时间与路径、配置位置及 WinGet 路径，使用既有跨进程锁。只读固定 Inno 包卸载身份与 Program Files 已知目录，不扫描磁盘或通用 PATH；多处实际核心阻断。配置优先非空绝对 MPV_HOME，再使用已存在的相邻 portable_config，否则 Windows RoamingAppData/mpv。相对路径、重解析点、安装目录和用户目录的实际配置歧义阻断，包附 mpv/fonts.conf 可保留。两端配置、增强、归属与备份/helper 均在检测根，Lua 同步从该根定位。核心不进入增强文件归属，不覆盖、恢复或卸载核心；外部核心变更会使配置/启动旧计划失效。移除 sevenz-rust2 及旧核心自下载/解包分支，依赖锁与两平台材料已同步。

控制台使用 MPV v0.41 原生 `script-binding commands/open`：反引号（Esc 下方）打开，Esc 关闭；右键菜单在播放信息后增加“控制台”。没有独立控制台页面、旧版别名或自制输入处理。实测打开时空格、上下键、Enter 不传给播放器；Esc 后音量键恢复。

渠道只读核验：WinGet 0.41.0 当前安装器来自 0GMou/mpv2winget，实际提取核心为 **v0.41.0-244-gaf9c81fa1**。包标签并不代表精确稳定标签提交；该渠道更新滞后的风险仍存在。已经下载并按清单校验、提取和实播，**没有在开发者主机执行真实安装器、UAC 或包升级**。模拟 WinGet 只在新隔离根复制同一已验证核心，测试失败/取消/升级/保护；不能代替干净 Windows 上的真实安装验收。详细证据及本批 EXE 摘要见 [evidence.md](evidence.md)。

依据：[WinGet 清单](https://github.com/microsoft/winget-pkgs/blob/master/manifests/s/shinchiro/mpv/0.41.0/shinchiro.mpv.installer.yaml)、[封装脚本](https://github.com/0GMou/mpv2winget/blob/master/.github/workflows/build_mpv.yml)、[MPV v0.41 input.conf](https://github.com/mpv-player/mpv/blob/v0.41.0/etc/input.conf)、[Windows 原生配置实现](https://github.com/mpv-player/mpv/blob/v0.41.0/osdep/path-win.c)、[原生 console/commands](https://github.com/mpv-player/mpv/tree/v0.41.0/player/lua)、[WinGet 错误码](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerSharedLib/Public/AppInstallerErrors.h)。未精确验证完整上游提交，不将包标签当作证明。

本轮交付仍为单 EXE，无提交、推送、CI、安装包或历史开发版迁移。下一任务：可用隔离 Windows 虚拟机上的真实 WinGet 安装/UAC/登记与升级验证；有 Linux 环境后补 ELF 构建、系统核心及 KDE Wayland 实播，之后继续既有实体 HDR/高刷/功耗和许可验收。

本批最终验证：Rust 59 通过/6 默认忽略，前端 2 通过，Windows 原生 7/7 通过（含 52 实际轨道子用例），fmt/clippy、单 EXE 构建、离线预览和两平台依赖材料检查通过。EXE 摘要、逐项证据及未测项见 evidence。Linux 仅共享/静态与材料检查，没有原生构建或 Wayland 实播；真实 WinGet 安装器/UAC/第三方登记待隔离虚拟机验证。

## 播放结束行为移入主配置区（2026-10-26）

按用户要求，结束行为从高级选项移至中间 MPV 配置面板，并在后续讨论后明确命名“全部播放结束后”。沿用 keepOpen 草稿与 mpv.conf 的 keep-open=yes/no、既有保存/预览保护及默认关闭；两平台共用界面。独立预览同步。最终顺序与新增连播字段见下节，验证见 evidence 的本次追加。

## 自动连播与字幕搜索（2026-10-26）

用户已授权实现自动连播开关，字幕规则优先参考 MPV 官方文档/代码，再参考开源预配置目录名。主区依次为画质 → HDR 输出 → 自动连播 → 全部播放结束后。新增严格当前格式 playback.autoPlaylist，默认 true，不补读此前开发版缺少字段的设置，不迁移真实数据。Mantine Switch 以 checked/onChange 控制，支持空格切换并关联说明；产品和独立预览一致。

启用生成 autocreate-playlist=filter，关闭生成 no；该行属于 GUI 管理区。原生 directory-filter-types=video,audio、directory-mode=ignore 只载入媒体所在目录的音视频，不载入封面、字幕、归档、播放列表或子目录；顺序采用 MPV 的自然文件名排序（1、2、10），从选中的媒体开始，前面的文件仍可手动回看。关闭只停止自动构建目录队列，显式播放列表仍连播。keep-open=yes 仅在最后一项结束时停留，no 则退出，不使用会阻止切集的 always。uosc 原生同目录浏览与前后切换保留，不启用 uosc autoload 或新增 Lua 扫描器。

字幕可手改默认区显式 sub-auto=exact，支持相同媒体主文件名及原生识别的语言、forced/SDH 等后缀，不使用 all 加载其他集字幕。以 sub-file-paths-append 逐项追加 sub、subs、subtitle、subtitles，均相对媒体目录；Linux 追加 Sub/Subs/Subtitle/Subtitles 及全大写形式，Windows 原生大小写不敏感，只追加四项防重复扫描。媒体目录本身仍由 MPV 搜索，不递归扫描任意目录。不增加字幕目录 GUI；用户可以直接修改这些默认参数，GUI 保存保留手改。两个平台能力检查增加对应原生参数，夹具与 Linux 专用断言同步。

依据：[MPV v0.41.0 options](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst)、[external_files.c](https://github.com/mpv-player/mpv/blob/v0.41.0/player/external_files.c)、[language.c](https://github.com/mpv-player/mpv/blob/v0.41.0/misc/language.c)、[目录自然排序](https://github.com/mpv-player/mpv/blob/v0.41.0/demux/demux_playlist.c)。辅助参考 [Argon-/mpv-config](https://github.com/Argon-/mpv-config/blob/master/mpv.conf)、[wopian/mpv-config](https://github.com/wopian/mpv-config/blob/master/mpv.conf) 的目录名称，未复制整套配置、模糊匹配或平台特定路径。界面核对 [Mantine 9.7.3 Switch 官方文档](https://mantine.dev/core/switch/) 与本地声明，使用 aria-label、aria-describedby、disabled 和原生键盘行为，无新依赖。

实现已完成：Rust 59 通过/6 默认忽略，前端 2 通过，Windows 完整原生 8/8 通过（含 52 个轨道子用例），fmt/clippy、离线预览、两平台依赖材料和单 EXE 构建通过。实际队列、EOF、关闭开关、显式播放列表及字幕语言/forced/SDH 后缀均已验证，最新 EXE 摘要与证据见 [evidence 追加记录](evidence.md)。

下一任务：有 Linux 环境后补原生构建、大小写字幕目录、队列及 KDE Wayland 实播；可用隔离 Windows 虚拟机后补真实 WinGet 安装/UAC。Linux 条件代码与参数已审阅，但没有原生环境，不把共享测试当作 Linux 实播通过。既有真实媒体、HDR/功耗和来源许可验收继续保留；无提交、推送、CI 或安装包。
