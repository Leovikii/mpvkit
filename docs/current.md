# 当前开发基线：v0.2.0-beta.1

更新：2026-10-31。当前为 beta.1 开发测试版，用户已授权实现、测试、构建及提交/推送 dev；本轮暂不合并，交接待 Windows 测试，不发布。公开版仍为 alpha.5，历史结果见 [alpha.5 交接](v0.2.0-alpha.5/plan.md)；本版结果与限制见 [beta.1 交接](v0.2.0-beta.1/plan.md)。构建入口见 [测试文档](testing.md)。

GUI 使用官方单实例插件，同一用户桌面会话重复启动唤回现有窗口；`--save-playback` helper 在 GUI 初始化前独立处理，不受单实例拦截。Linux 依赖有效会话总线，缺失时明确启动失败；desktop 仅声明 AudioVideo，升级软件包后生效。单实例不限制 MPV、WebKit 子进程或其他用户会话。

## 当前界面与操作

- 默认播放配置页，右上角进入设置；设置页左上返回，无侧栏。初始 560×760，最小 480×600，保留 fade-up 切页与现有 Modal/Accordion/Tooltip 动效及减少动态效果。配置草稿切页、失败后保留，关闭时提示放弃。
- 配置和增强统一安装：预览只读，核心缺失时一次应用确认，先运行原生包管理器，再复核核心并部署配置。健康组件跳过。配置页可在安装前编辑；保存配置不重装健康组件。
- 底部未安装为一键安装，已安装为一键卸载；草稿或管理区手改时显示保存，缺损可统一修复。画质、HDR、自动连播及结束行为保留，路径/组件版本/来源收入高级项。
- 设置包括语言/主题、紧凑 MPV 管理菜单、MpvKit 更新、打开日志文件夹及 GitHub 图标 + Leovikii 仓库按钮。无页面刷新、独立关于、版权许可证行或手动诊断导出；许可材料仍随包分发。
- 两页设置行共用布局：说明/状态在左、控件在右，换行后仍右对齐；主页主操作位于动作组最右，页内及设置页工具操作成组靠右。
- 页面铺满客户区，删除外层留白、描边和圆角底板；设置分组保留轻边框，底部操作区共用页面背景。
- Windows 使用 DWM Mica；Linux 使用 KWin 系统装饰，标题栏、按钮、边框和阴影遵循用户 KDE 设置；内容区保留模糊，高对比或无能力时实色。语言和主题默认跟随系统，测试入口不强制中文/浅色。完整规范见 [界面规范](design/ui-guidelines.md)。

## 核心与维护

Windows 固定 WinGet `shinchiro.mpv` / winget 源，Linux 固定 `/usr/bin/mpv` / Pacman `mpv`。安装/更新为 `-S --needed -- mpv`，卸载为 `-R -- mpv`。在原生终端中等待实际命令退出，不代答询问，不强制接受协议、递归卸载依赖、刷新数据库或整机升级；不从包管理器输出猜进度。

薄层 `--core-command` helper 在 GUI/单实例之前执行，仅接受固定动作及当前私有任务请求。Linux 用 Konsole 和 pkexec，Windows 使用真实控制台输入；helper 返回包管理器实际退出码和最多 16 KiB 输出尾部，同窗可查看，不持久记录原始输出。终端自身退出码不代表包管理器结果。命令结果与检测状态分别显示；独立核心卸载保留配置。

用户确认的一键卸载删除整个已展示的 MPV 配置根，包括手改、其他脚本及续播；“同时卸载 MPV”默认关闭。核心卸载失败或取消仍继续配置删除，分别报告，可仅重试失败阶段。危险根、链接/重解析点、过期预览、根身份和内容变化继续阻断或报告；仅删除已审核内容，新出现内容不扫除。开发测试不处理宿主真实目录。

正常安装/保存保留归属、摘要、互斥、手改保护及当前事务失败恢复。无历史备份/撤销/回退/手动缓存清理。稳定锁位于配置根的同级、按绝对路径摘要命名，完整卸载不会删除持有中的锁；安装、删除与播放期保存使用同一把锁。

日志放在应用数据根 `logs/`，与 MPV 配置分离。只记录白名单任务字段/错误代码，不保存媒体路径、原始配置、密码或终端全文。每条上限 8 KiB、单文件 1 MiB、当前加三份轮转共 4 MiB，启动/写入时清理超过 7 天的固定命名日志；不扫描其他文件。工作线程写入，检测问题去重，日志失败不阻止业务。无上传、导出路径输入或后台清理服务。

