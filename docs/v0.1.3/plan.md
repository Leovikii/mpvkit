# v0.1.3 配置与目录精简方案

日期：2026-10-22。状态：用户已批准并授权实施与本地测试；根据用户最新要求移除开发版迁移和兼容，本地回归已完成，验证记录见文末。当前版本为 v0.1.3 开发测试版，不代表正式发布。

## 目标与范围

按用户最新要求分成两类：**MPV 配置负责播放；MpvKit 配置负责软件偏好和环境管理。** 不再要求用户理解 user.conf、生成 JSON 与多个画质覆盖层。用户操作入口只有“播放设置 / 打开配置目录”和“应用设置”，内部恢复记录不进入设置界面。

2026-10-22 用户追加持续约束：在其明确宣布正式发布前，不设计或启用 GitHub Actions / CI/CD，不实现任何历史开发版兼容或迁移，不构建安装包，仅构建单 EXE 测试程序。执行入口与约束详见 [AGENTS.md](../../AGENTS.md)。仅做必要的本地检查与隔离测试，不修改真实默认应用。本次用户已授权提交并推送本批更新，授权不包含发布或打包。

## 一、MPV 配置

Windows 受管理播放器采用官方支持的便携目录：mpv.exe 与 portable_config 相邻。所有普通播放参数集中到 mpv.conf；键位使用 input.conf；脚本设置遵循 script-opts 约定。两类配置不意味着强行将 MPV 的不同文件语法塞进同一个文件。

~~~text
<播放器安装目录>/
├── mpv.exe
├── d3dcompiler_43.dll
├── portable_config/                 # “打开配置目录”直接进入这里
│   ├── mpv.conf                     # 画质、音频、字幕、续播、截图等
│   ├── input.conf                   # 快捷键
│   ├── fonts.conf                   # 固定构建附带的字体配置，按实际依赖保留
│   ├── script-opts/
│   │   └── thumbfast.conf           # 缩略图参数；无需要时不创建其他脚本配置
│   ├── scripts/
│   │   ├── mpvkit.lua               # 画质菜单及保存等必要桥接
│   │   ├── thumbfast.lua
│   │   └── uosc/                    # 保留上游模块组织
│   ├── fonts/                      # 文楷与 uosc 字体
│   └── watch_later/                # MPV 自行保存的续播记录
├── licenses/                       # 组件许可与来源材料
└── .mpvkit/                        # MpvKit 内部管理文件，见下一节
~~~

- 不生成、不要求、也不再由启动器特殊加载 user.conf。用户直接编辑 mpv.conf。
- 不单独生成 profiles.conf。三档画质使用 mpv.conf 内的原生命名 profile，菜单调用原生 apply-profile；三档仍从仓库同一份定义生成，不在 Rust 和 Lua 各维护一套。
- 移除部署出的 mpvkit-profiles.json。仅保留必要 Lua 菜单/保存逻辑，不在 Lua 重新实现画质参数表。
- 移除 mpvkit-context.json。路径由配置目录和固定相对布局确定并校验，必要脚本选项通过 mpv.conf 的原生 script-opts 机制提供。安装身份和修订继续从受管理元数据读取，不以相对路径猜测归属。
- 首次写入默认音量、字幕字号、续播等参数；之后用户可以在原位置修改。GUI 没有的项目不要求额外文件或开关，也不在保存其他设置时重置。
- 续播只保存位置，仍无 GUI 开关，使用标准 portable_config/watch_later 目录。
- 固定构建的 fonts.conf 不能只搬路径：实施时核对其中的引用和字体加载行为。许可证、组件内部文件保持必要结构，不以“文件少”为由裁剪。
- 原生目录让直接启动 mpv.exe 与通过管理器启动读取同一套配置。直接启动当然不会经过管理器的启动前完整性检查；两者的配置一致性与管理检查职责分开。

## 二、MpvKit 软件配置与内部状态

软件偏好与播放参数分离。为避免切换播放器目录后丢失语言和主题，应用偏好保持在全局应用数据目录；每个播放器环境的归属、GUI 应用记录和恢复数据随该环境保存。它们都属于 MpvKit 一类，但不能硬合成跨目录的单一大文件。

已实现的新安装默认位置：

