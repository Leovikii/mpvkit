# v0.2.0-beta.2 实施与交接

更新：2026-11-02。接续 dev `475070f`（Fix Wayland startup sizing）。用户已批准方案、正式实施与自行测试；随后明确“没有旧用户，不需要旧版兼容代码”。源码版本已更新为 **0.2.0-beta.2**，已公开预发布，标签 `v0.2.0-beta.2` 固定 `5f62599`，已有标签不移动。用户随后授权完成测试后推送 dev、PR 到 main，通过 CI 后合并发布 beta.2，再将 dev 同步到 main；Arch 构建与软件包检查交由 Actions CI。

beta.1 的发布、窗口修复及原始验证保留在[原交接](../v0.2.0-beta.1/plan.md)和[证据](../v0.2.0-beta.1/evidence.md)。本版验证单独记在 [evidence](evidence.md)，普通测试只用新隔离根。

## 已批准范围与实现

| 事项 | 本版决定与实现 |
| --- | --- |
| 预设更新 | 整套配置共用一个整数修订号，页内提示，由用户手动重新应用；不自动覆盖 |
| 重新应用 | 保留当前页面选择，覆盖预设手改，复用统一安装的预览、锁、过期检查、事务与恢复 |
| uosc | 删除外观 bridge 和专用倍速/四模式 Lua；固定上游原文，只通过公开配置与菜单接口集成 |
| 管理器组件 | SVG 管理箭头、ActionIcon 导航、自动连播 Checkbox、选择框关联可见标题；保留现有组件体系 |
| 版本 | package.json、Cargo.toml/lock、tauri.conf.json 更新到 beta.2，依赖版本不变 |
| 旧版兼容 | 不实现旧配置迁移、旧桥接识别/清理、旧字段推断、兼容层或双路径；从本版全新安装验证 |
| 发布拆分 | 本版仍由程序携带固定预设和组件来源；独立远程预设更新暂不实现 |

## 整套预设修订提示与手动重新应用

### 单一修订号

`src-tauri/src/config.rs::PRESET_REVISION` 当前为 1，标识整套自有预设：mpv.conf 默认内容、画质 profiles、input.conf 快捷键/菜单、uosc/thumbfast 配置和自有 Lua。实际预设变化才递增；管理器展示改动不递增，第三方组件仍使用自身版本记录。

`src-tauri/src/deploy.rs::InstallManifest.preset_revision` 保存到现有 `.mpvkit/install.json` 的 `presetRevision`。首次完整安装或完整重新应用才在同一事务中记录；普通保存、helper、组件局部修复保留原值。可选值表示尚未完整应用预设，不用于旧版迁移；检测不自动补写。失败、取消或事务恢复不能留下新标识。文件摘要仍负责手改检测，设置 revision 仍负责保存并发，不将三者混用。

`src-tauri/src/setup.rs` 比较标识并返回 `presetStatus`：较小为 outdated，未记录为 unknown，相同或更大不提示更新，不建议覆盖降级。未安装不显示预设提示。未来若独立交付预设包，可将常量来源替换为包元数据并复用安装记录和操作流程；本版不增加远程清单、逐文件版本或历史库。

### 已实现的手动流程

1. 配置页主设置上方显示非阻断信息。启动、重新打开、回焦只读检查，不自动弹窗或写配置，不阻断播放。
2. 提示处和高级区的“重新应用配置”调用同一 `preview_install(reapply=true)`；保留当前草稿选择。普通保存继续保护可手改默认区。
3. 复用 `config/portable.rs` 完整生成模式，预览显示实际根、文件变更和配置差异。用户确认后才执行同一安装事务，进度和结果留在原弹窗。
4. 重新生成已列明的预设文件和自有脚本，健康组件不重新下载/安装。不清空配置根；未知内容、其他脚本和续播不因重新应用而删除。受保护脚本修改、归属冲突、组件缺损、路径边界仍按既有规则阻断。
5. 无内容或修订变化时不写入。预览后即使原先无变化的文件被修改也判过期；此保护位于共享部署入口，覆盖所有调用方。

### 提示交互（已接受）

- 使用配置页内 Mantine 信息样式，提示和按钮放在一起；不藏到高级项，不用短暂通知代替。
- 非紧急提示显式使用 role=status / aria-live=polite，不抢焦点；实际错误继续使用 alert。
- 取消、失败后保留提示；完整应用成功并复核后消失。任务中禁用重复提交。
- 不新增忽略版本、已读状态、提醒偏好或第二次应用确认。