## 播放器默认与交互

默认均衡，HDR Auto 交由 MPV；三档原生 profiles、位置续播、音量 ±10、文楷字体保留。无播放硬件检测、显示器/分辨率设置、供电推荐、强制 HDR 硬件预校验或语种排序 GUI。alang/slang 默认汉语→日语→英语→西班牙语，位于可手改区。截图设置/菜单/Lua/参数均已删除，thumbfast 缩略图保留。

自动连播默认开启：`autocreate-playlist=filter`、`directory-filter-types=video,audio`、`directory-mode=ignore`，同目录音视频自然排序，从所选文件继续，不扫子目录；关闭不干预显式列表。`keep-open=yes/no` 仅决定整个队列结束后停留/退出。字幕 `sub-auto=exact` 支持原生语种/forced/SDH 等后缀，逐项追加 sub/subs/subtitle/subtitles；Linux 补首字母及全大写，Windows 避免重复扫描。默认区手改在 GUI 保存后保留。

uosc 左侧仅上下集/播放暂停（44/48/44 逻辑像素），右侧轨道、倍速/模式/列表、信息/更多/全屏；透明次要按钮、55% 统一衬底与短渐隐，窄窗收起项仍从更多访问。倍速为原生六项列表，外部改速同步；四模式菜单统一设置随机/列表循环/单集循环，外部非标准组合显示自定义，只在当前进程生效。固定 uosc main.lua 上游字节保留为前缀，末尾追加已声明状态/宽度桥接，不重写导航，双端来源与摘要随部署保存。

视频背景单击忽略、双击全屏；播放暂停用左侧按钮或空格。i 短暂显示信息、Shift+i/信息按钮切换原生 stats；反引号/右键打开原生 commands/open，Esc 关闭，控制台接管输入。

`autofit=75%x75%`、`auto-window-resize=no` 位于可手改默认区；保持比例，切集保留窗口，不持久化窗口或强制居中。Linux 首个视频实际唯一落屏后仅重设当前 autofit 一次并注销观察；显式 geometry/window-scale、全屏/最大化/最小化、已观察到的用户尺寸变化或跨多输出时跳过。4K 小窗已本机修复；混合 DPI、平铺/极短拖动竞态仍未验收。

## 当前目录

Windows 使用用户级 NSIS：启动即显示进度并自动安装到 `%LOCALAPPDATA%\Programs\MpvKit`，成功自动关闭，无确认/目录选择向导，非静默；异常和系统提示保留。见[安装器来源与边界](../packaging/windows/README.md)。Arch 原生包安装到 /usr/bin/mpvkit。默认开发仍可直接运行单程序。两端配置均直接包含 mpv.conf、input.conf、script-opts/、scripts/、fonts/、licenses/；没有 user.conf。

| 数据 | Windows | Linux |
| --- | --- | --- |
| 系统核心 | WinGet 精确登记及已知 Program Files 位置，不归 MpvKit 管理文件所有权 | /usr/bin/mpv，不复制系统核心 |
| 应用偏好 | %LOCALAPPDATA%/io.github.mpvkit/mpvkit.json | 同 Windows 职责，$XDG_DATA_HOME/io.github.mpvkit/mpvkit.json，回退 ~/.local/share/io.github.mpvkit |
| MPV 配置根 | MPV_HOME → 已存在的相邻 portable_config → %APPDATA%/mpv | MPV_HOME → XDG_CONFIG_HOME/mpv 或 ~/.config/mpv；标准目录不存在时识别原生 ~/.mpv |
| 播放期保存 | scripts/mpvkit.lua 调用已部署时的管理器绝对路径 | 同左，Arch 固定程序为 /usr/bin/mpvkit |

只检查已知本地位置。相对路径、符号链接、Windows 重解析点/多处核心或多个实际配置目录歧义均阻断，不合并、不迁移。Windows 无 MPV_HOME/portable_config 时，相邻实际配置与用户目录并存也阻断（包自带 fonts.conf 例外）。Linux 不部署 DLL、Windows helper 或 fonts.conf；普通系统 MPV 启动直接读取原生配置，不重设 MPV_HOME 到日志目录。

配置根 `.mpvkit/` 保存 settings.json、owner.json/install.json、当前 pending.json、staging/recovery；不生成 cache/sessions、launcher 或管理器许可副本；watch_later 为 MPV 按需生成的续播数据。不强制创建空目录。