~~~text
%LOCALAPPDATA%/Programs/MpvKit/       # 管理器软件安装位置
└── mpvkit.exe 等应用安装文件

%LOCALAPPDATA%/MpvKit/               # 软件数据
├── mpvkit.json                     # 语言、主题、强调色、所选播放器目录
├── .mpvkit/                        # 全局归属与跨进程锁，保留必要记录
└── player/                         # 新安装默认播放器目录，可在高级选项自定义
    └── 上一节的播放器结构
~~~

不让用户在两个目录分别配置播放器；默认用户无需修改任何 JSON。自定义播放器位置只改变 player 所指的环境，不搬迁全局软件偏好。管理器卸载默认保留播放器和用户数据。

播放器目录下的内部结构：

~~~text
.mpvkit/
├── settings.json                   # GUI 设置来源、修订和引导确认状态
├── owner.json                      # 管理目录归属
├── install.json                    # 组件版本、文件归属、摘要和安装身份
├── associations.json               # 系统登记及原值恢复记录
├── operation.lock                  # 并发写入保护
├── pending.json                    # 部署中或中断待恢复时存在
├── association-pending.json        # 登记中或中断待恢复时存在
├── previous.json                   # 上次可恢复操作
├── backups/                        # 必要备份
├── staging/                        # 暂存，安全清理
├── receipts/                       # 暂存/备份归属证据
├── cache/sessions/                 # 有界会话记录
└── launcher/                       # 稳定启动与保存入口及其许可文件
~~~

明确的收敛动作：

| 当前文件或目录 | 目标 |
| --- | --- |
| 全局 app-settings.json + target.json | 合并为全局 mpvkit.json；继续保护目标切换与原子写入 |
| state/settings.json | 移到环境 .mpvkit/settings.json；消除顶层 state/ |
| .mpvkit/playback.json | 取消独立偏好层，播放器菜单保存与 GUI 使用同一设置事务 |
| .mpvkit/setup.json | 引导确认信息并入环境 settings.json，保留安装身份与修订语义 |
| config/mpvkit-context.json | 移除，复用相对布局、原生脚本参数与内部元数据 |
| config/mpvkit-profiles.json | 移除，三档配置使用 mpv.conf 的原生 profile |
| 顶层 cache/、launcher/ | 收入 .mpvkit，保留现有必要功能 |
| runtime/ + config/ | mpv.exe 及依赖放安装根，配置归 portable_config/ |

settings.json 仍是 GUI 结构化设置的来源，记录“软件上次应用的设置”，不伪装成任意手写配置的解析结果。它不会在每次启动时覆盖 mpv.conf。若文件被手改，GUI 应提示存在手动修改，不能将旧 GUI 值冒充当前播放器实效值。

不合并 owner/install/associations/事务恢复记录：归属、文件部署与注册表恢复的生命周期不同。它们都归 .mpvkit 内部管理，用户无需编辑。完整 EXE 助手先复用；本版不开发额外轻量助手、不新增后台服务。

## 三、直接编辑配置与 GUI 共存

原则：**播放读取 MPV 配置，GUI 仅在用户明确保存时修改它管理的部分。** 取消启动时 playback.json 的命令行画质覆盖，取消 user.conf 的特殊最后加载。用户手改主配置后不因为可编辑文本摘要变化而拒绝播放。

1. mpv.conf 内用少量注释标明 GUI 管理的范围；其他默认值和用户新增内容原位保留，不要求用户放在单独文件或专属“用户区”。命名 profile 使用 MPV 自身语法，生成顺序明确，profile 段后使用 default 段恢复普通配置作用域。
2. GUI 保存复用原有结构化设置和生成逻辑，仅修改明确归属的范围；不建立任意 MPV 配置的完整解析器。无需更新的 input.conf 和脚本选项保持不动。
3. GUI 管理范围也允许手改并正常播放。若保存时发现该范围或文件结构被手改，展示具体差异，允许取消并保留现状；只有用户确认后才备份并替换涉及范围。标记缺失、段作用域无法确认时停止自动改写，提供恢复推荐配置的明确预览，不猜测合并。
4. 对外部 include、自定义 profile 等高级配置，不宣称 GUI 能完整反映运行效果。已知文件发生手改时显示简短状态；不会为了同步表单而执行用户配置或偷偷重写文件。
5. 播放器菜单“切换画质”只影响本次播放；“保存为默认”才通过同一保存事务更新 settings.json 与 mpv.conf。冲突时保留本次效果、拒绝静默覆盖，并提示回管理器查看。旧播放器实例仍受安装代际与修订校验限制。
6. 原生 apply-profile 会修改其所含选项；应保持三档覆盖相同参数集合，并保留现有切换失败处理。HDR、语言和其他独立偏好不被画质菜单改写。用户显式切换档位会覆盖本次会话对应的画质参数，这是菜单行为，不是隐藏启动覆盖。
7. 可执行文件和受管理脚本继续做完整性检查；用户可编辑配置的摘要只用于发现修改及覆盖保护。手写配置可以加载代码，哈希检查不构成 MPV 配置沙箱；软件不自动执行未知配置来做检测。