### 提示文案

| 场景 | 中文 | 英文 |
| --- | --- | --- |
| 修订较旧 | 预设已更新 | Preset updated |
| 未记录完整应用 | 可重新应用配置 | Configuration can be reapplied |
| 操作 | 重新应用配置 | Reapply configuration |
| 确认风险 | 按当前设置覆盖配置手改，无备份。 | Replaces manual configuration edits using current settings. No backup. |

产品不解释修订号、摘要或版本管理代码。必要风险在具体预览中说明，不省略实际目标和覆盖后果。无历史备份、撤销或智能合并。

## uosc 原生配置

部署固定 uosc 5.13.0 原始 Lua、字体和 helper；`components.rs` 不再追加任何 bridge。两端 source-materials 记录原始 main.lua 的一致摘要及独立配置摘要，保留许可原文与来源缺口，不改变许可证或 releaseReady 声明。旧桥接源文件与专用部署材料已删除，不实现已有旧桥接部署的迁移。

| 项目 | 当前设计 |
| --- | --- |
| 左侧 | 上一集、播放/暂停、下一集，统一 44/44/44，使用上游原生动作 |
| 右侧 | 字幕、音轨、倍速图标、播放列表、更多、全屏；字幕/音轨按原生媒体条件显示 |
| 进度条 | 原生底部独立一行；保留章节、拖动、缓存和 thumbfast，不合并到按钮栏 |
| 尺寸 | controls_size=44、controls_margin=12、controls_spacing=6；接受上游图标比例、悬停、背景和缩放 |
| 倍速 | command 的 speed 图标通过公开 show-submenu-blurred 打开 input.conf 的“倍速 / Playback speed”子菜单；六档 0.5/0.75/1/1.25/1.5/2，不显示动态倍速文字或当前项勾选 |
| 随机与循环 | 原生菜单中独立的随机、列表循环、单集循环，允许组合；不保留互斥四模式或状态同步 Lua |
| 窄窗 | 接受上游原生收起逻辑；更多菜单保留同一倍速、轨道、列表及低频入口 |

自有 mpvkit.lua 保留画质切换、保存偏好与 Linux 首次窗口修复；调用 uosc 仅用公开菜单消息。背景单击忽略、双击全屏、空格暂停、i/Shift+i stats、反引号及右键菜单的控制台入口保留。菜单不重新加入截图或独立 uosc 自更新。

依据为固定上游 README 的 controls/input.conf、show-submenu 和 shuffle 说明，以及本版固定源码；更早调查仅作[追溯](../v0.1.8/uosc-window-review.md)。本版不 fork、不改内部方法、不读取内部状态实现产品功能、不自绘，也不新增布局/兼容框架。测试私有副本的取证探针不进入部署文件。非破坏性上游更新仍须核对所用公开接口并定向回归，不能承诺未来永久兼容。

## 管理器组件审查

### 官方依据与审查范围

