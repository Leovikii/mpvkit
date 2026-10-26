# 当前开发基线：v0.1.8

v0.1.8 最初基于远端 v0.1.7 `e41cfda`；本批维护精简继续基于此前已推送的 `eb84250` 实施，见 [v0.1.8 计划](v0.1.8/plan.md)。始终同时维护 Windows x64 与已支持的 Linux x86_64 / Wayland；本轮 Windows 优先实测，Linux 原生构建与 Wayland 实播待有环境后补测。

实施与验证见 [v0.1.8 计划](v0.1.8/plan.md) / [证据](v0.1.8/evidence.md)。未正式发布；继续遵守无 CI/CD、无历史开发版兼容、只构建单程序（Windows EXE / Linux ELF） 的约束。

继承 v0.1.7 的原生 Linux 配置检测、系统 MPV 与 Wayland/Vulkan，Windows 改为 WinGet 系统核心及 MPV 原生配置目录。新增共享 script-opts/uosc.conf 和 uosc 状态桥接；main.lua 原版前缀保留，扩展源码、修改说明及两平台摘要随部署保存。播放模式仅当前进程生效，外部非标准组合显示自定义。

已实施（2026-10-26）：Windows 用 WinGet 源中的 `shinchiro.mpv` 安装和主动更新核心；无后台更新、每日核心下载或自下载回退。缺少 WinGet 时提供 App Installer 入口。先预览包元数据、核心/配置位置，再调用固定包安装或升级，最后复核实际核心能力，配置与增强另行执行。取消、失败、超时不写配置，已安装核心保留；增强卸载不卸载系统核心。Linux 现可经 pkexec 调用固定 pacman mpv 操作；保留依赖检查，不递归卸载依赖，不单独刷新包数据库。详细结果与安装器/UAC 未测边界见 [当前计划](v0.1.8/plan.md)。

## 当前界面与操作

侧栏只有“播放器设置”和“应用设置”。首次即可编辑画质/HDR 与高级偏好；草稿切页或安装失败时保留，关闭时提示放弃。组件可单独安装，也可在配置预览后安装组件，再确认配置与登记。只有所有必需组件健康才可应用配置，保存前复核文件摘要，手改管理区域在具体差异预览后覆盖；只保留当前失败操作恢复，不留历史备份。

底部固定“预览配置”和“保存配置 / 安装并保存配置 / 修复并保存配置”。未变化且配置健康时禁用保存；预览始终可用。主状态显示 mpv.conf 的真实修改时间，预览显示对应文件自己的 mtime，到秒；未变更文件不重写。

主配置区依次为画质、HDR 输出、自动连播、全部播放结束后；组件明细、增强卸载、预设重置及核心安装/更新/卸载统一放在顶部组件区；高级选项保留目录、来源和系统打开方式意图。自动连播默认开启，用 MPV 原生 autocreate-playlist=filter 按文件名自然顺序载入同文件夹音视频，从所选文件开始连续播放，不进入子目录。关闭后不自动扩展文件夹，用户明确提供的播放列表仍正常连播。keep-open=yes/no 只决定队列最终结束时保留窗口或关闭，不阻止中间切集。安装/配置目录使用同一个设置行：左侧“打开安装目录 / 打开配置目录”，副标题实际路径，右侧“打开”；无自定义路径。应用设置包含双语、主题、强调色、关于和诊断。遵守 [界面规范](design/ui-guidelines.md)。

字幕保留 MPV 原生 sub-auto=exact：匹配媒体主文件名，允许 MPV 识别的语种及 forced/SDH 等后缀。除媒体同目录外，追加 sub、subs、subtitle、subtitles；Linux 另外追加首字母大写和全大写形式，Windows 使用原生不区分大小写规则。逐项 sub-file-paths-append 避免两端路径分隔符差异，不递归扫描。字幕规则与目录过滤放在 mpv.conf 可手改区，保存 GUI 设置时保留用户修改。