例：用户将默认 save-position-on-quit 改为 no，之后仅在 GUI 调整音轨语言，这一续播设置应原样保留。该类非 GUI 参数不在每次保存时重生。

## 四、开发阶段不提供迁移与旧格式兼容

2026-10-22 用户明确：尚无第一个正式版，不需要迁移或兼容历史开发版。本决定取代本方案此前关于旧布局整理、旧设置合并、user.conf 导入和续播路径保留的设计。

- 只生成、读取当前布局和当前设置格式，不提供迁移按钮、旧布局路径映射或旧字段自动补全。
- 全局只读 mpvkit.json；环境设置只读 .mpvkit/settings.json。不读取旧 app-settings.json、target.json、playback.json 或 setup.json。
- 不自动导入 user.conf，不加载旧 Lua，也不转换旧续播记录。用户可自行使用 MPV 原生 include。
- 未识别的管理数据拒绝操作并保留原件；不扫描、删除或清空开发者的实际安装。开发验证从隔离空目录开始。
- 保留当前格式的具体变更预览、覆盖保护、跨进程锁、事务备份和中断恢复，这些用于防止本次操作损坏数据。
- 外部 MPV 仅引用可执行文件，在所选环境保存配置；不移动外部程序或修改它旁边的配置。

## 五、实施边界与完成标准

已按以下顺序实施：先调整可编辑配置和保存事务，再改为原生目录，最后整理应用设置与内部文件。可以分次实现，但不能在只改目录名称时宣称收敛完成。

完成应在本地隔离目录确认：新安装、直接启动/受管理启动配置一致、GUI 关闭后的菜单保存、手改保留与冲突、并发和中断恢复、字体/缩略图/续播，以及自定义目录和外部 MPV 边界。复用现有本地工具，只增加实现确实需要的检查，不设计 GitHub Actions 测试或发布流水线。

本方案不增加播放功能、额外设置开关、Linux 实现、配置编辑器、数据库或新依赖。完整设备/来源验收仍继承 v0.1.2 的未完成项；文档审核与本地验证均不等于正式发布许可。

## 官方依据与当前基线

已核对目标 MPV 提交 6a262d11f6 的官方手册。Windows 便携目录、原生配置分工、命名 profile 和 apply-profile 均有官方依据；MpvKit 元数据结构、GUI 注释范围是本项目建议，不冒称 MPV 官方规范。新布局的真实播放验证与适用边界见[实施验证](evidence/2026-10-22-implementation.md)。