核对 `src/App.tsx`、`src/style.css`、`src/app-updates.tsx`、`src/task-progress.tsx` 与用户截图。Mantine 依据仓库[官方索引](../vendor/mantine/llms.txt)及实际 Button、Menu、Group、NativeSelect、Checkbox、Accordion、Modal、Tooltip、Progress、Alert 正文，另查官方 [ActionIcon](https://mantine.dev/core/action-icon/) / [Switch](https://mantine.dev/core/switch/)。线上为 9.7.4，项目仍固定 core/hooks 9.7.3，按安装包声明/实现核对 API，不升级或格式化第三方快照。

WinUI 3 依据微软 [Buttons](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/buttons)、[Icons](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/icons)、[Menus](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus) 和 [Toggle switches](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/toggles)。采用设计和无障碍原则，由 Mantine 实现。

### 管理按钮图标错位

原 rightSection 使用字符 `⌄`。Mantine 的 flex 已居中，但字形及标签文字裁剪可能造成视觉偏低。本版替换为本地 16px SVG，通过 rightSection 原生居中、currentColor、aria-hidden；不新增图标库或 top/translateY 补丁。只有打开菜单一个动作，保留 Button + Menu，不做 SplitButton。

### 组件取舍

| 组件 | 已批准处理 |
| --- | --- |
| 设置/返回 | ActionIcon，2rem 命中区和 20px SVG；保留 ref、可访问名称及键盘 Tooltip |
| 自动连播 | 改为 Checkbox，保留草稿和保存流程；即时保存的预览版更新仍用 Switch |
| NativeSelect | 保留原生选择框，通过 aria-labelledby 关联可见行标题，不另封装表单 |
| 预设提示 | 页内 Alert 显式 status / polite；无自动弹窗 |
| GitHub、Accordion、Paper、Group、AppShell | 保留；Accordion order={3} 是固定版本有效 API，Control 不嵌套交互控件 |
| Menu、Modal、Tooltip | 保留组件键盘、Esc、焦点锁和 returnFocus；验证菜单到任务及关闭后的回焦，不自造焦点管理器 |
| Progress、Loader、共享样式 | 保留；已知总量用真实百分比，未知用 Loader，按实际窄窗问题定向修改 |
| 溢出滚动 | 两页使用 ScrollArea，任务/更新使用 Autosize；8px 轨道、细中性滑块、仅纵向溢出时显示。高对比系统色、键盘可达；长文本随外层滚动，不叠加原生 pre 滚动条 |

实际 App 与 `docs/v0.1.6/preview` 共用组件/样式，预览 fixtures 和检查脚本同步更新。双语、主题、最小/默认窗口、200% 文字、键盘、高对比与原生 DPI 按证据区分，不以截图证明辅助技术或双端原生通过。

2026-11-02 用户补充：统一处理页面溢出滚动条，依据 Mantine ScrollArea 官方快照和微软 Scroll viewer controls，参数及取舍写入[界面规范](../design/ui-guidelines.md)。播放器测试新增固定 FFmpeg 合成纯白视频，保留 1200/900/640 三档控件和倍速菜单截图，检查亮背景下的图标描边、进度条和菜单可读性；只测试原生外观，不增加 uosc 自定义衬底或渲染 Lua。

### uosc 外观交互预览（2026-11-02）

用户希望亲自调节可见性与美观的平衡，已准备并打开 `.local/beta2/uosc-preview/start.cmd` 隔离预览。配置入口 `config/script-opts/uosc.conf`，1/2/3 切白底/黑底/彩色动态测试图，编辑保存后 R 重启读取（返回白底），关闭播放器结束。暂停时固定展示控件/进度条仅用于预览，无 uosc Lua 修改或新增调节 GUI。

用户最终选择上游默认：`text_border=1.2`、controls 背景 opacity=0。产品与预览均删除这两项覆盖，直接沿用固定上游默认；0.55 试值不采用。媒体与配置均在预览私有目录，既有真实配置不受影响。

### 中英文排版与边缘检查

用户要求按标准核查英文底部裁切，并同时检查中文、对齐和安全距离。复查微软 Typography/Content layout/Buttons 及固定 Mantine Button 的 root/inner/label 公开样式。遵循可读性、常规字体、简短文案、一致留白与控件对齐；数值阈值按项目明确，不声称 Web 实现是 WinUI 像素级认证。

| 检查 | 发现与处理 |
| --- | --- |
| 中英按钮裁切 | 旧 height=auto 与 label height=100%/line-height=1/text-box-trim 组合，使 19px DOM 文字范围超出约 14px 标签盒，中文及英文均有上下裁切。共享样式改完整行盒/1.5 行高/不裁剪字体，最小高度沿用 Mantine sm 36px，删除重复页面/弹窗规则 |
| 对齐 | 文字与 SVG 各用正常布局居中；设置行控件仍统一右边缘，换行后同样对齐。无 top/translateY 逐控件补丁 |
| 边缘距离 | 实测文字须在标签内，离按钮四边至少 4px（项目检查阈值）；普通字号上下约 8px；沿用 sm 水平留白、按钮间距 8 和卡片内边距 16 |
| 字体 | Windows 使用 Segoe UI Variable → Segoe UI → Microsoft YaHei UI，补齐已声明的系统字体优先顺序；Linux 系统字体保留 |
| 弹窗外层滚动 | 原自定 max-height 与 Mantine 默认 yOffset 不匹配，放大时出现第二条原生滚动条；删除重复高度计算，用组件原生最大高度与内层 ScrollArea |
| 回归 | 两页双语、480/560/720、明暗、100%/200%，补任务弹窗双语/480/200%；检查实际文字范围/边距/居中、箭头中心、按钮/行边缘及外层溢出，另在真实 WebView 复验更新和中英页面 |

## 发布解耦评估

当前管理器、预设与组件来源契约继续一起交付；组件从固定上游版本和摘要下载。配置改动频率暂不足以支持额外的远程兼容、签名和中断管理成本，先完成手动重新应用。后续若需要频繁独立发布，优先评估“管理器 + 整套预设包”，预设包固定组件版本；不使三部分分别追随最新版。本版不提前建设相关基础设施。

## 本轮交接

已实施预设修订提示、保留选择的完整重新应用、uosc 原生配置和已批准的组件优化，源码版本 beta.2。用户后续明确排除旧版兼容，故未实现旧配置迁移/桥接恢复。新预设从全新隔离根验证；历史兼容性不作为本版目标。

构建、静态检查、53 项 Rust 默认测试、固定组件部署、19 项前端/发布契约和离线 UI 矩阵通过；11 项 Windows 原生场景经整套与定向复验通过，覆盖新增预设、滚动条与白底播放器。验证、产物位置、中途失败及未测边界统一见 [本版证据](evidence.md)。[PR #16](https://github.com/Leovikii/mpvkit/pull/16) 已在双端 CI 通过后合并发布，正式签名与匿名下载核验通过。发布证据及摘要见本版 evidence；通过文档 PR 更新公开入口后，将 dev 与 main 同步。beta.1 公开标签与历史证据保持不变。

下一入口为目标 Arch/KDE Wayland 上复验 beta.2 原生播放/GTK WebKit，以及需要真实系统环境的安装更新、权限、DPI/辅助技术项目；这些不伪称当前 Windows 夹具已通过。本批已有明确发布授权，Arch 以 CI 构建和隔离包验收为准；目标桌面、真实提权/安装更新等未测项继续如实保留。

## 发布后 dev 修正（2026-11-02）

用户批准先修复、自行测试并提交推送 dev；不合并 main、不建 tag、不发布。公开 beta.2 标签及发布材料不变，源码版本暂保持 beta.2，当前预设修订从 1 增至 2。以下决定覆盖上文已发布批次的安装顺序与倍速按钮设计。

- 安装入口先展示实际目录、组件文件清单、配置差异与覆盖风险；预览不下载、不启动包管理器、不写配置。点击“安装”一次确认后才下载/校验/安装，进度与结果留同一弹窗，不再追加应用确认。系统授权仍由 UAC/Polkit 在实际需要时处理。修复安装和重新应用共用此流程。
- 核心缺失时先运行原生包管理器，失败/取消不继续获取组件或部署配置。下载期间保留取消入口，取消/失败不写配置；已成功安装的系统 MPV 不自动回滚卸载。
- resources/component-files.json 只记录固定组件实际部署文件的路径和 SHA-256，来自已校验的固定归档/原始文件，用于离线具体预览，不是配置版本系统或远程清单。共用 deploy 计划保存原目标摘要与归属守卫，确认后向同一计划补齐已验证字节；不重新准备一个会接受新文件状态的计划。组件升级必须同步清单，并用固定归档专项核对。
- 下载后的归档摘要、解压边界、文件集/逐文件摘要继续核对；不符、目标变化或新冲突停止，不扩大原确认范围。未补齐载荷不能创建事务。继续保留锁、手改保护、过期计划与失败恢复。
- uosc 左侧仍为上一集、播放/暂停、下一集；右侧顺序为播放列表、字幕、音轨、更多、全屏。音轨条件从 has_many_audio 改为 has_audio，单音轨也显示，无音轨隐藏。倍速仅在更多菜单的六档子菜单中提供。原生进度条、44/12/6 尺寸与留白及上游默认外观不变。
- 接受上游从中间向两侧收起的规则，不加独立优先级逻辑或外观 Lua；实际窄窗/缩放下验证音轨、更多和全屏的可用性。

验证结果与未测边界见[本版证据](evidence.md#发布后-dev-修正验证2026-11-02)。

本批实现与本地复验已完成：55 项 Rust 默认测试、固定归档专项、19 项前端/发布契约、离线 UI 矩阵及 4 项 Windows 原生回归通过。仅提交推送 dev，下一入口为 Arch/Linux 对应验证；没有 main 合并或新发布。
