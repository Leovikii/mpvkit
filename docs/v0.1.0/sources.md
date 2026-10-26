# 来源政策与配置决策

更新日期：2026-10-20。以下通用链接是研究入口，固定研究候选见文末及来源报告；候选不等于正式组件清单，也不代表运行验证通过。

最新补充：[选中模块声明与生成输入](evidence/2026-10-20-helper-module-review.md)核对22个模块文件、7个uosc文件及SDK生成定义；一项x/sys生成输出与固定归档逐字节一致。两个其他生成器未固定历史Windows SDK输入，完整对应源码/许可审阅仍未完成，未修改产品组件。

## 信息来源的职责

2026-10-26 v0.1.8 集成补充：uosc 仍固定 5.13.0 / d124c2c930d69446448022851373e00ae592390d，下载 ZIP 未改。该版没有公开的随机状态 getter；为准确同步外部状态并按真实缩放收起可选按钮，在部署的 main.lua 原版前缀后追加 MpvKit 自有 uosc-bridge.lua，GPL-3.0-or-later，注明日期和修改内容。两平台 catalog 标记 modified=true，source-materials 的 uoscIntegration 分别记录原版、扩展、合并结果摘要，部署附源码和修改说明。其余上游脚本、字体、helper 和导航算法不改。MpvKit 自有 script-opts/uosc.conf 与未部署的上游示例分开记录。实现依据、验证及 Linux 待测边界见 [v0.1.8](../v0.1.8/plan.md)。

| 信息 | 首要依据 | 补充依据 |
| --- | --- | --- |
| MPV 参数、配置加载、Lua/IPC 语义 | 目标版本官方手册及对应实现 | 官方变更记录、问题讨论、实际构建 |
| MPV 渲染行为 | MPV 与其使用的 libplacebo 版本 | 目标后端和设备实测 |
| uosc、thumbfast 的 API 与安装 | 各组件上游文档和发布资产 | 社区集成实例 |
| 第三方 shader | 采用的具体实现上游、代码、许可证 | 原算法说明、独立对比与本地测试 |
| 界面、菜单和整合体验 | 成熟社区方案 | MpvKit 的范围与用户验证 |
| 默认推荐和能效 | 上述技术依据加目标设备实测 | 社区经验用于提出候选 |

“第一手来源”针对具体内容。社区作者原创或修改的实现，其仓库就是该实现的直接来源；不能把原版效果结论无条件套到改版，也不应无条件拒绝有明确收益的社区补丁。

## 官方入口