截图功能已完整删除：无设置字段、菜单、快捷键、Lua 动作、配置参数或图片目录查询；thumbfast 进度条缩略图继续保留。不保留旧字段兼容，也不清理用户已有图片。

## 当前目录

单程序由用户放置；下列为 Windows 布局。不规定管理器安装目录，不生成安装包。

~~~text
%ProgramFiles%/MPV Player/          WinGet 包的默认核心位置；以精确登记为准
├── mpv.exe / mpv.com              系统包管理，不归 MpvKit 所有
└── ...                            由包维护；MpvKit 不覆盖或卸载
%LOCALAPPDATA%/MpvKit/
├── mpvkit.json                    应用语言、主题、强调色
└── .mpvkit/                       应用偏好归属及写入锁
%APPDATA%/mpv/                     常规配置根；MPV_HOME/相邻 portable_config 优先
├── mpv.conf / input.conf
├── script-opts/                   thumbfast.conf、uosc.conf
├── fonts/                         文楷与 uosc 字体
├── scripts/                       mpvkit.lua、thumbfast.lua、uosc/
├── watch_later/                   MPV 按需生成，仅保存位置
├── licenses/                      增强及应用许可/来源
└── .mpvkit/
    ├── settings.json              GUI 设置、修订、手改标记、集成意图
    ├── owner.json / install.json  配置根归属、增强摘要/版本/安装代际
    ├── operation.lock             配置、部署、启动、登记短时互斥
    ├── associations.json          MpvKit 登记及原值
    ├── association-pending.json   登记事务中断恢复
    ├── pending.json                  当前文件事务；完成标记用于中断后清理重试
    ├── recovery/ / staging/           当前操作副本和暂存，成功或恢复后自动删除
    ├── cache/sessions/            受管理启动的有界日志
    └── launcher/                  稳定 helper 及应用许可
~~~

不强制创建空目录。thumbfast 临时图像按上游实现存于系统临时目录。Debug 隔离根需显式设置 MPVKIT_TEST_MODE=1 与合法绝对 MPVKIT_TEST_ROOT，登记使用私有文件后端；Release 忽略测试覆写。测试记录不属于正式数据结构。

Linux 应用偏好仍在 `$XDG_DATA_HOME/mpvkit/mpvkit.json`（回退 `~/.local/share/mpvkit`）。增强与配置根由 `platform/linux.rs` 检测：非空绝对 MPV_HOME 优先；否则使用绝对 XDG_CONFIG_HOME/mpv 或 ~/.config/mpv；标准目录不存在时，按 MPV 原生规则识别现有 ~/.mpv。检测只检查已知用户位置，不扫描磁盘。相对路径/符号链接阻断；标准目录与 ~/.mpv 同时存在实际配置时报告歧义，不合并。

检测根中直接放置 mpv.conf、input.conf、scripts/、script-opts/、fonts/、licenses/；.mpvkit/ 内保留本环境的归属、设置、摘要、锁、当前失败恢复与 launcher/mpvkit，继续使用单目录事务。普通系统 MPV 默认读取，稳定 helper 也可显式启动同一目录；Linux 启动不再重设 MPV_HOME 为日志目录。Lua 在两端均直接根据原生配置根查找保存记录/helper。不部署 DLL、Windows helper 或 fonts.conf。

Linux 首屏显示检测路径和来源，可打开目录；两个路径相同，不重复显示安装目录行。Linux 默认不额外登记打开方式，普通 MPV 已可读配置；用户仍可在高级项中启用可选登记。不迁移此前开发版数据，不清理此前实际目录。隔离测试根使用 MPVKIT_TEST_ROOT/config/mpv，真实配置不参与测试。完整实现与证据见 [v0.1.7](v0.1.7/plan.md)。