- [固定版本手册：Windows 文件位置](https://github.com/mpv-player/mpv/blob/6a262d11f6/DOCS/man/mpv.rst#files-on-windows)。
- [固定版本手册：配置文件与 profiles](https://github.com/mpv-player/mpv/blob/6a262d11f6/DOCS/man/mpv.rst#configuration-files)。
- [固定版本输入命令](https://github.com/mpv-player/mpv/blob/6a262d11f6/DOCS/man/input.rst)与[脚本 API](https://github.com/mpv-player/mpv/blob/6a262d11f6/DOCS/man/lua.rst)。
- [官方在线手册](https://mpv.io/manual/stable/#files-on-windows)用于阅读参考，实施行为以固定构建验证为准。
- [v0.1.2 实际落盘结构](../v0.1.2/storage-layout.md)、[v0.1.2 实施与交接](../v0.1.2/plan.md)、[既有架构与恢复边界](../v0.1.1/architecture.md)。

## 实施与交接（2026-10-22）

- 新安装落盘为 mpv.exe + portable_config；保留固定 MPV 归档附带的 d3dcompiler_43.dll 和 fonts.conf。原生直接启动与受管理启动共用主配置。详见[实际目录](storage-layout.md)。
- GUI 保存只改标记范围，保留续播、音量和其他手写内容；手改管理范围需高级选项明确替换并查看前后文本。管理标记损坏时拒绝猜测，提供“恢复推荐主配置”的整文件预览与备份。手改状态在保存后继续提示；GUI 展示的是上次应用值，不保证等于任意 include/profile 的最终实效值。
- 新环境不生成 user.conf、profiles.conf、mpvkit-context.json 或 mpvkit-profiles.json；菜单复用原生 profile，默认档位与 GUI 共用 settings.json 事务。启动不再使用旧 playback.json 的命令行覆盖。
- 全局 mpvkit.json 保存语言、主题、强调色和所选目录，采用同一跨进程锁；不读取或导入历史记录。
- 非活动会话按归属删除日志、记录及空目录；仍在运行的进程和未知内容保留。没有新增后台服务。
- 管理器新安装默认 Programs/MpvKit；已有位置与显式 /D 保留。播放器默认位置独立为 MpvKit/player。

旧实现的验证记录保留在[历史实施验证](evidence/2026-10-22-implementation.md)，其中迁移测试与旧产物不代表当前实现。当前清理范围和重新验证结果见[移除兼容分支验证](evidence/2026-10-22-no-compat.md)。下一任务是用户复查新布局与高级选项体验，实体高刷/HDR/功耗、干净系统和完整来源验收仍待完成。不得把本地构建产物当作正式发布许可。

同日公开证据追问：研究记录补入 Artoriuz 的量化画质指标，以及 Ryzen 5 5600G 核显的 480p 两倍放大计时；已检查作者脚本并记录单次整进程计时、未固定后端/核验生效和无功耗数据等限制。公开数据支持场景化取舍，不证明全设备最优。本轮仍为文档调查，未运行下载脚本或改产品。

同日 HDR 参数说明：研究记录补充 percentile=0 的最亮像素语义、99.995 的直方图成本，以及 contrast-recovery=0.30 的局部对比恢复/额外特征图路径。依据固定 mpv 与版本字符串对应上游 libplacebo 源码，保留 dirty 构建对应性和实机 HDR/功耗缺口；产品参数未改。

同日 HDR 直通追问：研究记录补充 target/source 模式区别及 Windows 元数据传递边界；当前 HDR On 为 PQ/BT.2020 输出并沿用 target，不能称为明确直通。未改产品策略；是否引入 source 选项需按实际屏幕链路比较，不能从 HDR 标志推断画质更优。

## 本批提交交接（2026-10-22）

三项开发测试硬性约束已同步到 AGENTS.md 和 README。移除会在 push / pull_request 时运行的 Windows workflow，并将 Tauri bundle.active 设为 false，后续只使用 --no-bundle 构建单 EXE。之前的 NSIS 构建及安装目录测试为本约束生效前的历史记录，不再作为当前验收要求。本批包含 v0.1.2/v0.1.3 实现、验证与规范更新；提交推送按本次用户授权执行。

本次规范更新已通过文档链接、Git diff 和依赖清单检查；关闭 bundle 后的 Debug 单 EXE 构建通过。本次未构建安装包，未运行安装器测试。业务代码沿用上一轮 63 项后端、4 项原生与 2 项双语通过的结果，未因纯规范改动重复业务测试。

## v0.1.4 设计接续（2026-10-22）

下一版方案现已修订为[三标签与固定目录](../v0.1.4/plan.md)：取消引导/状态区分、移除自定义路径和外部播放器、配置由用户单独生成安装；管理页承载状态并保留默认开启的系统集成意图。此前两步引导建议已被替代，仍待整体审核，未改代码或版本。本轮整理旧设计文档并新增[当前基线](../current.md)。只读检查发现 integration.status_with 仍残留 current_exe 旧登记回退，已记入下一版删除项；此前“兼容全部删除”的表述不覆盖此遗漏。