- [MPV 手册入口](https://mpv.io/manual/)：区分稳定版本与开发版。
- [MPV 源码](https://github.com/mpv-player/mpv)：按目标 tag/commit 查看。
- [官方内置 profiles](https://github.com/mpv-player/mpv/blob/master/etc/builtin.conf)：仅作定位入口，正式依据换成目标提交链接。
- [MPV 安装入口](https://mpv.io/installation/)：寻找平台发行与构建来源，不把第三方 Windows 构建误称为官方发布二进制。
- [libplacebo 上游](https://code.videolan.org/videolan/libplacebo)：仅在需要理解渲染或版本差异时深入。
- [uosc 上游](https://github.com/tomasklaen/uosc)。
- [thumbfast 上游](https://github.com/po5/thumbfast)。

先查手册，查询实际构建能力；只有文档不足、行为不一致或依赖变更时再追代码，避免为了每个选项进行无限研究。

builtin.conf 不是完整默认配置。普通选项默认值在实现中，实际构建可通过 --list-options 等查询；该查询也不能替代播放期间实际状态检查。high-quality 不意味着全部处理阶段成本最高，fast 也不自动等于正确的移动设备方案。

## 社区项目的使用方式

### hooke007/mpv_PlayKit

[项目](https://github.com/hooke007/mpv_PlayKit)适合作为中文说明、Windows 使用经验和增强方案的研究入口。[着色器说明](https://github.com/hooke007/mpv_PlayKit/wiki/4_GLSL)明确指出其中包含修改和自制实现。

采用方式：

- 学习使用条件、菜单交互、后端限制与常见问题。
- 逐项追溯想采用的脚本或 shader，区分原版与社区变体。
- 选择其修改版时记录实际收益、与原版的差异和来源。
- 不把整个 shaders 目录作为未经验证的官方组件库。

### dyphire/mpv-config

[项目](https://github.com/dyphire/mpv-config)适合作为完整配置组织、脚本组合、快捷键、菜单与条件 profiles 的参考。

采用方式：

- 阅读实际启用项及后续覆盖关系，不能只依据注释。
- 区分作者个人偏好、Windows 特有项、兼容性 workaround 与一般必需项。
- 对发现的集成问题建立测试场景，再决定引入最小配置或补丁。
- 不继承整套默认画质策略，也不把“作者机器上适合”当作跨设备最佳值。

两个项目都是重要参考，不是 MpvKit 的整体上游依赖。首版默认不 fork 整个配置仓库，不自动同步其主分支。

## 引入一个设置的流程

1. 写清要解决的具体问题，先确认 MPV 或组件默认行为是否已覆盖。
2. 查目标版本语义、平台限制、即时修改能力和与其他参数的关系。
3. 在最小基线上只增加该项，验证能重现并解决问题。
4. 检查对核显本、台式机、HDR 和下一版跨平台边界的影响。
5. 有必要才纳入配置，记录来源与验证证据；无收益则保持默认。

不要把社区 workaround 无条件推广到所有 GPU。问题消失、上游修复或新版本行为改变后，应重新评估能否删除该覆盖。

## 配置决策记录模板

首次确定真实参数时在此文档追加条目；当前不填未经验证的推荐值。

~~~text
ID / 解决的问题：
参数或方案及生效阶段：
相对上游默认的变化：
官方依据（版本/固定提交链接）：
社区参考（如有）：
适用平台、MPV/依赖版本、前置条件：
性能/画质及兼容性取舍：
验证报告：
回退方式：
复核触发条件：
~~~

## 组件清单要求

实现 resources/catalog.json 时至少包含：

- 稳定组件 ID 和用途。
- 来源项目与实际发布/移植项目；是否为 fork 或变体。
- 固定版本/tag/commit、平台/架构和下载资产。
- SHA-256、校验信息的来源。
- 许可证标识、对应文本/署名位置及需要保留的材料。
- 最小已知能力要求、已验证的 MPV 版本/后端。
- 必要依赖和与其他组件的冲突。
- 是否修改、补丁位置与修改原因。

首版只处理固定少量组件的依赖关系，不建设通用版本求解器。上游发布包包含辅助程序时一并审查和部署；不因主要实现是 Lua 就假定没有平台二进制。

## 更新与供应来源

仓库研究可看最新分支，发布目录必须固定版本与哈希。第三方组件从选定可信来源获取，不直接执行上游一键安装脚本，也不在每次启动自动抓取 main。

首版用随应用发布的已验证清单驱动组件更新。新增远程清单会引入真实性、降级与兼容管理需求，属于后续独立工作，不用一个可写 URL 临时替代。

哈希保证资源与清单一致；清单本身也要来自可信的 MpvKit 发布。不能把“下载后自己算了一次哈希”宣称为来源真实性验证。

## 许可证与署名

保留仓库现有 LICENSE。精确项目许可证声明在生成发布元数据前核实，不自行推断 GPL 的 only/or-later 选择。

复制代码、配置片段、文档或 shader 时分别检查其适用条款；聚合仓库顶层许可证不能自动覆盖全部第三方内容。需要再分发时保留原许可、版权声明、修改记录，以及适用的源码和构建材料。仅参考设计思路与直接复制文件应分别记录。

首个真实第三方资产引入时建立实际声明清单，不提前编造已分发组件或虚假的版本。

## M1-001 / 原生默认值与隔离查询

- 问题：应用骨架需要检查用户选择的 MPV，并启动独立环境；M0 尚未验证画质参数。
- 方案：查询使用 --no-config、--load-scripts=no；启动增加显式 --config-dir、--no-terminal 和 -- 媒体参数边界。当前不复制配置片段或脚本，不启用画质档位。
- 固定依据：[mpv v0.40.0 options.rst](https://github.com/mpv-player/mpv/blob/v0.40.0/DOCS/man/options.rst) 中 --no-config、--config-dir、--load-scripts、--terminal 语义；[同版本 mpv.rst](https://github.com/mpv-player/mpv/blob/v0.40.0/DOCS/man/mpv.rst) 的命令行语法。--no-config 优先于 --config-dir，所以当前不能声称已验证配置目录内文件的加载。
- 适用边界：这只是待真实构建验证的启动契约参考，不是已选定 v0.40.0 发行资产，也不代表所有版本兼容。运行前检查必要参数；未知能力拒绝使用，未来 M0 再固定正式构建与依赖版本。
- 验证：[M1 记录](evidence/2026-10-20-m1.md) 中模拟播放器/实际 IPC 检查；真实 MPV 的加载、原有 portable_config/MPV_HOME、SDR/HDR 与组件行为未验证。
- 回退：取消预览不写文件；本切片不替换已有配置或外部 MPV。确认启动失败时保留会话记录。
- 复核触发：M0 选定构建、M2 接入生成配置，或发现版本能力/加载行为差异时重新验证。正式引入第三方资产前仍须满足组件清单与许可要求。

## M0-001 / 固定来源研究候选

已从 MPV 官方安装入口核实 shinchiro 构建来源，固定 MPV 20261020 x86_64 资产（mpv 提交 6a262d11f63733d37e3ee6d5fafcb1be1ef83952）、uosc 5.13.0，以及 thumbfast 提交 0f711de3138c9bd6718209d819ac54022c23ded2。资产摘要、提交、包内容和许可证入口见 [来源报告](evidence/2026-10-20-m0-sources.md)。

MPV/uosc 下载摘要与固定 GitHub Release Asset 元数据一致；thumbfast 下载字节与固定提交 Git blob 一致。只读检查归档、未执行播放器/安装脚本。uosc 包含字体和 ziggy-windows.exe，不能仅部署 Lua；MPV 候选包和 uosc.zip 均未携带完整许可证材料，正式分发前须另行补齐并核对依赖。尚不建立应用可用 catalog.json，不把候选当已验证组合。

这使 M0 从未开始进入来源研究阶段；真实版本/构建依赖、隔离加载、SDR/HDR、缩略图与目标设备结果仍待获取。

## M1-002 / 显式实机验收入口

- 问题：现有模拟调用测试无法证明真实 MPV 的配置隔离和启动者退出行为，需要可重复、默认不执行真实播放器的实验入口。
- 方案：实机测试显式选定播放器与测试目录；在全新子目录中用不同 volume 哨兵、自动脚本正向控制和合成 PCM 检查。生产 no-config/load-scripts=no 契约不变。正向控制单独使用 config=yes；null 输出、length=0.1、terminal=yes、term-playing-msg、cache-dir/state-dir 仅用于测试，不进入应用推荐或生产启动。
- 固定官方依据：[目标 MPV 提交 options.rst](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/DOCS/man/options.rst)。本轮只读核对：config-dir 覆盖其他配置目录及 MPV_HOME，no-config 优先；cache/state 路径仍有独立探测逻辑，因此正向控制额外指定测试目录。load-scripts=no 禁止自动脚本；term-playing-msg 在播放开始后展开属性；length 限定片段时长，audio-display=no 关闭音频封面显示。
- 验证与边界：[验收入口证据](evidence/2026-10-20-m1-validation.md)。普通测试仅证明测试代码可执行和保护流程；真实构建尚未运行，不因此认定目标版本、配置或解码通过。使用此入口前仍需用户明确指定目录与可信程序，分发材料要求不变。
- 回退与复核：不修改生产启动契约、不引入第三方资产；实验失败保留记录。真实运行发现差异时修正入口并记录，之后才能更新 M1.5/1.6 状态。

## M1-003 / 自动隔离测试与 MPV_HOME 更正

用户 2026-10-20 明确要求自动完成全部测试，开发实验据此默认选择仓库 .local/v010-tests 或新建临时子目录，不重复等待提供路径。不得使用这一安排改写开发者真实配置、默认应用或注册表。

固定 MPV 候选已实际运行：v0.41.0-1224-g6a262d11f，依赖版本见 [实机与部署证据](evidence/2026-10-20-m2-core.md)。测试发现 M1-002 中所写 cache-dir/state-dir 不是该构建的选项，已删除，不作为产品能力。固定 [mpv.rst](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/DOCS/man/mpv.rst) 的 Windows 配置说明指出 MPV_HOME 同时覆盖用户配置、cache 和 watch_later 位置；生产启动在子进程中将其设为会话目录，config-dir/no-config/load-scripts 契约保持。

配置哨兵、自动脚本控制、合成 PCM 解码以及真实 GUI 关闭后的进程存续已通过；不能由此宣称视频渲染、HDR、硬解、缩略图或完整分发许可通过。完整构建材料仍待补齐，正式组件清单尚未建立。

## M2-001 / 带发布门槛的开发组件目录

为继续自动实现和隔离验证，resources/catalog.json 记录固定资产及来源/许可/依赖/冲突与实际验收范围；releaseReady=false，并明确列出未完成许可材料及设备/干净机门槛。它是开发基线，不表示完成再分发许可或发布验收。下载契约与来源目录通过一致性测试核对，不加载可写远程清单。此前“尚不建立清单”记录保留为历史，当前以此条为准。

已对真实上游未修改资产完成 HTTPS 哈希、安全解包和图形组件实验；仅下载和部署明确文件，不包含/执行上游安装更新脚本。真实图形和新原生交互记录见后续证据；MpvKit 自身 GPL 精确版本声明 O09 未改变。

## M5-001 / 来源与许可材料保留

当前固定构建日志ZIP已核对上游GitHub artifact大小和SHA-256；FFmpeg日志声明GPL version 3 or later。resources/source-materials.json保存精确日志身份和已核对许可文本摘要，resources/licenses保留原始字节。完整安装将这些材料及来源清单部署到licenses，并使用同一事务保护覆盖/恢复/卸载；使用外部MPV时不声称提供其构建许可。

日志能证明构建选项和部分依赖版本，不能代替完整可重建聚合源码。uosc两份字体已核对固定Git blob，图标字体有Google版权/Material Icons Round名称，但完整许可映射、纹理字体声明与Go辅助程序依赖材料尚待齐备。已有Apache文本不被自动套用于全部字体。releaseReady继续为false；本项目O09精确授权仍待维护者决定，LICENSE未改。当前实测以[汇总证据](evidence/2026-10-20-v010-functional.md)为准，前文来源实验的未运行状态作为历史保留。

## M5-002 / 应用依赖许可证据

已为Windows解析图（含构建期）297个Rust包和4个前端生产包收集546份许可/NOTICE，生成[清单](../../resources/app-dependency-inventory.json)与[原文](../../resources/app-dependency-notices.txt)。五个包外许可来源按固定提交或发布源码明确声明补齐；[复验与缺口](evidence/2026-10-20-app-notices.md)记录具体依据。此材料尚未接入安装包，不能替代附带原生库、工具链、组件完整源码/声明或O09授权审阅；releaseReady继续为false。

## M5-003 / uosc Windows辅助程序内嵌依赖

固定发布ZIP中的ziggy-windows.exe内嵌Go1.23.1和clipboard/browser/x/sys三模块版本与h1，均与固定go.sum对应。已保留三模块许可及Go LICENSE/PATENTS，纳入现有事务；来源/摘要与具体版本见[核对证据](evidence/2026-10-20-uosc-helper-materials.md)。元数据不证明完整可重建性，Go标准库内第三方及源码、字体许可映射仍待齐，releaseReady不改变。

## M5-004 / 应用原文随安装包保留

应用依赖原文/清单已通过明确资源映射随NSIS安装，新增固定Rust标准库版权汇编与匹配实际静态库字节的WebView2 SDK许可/NOTICE，并保留仓库原始LICENSE。版本、官方包SHA-512、库SHA-256与暂存/安装脚本核对见[分发证据](evidence/2026-10-20-app-distribution.md)。它补充前述“尚未接入”的历史状态；完整源码义务/字体及O09等门槛继续未完成，不把已保留原文等同完整合规审阅。

## 2026-10-20 项目授权决定（当前）

用户明确选择GPL-3.0-or-later（第3版或以后），O09已关闭。[项目许可声明](../../COPYING.md)与Cargo/npm/Tauri元数据同步，并随安装包分发PROJECT-LICENSE.md；原LICENSE正文和第三方许可保持原始字节。此决定补充前文O09未决的历史记录，不解决MPV聚合源码、uosc字体/Go完整材料或许可兼容性审阅缺口，releaseReady继续为false。用户确认暂时只有当前电脑，保留系统与设备验收缺口；[本轮证据](evidence/2026-10-20-project-license.md)记录交接。

## 2026-10-20 Mantine与官方开发文档

按用户要求新增core/hooks9.7.3，实际组件用法遵守下载到仓库的[官方LLM索引和页面](../vendor/mantine/README.md)，其非版本化URL以快照摘要保存，另核对固定包类型和构建。当前应用依赖范围变为321包/566份材料；包内或已核对原文已经随原安装包映射保留。新增react-remove-scroll-bar2.3.8的npm登记gitHead不可访问，包只声明MIT，精确发布对应原文未补齐；同项目2.3.7许可候选只作研究，不冒称已对应2.3.8。详见[迁移与许可证据](evidence/2026-10-20-mantine.md)。前文301包/546份材料为迁移前历史状态，原有上游/系统/设备发布门槛继续保留。

## M5-005 / 固定uosc与Go源码归档（当前）

已取得Go1.23.1官方源码（官方发布摘要一致）、三个实际helper模块源码（h1匹配内嵌信息和固定go.sum），以及固定uosc完整源码（全部94文件匹配固定Git树blob/大小/模式）。新[只读核对入口与机器结果](evidence/2026-10-20-uosc-source-archives.md)可离线复验，现有保留许可也与归档逐字节匹配；Go的33份独立声明已列位置与摘要。此前“源码尚待取得”的状态由本条补充；逐文件版权/链接适用、helper可重建性、字体及其他完整分发审阅仍未完成，releaseReady继续为false。MpvKit后端仍为Rust，Go是uosc第三方辅助程序使用的工具链；新增Python只用于可选开发核对，不是产品运行时。

## M5-006 / uosc图标字体精确映射

uosc_icons.otf与Google material-design-icons固定提交1c575764826fa5752ddc2bdc6b31ef7cc3233371的font/MaterialIconsRound-Regular.otf逐字节一致，400360字节、SHA-256 a853f50913597e67f76facd269ff230c2f3038f9acadec04dd653080785dee25，未修改。固定提交README明确按Apache-2.0提供图标；LICENSE与已保留原文字节一致。source-materials.json记录精确URL、版权及许可摘要，目录/组件说明同步；纹理字体继续不套用此许可。可重复原文件比较入口为node scripts/verify-uosc-icons.mjs，具体步骤、实际结果和仍有缺口见[字体证据](evidence/2026-10-20-uosc-icon-font.md)。

## M5-007 / Go源码额外声明与纹理来源历史

已从官方固定Go源码保留33份独立声明和11个已审阅源文件段落，保持版权/条件/免责声明原始字节；收集器默认只读check，source-materials及组件事务纳入新合集。[范围与证据](evidence/2026-10-20-go-source-notices.md)说明为什么保守保留不等于实际链接或完整合规结论。纹理字体的来源/对齐/重命名记录已对应固定blob；PR222明确JSON工程未改，精确许可和重建缺口继续保留。此条补充此前“未取得声明”的历史状态，不关闭完整L/W/P门槛。

## M5-008 / 第三方helper离线重建与源码选择

已用固定Go1.23.1官方便携SDK重建ziggy，全部输出留在仓库新隔离目录。158包/952选中源码中的951文件与固定Go/模块/uosc归档逐字节对应，另1个为已校验SDK的生成文件；所有内嵌构建信息相同，但二进制大小/摘要不同，reproducibleBinary仍false。入口、4项边界测试、离线模块元数据警告及失败修正记录见[重建证据](evidence/2026-10-20-uosc-rebuild.md)。这补充此前“未做重建”的历史状态，不证明上游二进制完整链接图或关闭版权/许可审阅；没有替换运行时组件，MpvKit技术栈仍为Tauri/React/Mantine/Rust，Go仅属于第三方helper。

## M5-009 / 固定helper运行时元数据对应

官方Go标准库只读PE核对进一步得到：上游与重建程序5902个函数的名称/顺序/偏移/长度一致，583项规范化源码路径一致，其中582项匹配此前已验证源码、1项为自动生成标记；[工具、测试及机器结果](evidence/2026-10-20-uosc-link-metadata.md)保留固定摘要和比较范围。代码节/多个数据节字节不同，仍不构成可复现或语义等价证明，不关闭完整分发审阅。固定版本ziggy源码历史也已记录，不能据此冒称上游二进制具有VCS构建声明。工具为可选维护代码，不改变Rust后端、产品资源和构建依赖。

## M5-010 / 同包相同代码的MIT原文保留

react-remove-scroll-bar2.3.8的npm gitHead不可访问，但已核对其与2.3.7的26个代码/声明/说明文件字节相同，根清单仅版本及已单独收集的依赖范围不同；2.3.7固定gitHead与附LICENSE提交30源码文件相同、只新增MIT正文。该具体代码对应性支持保留同一项目对相同代码的授权原文，不按版本相近推断，也不把公开HEAD称为2.3.8固定提交。[输入/逐文件机器记录](evidence/2026-10-20-scroll-bar-license.json)与[复核/风险测试/构建证据](evidence/2026-10-20-scroll-bar-license.md)保留完整边界。当前应用321包/567份材料、缺失原文项为空，完整适用审阅及其余L/W/P仍未完成，releaseReady=false。

## 固定构建的缓存来源边界

当前[完整回归及来源核对](evidence/2026-10-20-current-regression.md)已验证三份固定CMake配方Git blob；其未指定GIT_TAG，libplacebo日志为复用缓存、只记版本7.377.0，日志完整40位值属于Clang编译器。版本号、今天的HEAD或编译器提交不能替代该库实际缓存对应源码；保留固定构建的源码提交/快照、补丁与生成输入缺口。此有限核对不宣称全部依赖已审阅或不存在其他上游证据，completeCorrespondingSource与releaseReady继续false。

## M5-011 / 实际选中Go输入的来源原文

复核952个选中输入摘要后，确认AES参考实现作者/完整免责声明和amd64指数函数的历史公有领域来源此前未保留，现按固定源码完整字节范围追加。33独立声明+13选段，共46条/90469字节；原88051字节全部保持。[选段清单](evidence/2026-10-20-go-notice-excerpts.json)与[验证证据](evidence/2026-10-20-go-selected-notices.md)记录来源、完整范围和旧前缀保护。不是用当前SLEEF版本替代历史声明，也不把二进制预计算表或生成文件误称全部许可审阅已完成；完整Go/原始构建、纹理字体、MPV聚合及W/P门槛仍保留。

## M5 / 立方根实现SunSoft声明（当前）

固定923个standard输入的头部/摘要核对发现cbrt.go署名为SunSoft，不与既有SunPro原文混同，已按310:684完整字节追加。16处相同数学授权和fiat README对应不重复追加；现47条/90968字节。范围和验证见[本轮证据](evidence/2026-10-20-go-sunsoft-notice.md)，生成来源/完整适用与原始构建、MPV/uosc其余材料及W/P继续未完成。


## 当前 v0.1.8 系统核心边界（2026-10-26）

Windows 不再分发或解包固定 shinchiro 每日核心，改为 WinGet 的 shinchiro.mpv 包；Linux 保留系统 /usr/bin/mpv。MpvKit 仅分发固定增强、字体与自身程序，原核心/FFmpeg 材料保留作历史证据，不再随增强部署。WinGet 的来源、实际包哈希、提取后二进制版本及测试限制见 [v0.1.8 证据](../v0.1.8/evidence.md)。包管理器校验安装器，不保证包标注版本对应上游稳定标签的精确提交；运行能力另行复核。uosc/thumbfast/字体仍需按固定来源与摘要校验，不放宽来源政策。
