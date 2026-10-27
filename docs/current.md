# 当前开发基线：v0.2.0-alpha.2

更新：2026-10-27。当前为第二个 alpha 预览版，0.2.0 稳定版尚未发布。当前修复、验证和发布状态见 [alpha.2 交接](v0.2.0-alpha.2/plan.md)；[alpha.1](v0.2.0-alpha.1/plan.md) 与其证据仅作历史基线。构建和发布流程见 [CI/CD](ci/README.md)，按影响面选择 [必要测试](testing.md)。

GUI 使用官方单实例插件，同一用户桌面会话重复启动唤回现有窗口；`--save-playback` helper 在 GUI 初始化前独立处理，不受单实例拦截。Linux 依赖有效会话总线，缺失时明确启动失败；desktop 仅声明 AudioVideo，升级软件包后生效。单实例不限制 MPV、WebKit 子进程或其他用户会话。

## 当前界面与操作

- 两页：**播放器设置 / 应用设置**。首次即可预编辑，草稿切页或安装失败保留，关闭提示放弃；组件安装与保存配置解耦。保存前要求必需组件健康，并校验摘要/fingerprint，手改覆盖须具体差异预览。
- 顶部组件区集中核心/增强/配置及明细。核心缺失才安装，已有先检查更新，有更新才显示更新，无更新明确提示；核心能力不满足不等于未安装。增强按缺失、损坏、内置版本变化分别安装/修复/更新，健康当前版本不重复安装，核心缺失不改变健康增强状态。
- 主区依次为画质、HDR、自动连播、全部播放结束后；底部固定预览和保存动作，未变且健康时禁用保存。显示 mpv.conf 实际 mtime，文件未变不重写。低频路径/来源放在高级项；默认应用按钮位于 MPV 核心行，目录行左侧标题/实际路径，右侧“打开”。Linux 两个根相同时不重复显示。
- 应用设置：简中/英文、系统/浅色/深色、关于/许可/诊断。固定图标紫色 **#A84BD6**（src/brand.ts）；无主题色字段/选择或系统强调色读取。Windows 原生材质按系统策略回退；Linux 纯色。前端遵守 [界面规范](design/ui-guidelines.md)。

## 核心与维护

Windows 用 WinGet 固定 `shinchiro.mpv` / winget 源；缺少 WinGet 给出 App Installer 入口。Linux 使用系统 `/usr/bin/mpv`，固定 pacman 包 mpv，经 pkexec 原生授权。操作先预览再执行并复核实际版本/能力；不自下载每日核心或后台更新。Linux 查询 `pacman -Qu mpv`，安装/更新使用 `-S --needed mpv`，卸载 `-R mpv`，不单独刷新数据库或递归卸载依赖。查询错误不能当无更新。WinGet 包标签不能证明内部精确稳定提交。

核心安装状态独立于增强清单，核心卸载保留配置，增强移除保留系统核心。重置恢复默认结构化设置及四个完整预设文件；普通保存只改 GUI 管理区域，保留默认区手改。移除增强按归属/摘要删未改的自有文件、设置/安装记录，保留未知/手改、续播和应用偏好。

无历史备份、撤销、回退、手动缓存清理。当前操作 staging/recovery 只供失败恢复，成功或恢复后自动清理；完成清理中断可重试。卸载仅清理未改动的自有文件、未用记录与空目录；owner.json/operation.lock 保留，不能删除正在持有的锁。不递归删除根或清空 WinGet/pacman 共享缓存。完整决定见 [maintenance](v0.1.8/maintenance.md)。

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

配置根 `.mpvkit/` 保存 settings.json、owner.json/install.json、operation.lock、当前 pending.json、staging/recovery；不生成 cache/sessions、launcher 或管理器许可副本；watch_later 为 MPV 按需生成的续播数据。不强制创建空目录。

媒体打开方式由系统 MPV 包维护；不再生成或管理 MpvKit 媒体登记、意图字段或登记恢复记录。Windows 包在安装后按系统 audio/video 分类登记扩展名，Linux 包自带 mpv.desktop。默认应用由用户在系统设置选择；“设置默认应用”按钮放在顶部 MPV 核心操作行，核心未安装时禁用，不参与草稿/保存。见 [来源核验与限制](v0.1.9/evidence.md#windows-软件包的媒体登记核验)。

Linux 管理器入口由 Arch 包维护 packaging/io.github.mpvkit.desktop 与系统图标；Exec=mpvkit，无媒体 MIME、--open 或用户 applications 链接。GTK app_id 为 io.github.mpvkit。移除增强不会删除系统包入口。

应用设置 → 关于：正常 GUI 界面就绪 3 秒后检查一次；提前手动检查则合并，无周期检查或自动检查开关。接收预览版默认 false；切换废弃旧结果，保存后可立即重查，迟到响应无效。结果/进度只在内存。弹窗显示纯文本日志、版本和日期；Windows 签名下载/安装，草稿未保存时禁止；Linux 提示用户通过 yay/paru 等 AUR 助手手动更新本程序后重启，仅链接已发布配方，不执行 helper 或写 /usr/bin。预览开关只控制应用内提醒，不限制包管理器的更新渠道。关于区采用 WinUI 3 设置行，更新弹窗操作区固定。公开渠道已提供 alpha.2 预览；接收预览后同版检查返回最新，关闭预览且尚无稳定版时正常提示无稳定版。

Debug 只有显式 MPVKIT_TEST_MODE=1 + 合法绝对 MPVKIT_TEST_ROOT 才覆写根；配置位于其 config/mpv；更新元数据只读其 test-tools/updates，禁止测试安装器。Release 忽略测试覆写。Windows 隔离核心/WinGet 仅来自 system-mpv/test-tools；不得回退调用本机包管理器。

## 源码导航

| 修改对象 | 首先阅读 |
| --- | --- |
| UI/动作/双语/品牌 | src/App.tsx、component-actions.ts、types.ts、i18n.ts、brand.ts；当前演示仍在 docs/v0.1.6/preview |
| 命令与操作编排 | src-tauri/src/commands.rs（薄层）、setup.rs |
| 配置与播放偏好 | config.rs、config/portable.rs、preferences.rs；resources/ 中共享预设/Lua |
| 组件与安全文件事务 | components.rs、deploy.rs；下载/解压有界、摘要/锁/过期保护/当前恢复 |
| 平台路径/核心/桌面 | platform/ 下 windows_mpv/linux_core、windows/linux、desktop/linux_desktop、appearance/linux_appearance |
| 系统默认应用快捷入口 / 管理器桌面 | platform/windows.rs、linux.rs 只打开系统设置；系统包入口在 packaging/ |
| 更新/诊断/核心进程 | updates.rs、app-updates.tsx、mpv.rs、diagnostics.rs |

业务边界和安全规则以 AGENTS 为准：前端不合并文件；文件变更使用现有事务；全局偏好使用全局根的锁。本版验证见 [交接](v0.2.0-alpha.2/plan.md)，来源政策、第三方许可证与旧批次原始证据继续保留。