媒体打开方式由系统 MPV 包维护；不再生成或管理 MpvKit 媒体登记、意图字段或登记恢复记录。Windows 包在安装后按系统 audio/video 分类登记扩展名，Linux 包自带 mpv.desktop。默认应用由用户在系统设置选择；“设置默认应用”收入设置页 MPV 管理菜单，核心未安装时禁用，不参与草稿/保存。见 [来源核验与限制](v0.1.9/evidence.md#windows-软件包的媒体登记核验)。

Linux 管理器入口由 Arch 包维护 packaging/io.github.mpvkit.desktop 与系统图标；Exec=mpvkit，无媒体 MIME、--open 或用户 applications 链接。GTK app_id 为 io.github.mpvkit。移除增强不会删除系统包入口。

设置 → 应用更新：GUI 就绪 3 秒后检查一次，提前手动检查则合并，无周期检查。接收预览版默认 false；切换废弃旧结果，迟到响应无效。固定 GitHub Releases API 按 SemVer 发现完整版本，排除草稿及不符合预览偏好的版本；每版 latest.json 包含双平台标准项，不再有渠道索引或 AUR。API 分页/超时/限流错误不能当成无更新。

更新对话框显示纯文本日志、版本和日期，下载、校验、预览、安装及结果保留在同一个 Modal。已知总量用独立 Mantine Progress 和真实百分比，未知阶段用 Loader 与短状态；按钮保持正常内边距，不承载进度。两端先下载验签，呈现具体预览后只确认一次安装；取消已准备的更新释放候选及载荷。两端复用官方 updater 下载/验签、GitHub digest 与 SHA256SUMS 核对。Windows 确认后将已验签内存载荷交官方 NSIS 安装；Arch 先下载到私有临时目录，核对包名/版本/架构及 Pacman 事务，再由用户确认并通过 pkexec 执行固定 Pacman 参数。安装前核对候选/包/事务状态，持有应用及配置锁；草稿或当前事务阻断安装。无 shell、签名降级、数据库刷新、常驻提权服务或 Pacman 强杀。Linux 仅支持 /usr/bin/mpvkit 且归属 mpvkit-bin 的系统安装，开发/解压程序明确拒绝应用内系统升级。

Arch 成功后核对版本/归属，释放锁后请求自动重启；Windows 显式启用官方 updater 的安装后重启。双端真实跨版本重启仍待对应原生环境验收。授权取消/失败就地反馈，重新检查后重试。更新区遵守 WinUI 3 设置行，对话框操作区固定，未知进度不伪造百分比。Arch Minisign 文件使用 .tauri.sig，避免被 Pacman 当作 OpenPGP 的 .sig；继承系统 LocalFileSigLevel。首次安装仍从 GitHub 下载后 pacman -U 本地包，不通过 AUR。

Debug 只有显式 MPVKIT_TEST_MODE=1 + 合法绝对 MPVKIT_TEST_ROOT 才覆写根；配置位于其 config/mpv；更新元数据只读其 test-tools/updates，禁止测试安装器。Release 忽略测试覆写。Windows 隔离核心/WinGet 仅来自 system-mpv/test-tools；不得回退调用本机包管理器。

## 源码导航

| 修改对象 | 首先阅读 |
| --- | --- |
| UI/动作/双语/品牌 | src/App.tsx、types.ts、i18n.ts、brand.ts；当前演示仍在 docs/v0.1.6/preview |
| 命令与操作编排 | src-tauri/src/commands.rs（薄层）、tasks.rs、setup.rs、removal.rs |
| 配置与播放偏好 | config.rs、config/portable.rs、preferences.rs；resources/ 中共享预设/Lua |
| 组件与安全文件事务 | components.rs、deploy.rs；下载/解压有界、摘要/锁/过期保护/当前恢复 |
| 平台路径/核心/桌面 | platform/ 下 windows_mpv/linux_core、windows/linux、desktop/linux_desktop、appearance/linux_appearance |
| 系统默认应用快捷入口 / 管理器桌面 | platform/windows.rs、linux.rs 只打开系统设置；系统包入口在 packaging/ |
| 更新/日志/核心进程 | updates.rs、app-updates.tsx、logging.rs、native_core.rs、mpv.rs |

业务边界和安全规则以 AGENTS 为准：前端不合并文件；文件变更使用现有事务；全局偏好使用全局根的锁。本版验证见 [交接](v0.2.0-beta.1/plan.md)，来源政策、第三方许可证与旧批次原始证据继续保留。