两端系统 MPV 安装状态与实际版本均独立于增强归属记录，首次和卸载后仍显示。Windows 安装路径只读固定包的精确卸载登记与已知 Program Files 位置；多处核心阻断，不扫盘。MPV_HOME 必须为绝对本地路径；没有 MPV_HOME/portable_config 时，安装目录直接配置或其 mpv/ 子目录的实际配置与用户目录并存会阻断，避免混合加载（包附 fonts.conf 例外）。系统核心不进入文件归属/失败恢复，但其文件状态作为配置和启动预览的过期保护。Debug 隔离核心位于 MPVKIT_TEST_ROOT/system-mpv，WinGet 只能来自 test-tools/winget.exe；不得回退调用本机包管理器。

Linux 系统语言优先 KDE 显式配置，明暗优先 portal Settings，GTK 回退；前端和原生背景同源，保留手动设置。Linux 系统强调色暂固定蓝色。

## 模块与保护

commands 只派生固定根、存放预览并调用业务；setup 汇总状态和组织具体文件/登记顺序；config 生成和保护配置；components 获取标准资产；deploy 负责归属、摘要、锁、当前事务恢复和自动清理。integration 对外提供操作入口，Windows 注册数据与 Linux desktop entry 后端复用 platform/integration.rs 的登记事务与恢复。Linux 仅登记用户级打开方式，不接管默认应用；测试使用隔离 applications 目录。无通用工作流框架。

全局偏好不再依赖目标选择锁，直接使用全局目录的部署互斥。受管理 helper 校验固定根；原程序移动或 GUI 关闭不影响稳定入口。文件/注册表分别有事务，不宣称跨资源原子性；所有相关状态在执行前校验。无文件变更时使用实际保留的安装记录摘要，登记预览同时保护已有且无需改写的注册值。

两平台共用预设新增 i 短暂显示播放信息、Shift+i 持续显示/关闭；资源由同一 input.conf 生成。反引号 ` 绑定原生 commands/open，右键菜单加入“控制台”；Esc 关闭，输入时控制台接管空格/方向键/Enter。现有文件保留手改保护，不自动追加。

视频背景单击忽略，双击切换全屏；播放/暂停使用中央按钮或空格，避免双击全屏时触发暂停。此规则为两平台共用 input.conf，uosc 控件继续使用各自鼠标绑定。

新配置默认 autofit=75%x75%、auto-window-resize=no：启动时在屏幕宽高 75% 范围内按视频比例适配，切集保留当前窗口尺寸；手动缩放与全屏可用。由共用 Rust 生成器写入 mpv.conf 的可手改默认区，GUI 保存不覆盖用户修改；不新增尺寸设置页、持久窗口记忆或 Lua 窗口调度。Linux Wayland 的位置由桌面管理器决定，不强制跨平台居中。

## 验证边界

本版证据在 [v0.1.8 记录](v0.1.8/evidence.md) 更新；历史结果不等于本版验收。Windows 已覆盖本版共享播放逻辑，Linux 原生构建与 Wayland 实播待测。实体高刷/HDR/功耗、不同系统策略、干净系统及完整来源仍有缺口，参见[验收台账](v0.1.0/acceptance.md)。[来源政策](v0.1.0/sources.md)、第三方许可证和历史证据继续保留。不得执行历史安装器测试。

## 组件管理与自动清理（2026-10-26）

已按[本轮维护方案](v0.1.8/maintenance.md)实施。重置恢复结构化默认设置与四个完整预设配置；普通保存保留管理区外和其他可编辑文件。增强卸载删未修改的自有预设、脚本、字体、许可证、helper、设置/安装记录和自身登记，保留手改/未知文件、续播及应用外观；不再恢复安装前文件或提供撤销。

下载与解压保持有界内存，不生成磁盘下载缓存。成功或失败恢复后立即按记录和摘要清理本事务 staging/recovery；中断的已完成清理在下次状态检测重试。不改动未完成操作的恢复内容。增强卸载纳入已退出自有日志清理，活跃/被改/归属未知日志保留，目录只非递归移除空祖先。owner.json 与 operation.lock 是安全归属/互斥所需的最小记录，不能在锁仍被持有时删除；根目录及未知内容不递归清理。系统包管理器自己的缓存由其管理，不擦除共享 WinGet/pacman 缓存。
