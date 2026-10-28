# v0.2.0-alpha.3 实施与交接

更新：2026-10-28。用户已授权完成 alpha.3、双平台可行测试、更新 Actions、推送 dev、PR 合并 main 并同步 dev。当前源码统一为 **0.2.0-alpha.3**；以下为本版设计与实施边界，实际验证及发布状态见末尾交接。

## 目标与取舍

以个人项目的最小维护成本完成两端分发和应用更新：GitHub Releases 保存版本与资产，Tauri 官方 updater 处理下载及验签，Windows 继续安装 NSIS，Arch 通过 Pacman 安装原生包。不维护自己的渠道索引、更新服务器或 AUR 接入。

- 保留“接收预览版”（默认关闭）、启动就绪后检查一次及手动检查；无正式版时关闭预览应显示“暂无稳定版”，不能把 alpha 标成稳定版。
- 删除 `updates/channels.json`、渠道推广及其专属测试；不保留旧 alpha 协议、兼容读取、迁移或双写。旧客户端不保证继续应用内更新，用户可重新安装新包。
- 只有一个应用版本、一个 Arch 包和两种安装方式。保留现有 `mpvkit-bin` 包名以减少无关改动；`-bin` 只表示二进制包，不表示依赖 AUR，不另设 preview 包。
- 不新增发行版、AppImage、DEB/RPM、MSI、自建 Pacman 仓库、常驻提权服务、远程安装脚本或后台自动安装。
- 不为精简删除签名、版本核对、路径边界、未保存草稿保护和包管理器错误处理；不顺带重构播放器、配置事务或其他界面。

## 依据与参考方案的修正

| 已核对的依据 | 对本版设计的影响 |
| --- | --- |
| [Tauri updater 官方文档](https://v2.tauri.app/plugin/updater/) | 使用标准静态 manifest 和官方签名校验，不设计第二套更新协议。 |
| [GitHub Releases API](https://docs.github.com/en/rest/releases/releases) | Latest 不包含 Pre-release；当前全是 alpha，不能只用 `/releases/latest/download/latest.json` 解决预览更新。公开列表可匿名读取。 |
| [Pacman 手册](https://man.archlinux.org/man/pacman.8.en) | `pacman -U` 支持文件或 URL，并从当前同步数据库解析所需依赖；不需要 AUR 或给最终用户运行 PKGBUILD。 |
| [Pacman 配置手册](https://man.archlinux.org/man/pacman.conf.5.en) | URL 安装遵循 `RemoteFileSigLevel`，本地文件遵循 `LocalFileSigLevel`；Minisign/Tauri 签名不等于 Pacman 的 OpenPGP 签名。 |
| [pkexec 官方手册](https://www.freedesktop.org/software/polkit/docs/latest/pkexec.1.html) | 复用桌面授权代理，分别处理拒绝/取消与 Pacman 失败；不保存密码、不自制提权机制。 |
| [已固定 updater 2.11.0 源码](https://github.com/tauri-apps/plugins-workspace/blob/updater-v2.11.0/plugins/updater/src/updater.rs) | `check()` 和 `download()` 可复用；`download()` 验签后返回字节。该版本安装分支没有 Pacman，Arch 不能直接调用其 `install()`。本地锁定源码也已核对，尚未作 Linux 运行验证。 |

参考命令的方向可行，但不能直接承诺现有包可以这样安装：

```sh
sudo pacman -U https://github.com/Leovikii/mpvkit/releases/latest/download/mpvkit-x86_64.pkg.tar.zst
```

原因是当前没有稳定版，现有资产带版本号，且没有供 Pacman 信任的 OpenPGP 包签名。为这一行命令再增加固定文件别名、PGP 密钥导入和信任配置，不符合本版最小成本目标。

本版采用**普通用户下载、校验，再 `pacman -U` 本地包**。下载与安装可以用 `&&` 连成一行；不要求使用 Pacman 的远程 URL 模式，不修改系统签名策略。依赖解析仍由 Pacman 完成，系统同步数据库过期或依赖冲突时明确失败，提示用户自行完成系统更新后重试；程序不执行 `pacman -Sy` 或擅自进行全系统升级。

## 发布和更新结构

```text
GitHub Releases API → 筛选版本 → 此版本的 latest.json
                                  ├─ windows-x86_64 → NSIS + Tauri 验签/安装
                                  └─ linux-x86_64   → pkg.tar.zst + Tauri 验签 → pkexec pacman -U
```

### 版本发现

共用一条版本发现逻辑：读取固定仓库的公开 Releases 列表，排除草稿、非版本 tag、尚未完成资产上传的 Release；默认排除 Pre-release，开启预览后同时考虑稳定版和预览版，按现有 SemVer 比较选择最高版本，不按列表第一项或发布时间猜版本。

- 使用 GitHub 标准分页、请求超时及响应大小限制；不在客户端放 Token。遇到限流、网络错误或达到读取上限时明确失败，不能伪装成“已经最新”或“暂无稳定版”。
- 无稳定版属于正常结果；稳定版高于同系列 rc 时选择稳定版；关闭预览不会降级当前程序。
- 更新候选仍保存在内存；复用现有过期时间、预览偏好变化失效和迟到响应保护，不新增本地版本数据库或持久更新队列。
- 先核对 tag、manifest 版本、资产地址和摘要归属同一个版本、同一个固定仓库，再交给 updater。网页日志继续按纯文本显示。

### 每版资产与完成标记

每个版本只发布以下六项文件：

| 资产 | 用途 |
| --- | --- |
| `MpvKit_<version>_x64-setup.exe` 及其 Tauri `.sig` | Windows 安装与应用内更新 |
| `mpvkit-bin-<pkgver>-1-x86_64.pkg.tar.zst` 及其 Tauri `.tauri.sig` | Arch 首次安装与应用内更新；pkgver 沿用现有 alpha/beta/rc 转换 |
| `latest.json` | Tauri 标准 manifest：version、notes、pub_date、两个 platforms 项的 url/signature |
| `SHA256SUMS` | 所有发布文件的 SHA-256，保留独立核验入口 |

将当前 `updater.json` 直接改为标准示例名 `latest.json`，不兼容旧文件名。Linux 增加正式签名并复用已有签名密钥、公钥和工具；签名文件沿用 Tauri 格式；Arch 使用 .tauri.sig，避免 Pacman 将相邻 .sig 当作 OpenPGP 包签名。**不把它作为 Pacman OpenPGP 签名提供**。不添加自己的 manifest 字段或第二份渠道文件；资产摘要使用已有 `SHA256SUMS` 和 GitHub 资产 digest，缺失/不一致即拒绝安装。

继续从创建起 `draft=false`、alpha/beta/rc 为 `prerelease=true / latest=false`，不移动版本 tag。上传顺序调整为安装包、签名及其他必要资产 → `SHA256SUMS` → **最后上传 `latest.json`**；版本发现只有在 manifest 和双平台必要资产均齐备时才选择它，避免用户撞上上传中的公开 Release。已存在但损坏的 manifest 按错误处理，不能静默降级成“无更新”。

保留 main 检测版本/tag、已存在 tag 跳过、失败复用完整构建产物与只补缺项的能力；删除渠道串行推广和 AUR 参数，仍需保留同一版本发布互斥。人工发布批准门槛不增加，缺失许可材料、签名、摘要和资产契约检查继续执行。

## 两端安装与应用内更新

### Windows

维持 x64 用户级 NSIS 和 Tauri 官方 updater。由共同版本发现逻辑确定 manifest URL，Tauri 执行 check/download/install；保留公钥、下载进度、草稿/当前操作阻断和安装前复核。不得自行写第二套 EXE 更新器。真实 NSIS 行为仍需隔离 Windows 用户/VM 验收，不能用下载成功代替安装成功。

### Arch 首次安装

README 保持英文，提供当前实际版本的 Release 下载链接和 `sudo pacman -U ./<包名>.pkg.tar.zst`，以及对应的一行下载/校验/安装命令。不宣传 AUR、yay/paru 或普通 `pacman -Syu` 能更新本程序。

- 一行命令只组合 `curl`、发布哈希校验和 `sudo pacman -U`，不下载/执行 shell 安装脚本，保留 Pacman 原生确认。
- 安装命令使用具体 tag 和真实文件名，不增加无版本包副本；最终命令及实际 SHA-256 在构建完成后填入 Release 说明，不能预写伪造摘要。README 给稳定的说明入口及命令格式，alpha 使用具体版本链接。
- 继承系统 `LocalFileSigLevel`，不改 `/etc/pacman.conf`、不导入私钥、不设置 `SigLevel=Never`。用户自定义为必须 OpenPGP 签名时，本版明确提示限制，不能绕过。
- `webkit2gtk-4.1`、GTK 等依赖由 Pacman 的系统仓库解析；PKGBUILD 负责准确声明依赖，不为它们另写安装代码。桌面入口、图标及 `/usr/bin/mpvkit` 继续由包管理。

### Arch 应用内更新

在现有更新弹窗内完成“下载准备 → 安装预览 → 授权安装 → 完成”，不增加设置页或终端窗口。下载/准备由用户触发；检查更新不会后台安装或提权。

1. 选择新版本后，复用同版本 `tauri-plugin-updater =2.11.0` 的 check/download。Linux 扩大该既有依赖的目标范围、注入相同公钥；只调用下载/验签，**不调用插件 Linux install**。先做一个原生包下载验签的最小验证，确认可用后再接安装流程。
2. 将验签后的包写入本事务独占临时目录，使用成熟的 `tempfile`（锁文件已有间接依赖；已显式声明 Linux 目标固定版本，更新两平台许可清单），目录仅当前用户访问，文件不接受外部指定路径、符号链接或复用固定 `/tmp/xxx`。使用现有 sha2 复核发布摘要和落盘内容，不手写验签或临时目录算法。
3. 通过 Pacman 只读查询核对包名、版本、架构和将要执行的事务，在同一弹窗列出目标版本及新增/升级依赖。**“一键安装”指在此具体预览上点击安装，再完成系统授权**；不以 `--noconfirm` 代替预览。若事务要求移除其他包、解决替代/冲突或数据库过期，则停止并提示系统处理，不自动作答或扩大操作范围。
4. 安装前重新检查候选未过期、预览偏好与本地包状态未变、草稿已保存、无当前配置事务；重新查询事务，变化则要求刷新预览。复用现有应用/配置根锁覆盖安装阶段，阻止 helper 或其他维护写入。确认当前运行文件由已安装的 `mpvkit-bin` 包管理，且是 `/usr/bin/mpvkit`。手动运行的开发/解压程序明确提示只支持系统包安装，不提供另一个下载路径或悄悄切换成系统安装。
5. GUI 以普通用户运行，仅用固定可执行文件和分离参数调用：

   ```text
   /usr/bin/pkexec --disable-internal-agent /usr/bin/pacman -U --noconfirm -- <已核验的绝对包路径>
   ```

   复用平台模块的命令启动与错误处理。实际代码不拼 shell；不关闭依赖、架构、文件冲突或系统签名检查，不自制 Polkit 免密规则。授权取消、未授权、缺代理和 Pacman 失败分别反馈，命令输出临时落盘以避免管道阻塞；不持久保存原始包管理器日志，失败提示检查系统包状态。
6. Pacman 事务开始后禁用重复操作/退出，不使用下载超时去强杀正在提交的包管理器，不删除 `db.lck` 或尝试自行回滚系统包。进度只对包下载显示真实百分比，授权/安装阶段显示状态，不伪造总进度。
7. 退出码成功后再查询已安装包版本并核对程序归属；显示“更新完成，请重新启动”，由用户退出并重新打开，沿用现有单实例。当前进程版本不伪装成新版本，不增加后台重启 helper。仅在包管理器结束后清理本事务临时文件；不清理系统包缓存或旧事务残留目录。

临时目录及执行前复核防止其他用户替换文件和误用过期下载，不宣称防御已完全控制同一用户会话的恶意进程。已有系统强制 OpenPGP 校验时尊重拒绝结果；若以后确有该部署需求，再独立考虑 Pacman 签名发行，不能在本版偷偷降级系统策略。

应用的只读预览不是持有 Pacman 数据库锁的系统事务，Polkit 等待期间系统状态仍可能变化；最终依赖/冲突校验由实际 Pacman 事务完成。不为消除这段间隔引入 libalpm 前端或特权服务，也不宣称 GUI 预览具有跨进程原子性。

## 前端设计：WinUI 3 / Fluent

前端遵循 [WinUI 3 / Mantine 界面规范](../design/ui-guidelines.md)，以现有 Mantine 组件实现微软 Windows 应用设计原则。保留播放器设置、应用设置两页；应用更新仍位于“应用设置 → 关于”，不增加更新中心、下载管理器或第三方 UI 依赖。Windows 与 Linux 共用布局和进度展示，仅安装动作及必要说明按平台区分。

### 布局与操作

采用 WinUI 设置行：名称与状态在左，检查/更新按钮在右；窄窗换行。更新对话框按“目标版本及日期 → 更新日志 → 当前状态/Arch 安装预览 → 操作区”排列。标题与正文左对齐，使用既有系统字体、留白和中性表面；主动作使用品牌紫色 #A84BD6，控件圆角 4、弹窗圆角 8。上述尺寸为项目基线，不能称为微软强制指标。

复用 Mantine Modal 的标题、焦点锁和回焦行为，正文独立滚动，底部操作区固定可达；执行按钮在左，“稍后”在右。一个阶段只有一个主动作，在同一个对话框内切换状态，不叠加确认框。Arch 具体事务预览后才显示“安装”；Windows 显示“下载并安装”。运行中禁用重复操作及关闭入口，明确显示当前阶段；系统授权仍由原生 Polkit 窗口处理。

### 两端进度反馈

依据微软 [Progress controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls) 与 [Dialog controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs)，采用以下状态。按钮内填充是本项目的辅助设计，并非 WinUI 原生控件要求。

| 阶段 | 用户看到的反馈 | 可用动作 |
| --- | --- | --- |
| 点击后、请求及首字节等待 | 立即显示“正在下载…”及按钮内加载指示，无须等首个下载回调 | 禁用重复点击 |
| 下载中，总量已知 | “正在下载 42%”，根据已接收字节显示确定进度；只在按钮背景填充，不另设进度条 | 保持状态可见 |
| 下载中，总量未知 | “正在下载…”及按钮内加载指示，不把未知进度显示为 0% | 保持状态可见 |
| 下载完成、验签和准备中 | “正在校验…”或“正在准备安装…”，使用不确定指示 | 不显示更新成功 |
| Arch 预览已准备 | 目标版本和新增/升级依赖列表，显示“安装”与“稍后” | 可审阅、安装或关闭 |
| 安装中 | Windows“正在安装…”；Arch“请完成系统授权，随后安装…”，不伪造授权完成事件或百分比 | 禁用重复操作及关闭；不强杀 Pacman |
| 失败或授权取消 | 原位置显示简短原因和下一步；错误用 Alert，取消按普通状态提示 | 重新检查或关闭，重试前重新核对候选 |
| 安装完成 | Windows 沿用官方安装器退出行为；Arch 复核已装版本后显示“更新完成，请重新启动” | Arch 可退出后重新打开 |

只保留一个按钮外观的进度控件：运行时以 Mantine Button 的公开多态 API 显示只读 progressbar，背景表示真实下载百分比；未知总量、校验和安装使用内置加载指示与短阶段文字。没有独立横条或重复状态段落。确定进度设置 aria-valuenow，未知时省略；仅阶段变化用 polite live region，不逐字节播报。高对比和减少动态效果下保留文字与数值。更新区域删除启动检查副标题、预览版本解释及重复操作指南，只保留当前结果、安装重启提醒和实际错误。

### 设计验收

产品与现有离线预览同步覆盖 Windows / Arch、简中/英文、浅色/深色、高对比和减少动态效果。检查 640×600、900×700、1200×800 及 200% 文本缩放，长日志与依赖列表不能遮挡操作区；键盘可操作，焦点可见，关闭后返回可用触发控件。增加首字节延迟、未知总量、正常百分比、下载失败、验签失败、授权取消及完成状态的演示/验证。原生 WebView、系统授权和真实安装验收分别记录，离线预览不能代替。

## PKGBUILD 与删减清单

保留一份仓库内的 `packaging/arch/PKGBUILD`，供 CI 用 makepkg 封装已编译文件；由现有发布脚本准备输入，不把 Rust/前端构建重复跑一次，不为最终用户增加 makepkg 安装步骤。版本及依赖与既有检查同步核对。

本版移除：

- `Index` / `Release.archPackage`、`promote()`、promote CLI、`channels.json` 的创建/读取/覆盖及专属测试；API 版本筛选复用现有 SemVer。
- 工作流 `arch_package` 输入、环境变量、AUR 包名选择、发布配方 `.SRCINFO` 和 `aur-recipe.tar.gz`；现有中间 Linux tar 如仍供 makepkg 使用则仅留构建内部，不再作为第二种 Linux 用户分发。
- 前后端 `method: aur`、AUR 包名字段、yay/paru 文案、AUR 链接和“配方尚未发布”分支；更新弹窗保留 Windows / Pacman 两种真实安装行为。
- 旧 alpha 双协议、迁移/回退、固定 latest 包别名、可配置任意仓库或更新源；不保留无调用的泛化类型或抽象层。

alpha.3 不为旧 alpha 留兼容桥。旧 alpha.1、alpha.2、updates 的 Release/tag 已删除；本轮 API 复核发布列表与 tag 列表均为空。历史 evidence 保留。

## 实施顺序与验收

| 顺序 | 最小工作 | 完成依据 |
| --- | --- | --- |
| 1 | 共用 Releases 发现、标准 latest.json；删除渠道协议 | 无稳定版、预览筛选、版本顺序、分页/限流、资产不完整与非法来源均有有效检查 |
| 2 | Arch 包签名和 updater 下载验签；同步公钥及许可材料 | 真包能下载并验签；内容篡改、错误公钥/签名和摘要不符均在提权前拒绝 |
| 3 | 固定 Pacman 平台操作及现有弹窗接入；删除 AUR 分支 | 隔离环境中预览、授权、安装结果复核、取消和失败路径有效；Windows 流程未被破坏 |
| 4 | 收敛 PKGBUILD、CI 资产/上传顺序及发布重试 | 双端包完整，latest.json 最后上传；中断重试只补缺项，不改 tag/已有资产 |
| 5 | 同步产品、离线预览、README、当前行为/测试/CI 文档及版本号 | package/Tauri/Cargo/锁文件与英文 notes 一致；所有“计划”有实施证据后才改成“完成” |
| 6 | 发布 alpha.3，核对旧元数据已删除 | 匿名访问新 manifest/包成功，Release 为公开 Pre-release；实际应用能发现并使用新协议 |

必要验证沿用 [本地测试入口](../testing.md)，按变更运行前端构建/测试、Rust fmt/test/clippy、发布脚本测试及 actionlint；签名依赖扩大到 Linux 后重新收集两端许可材料并 `--check`。不为静态文案另写重复测试，不重跑无关完整播放/HDR 矩阵。

重点风险用现有测试设施覆盖：

- 版本发现：仅 alpha、稳定与预览混合、稳定高于 rc、较旧版本后发布、不完整 Release、API 分页和限流；错误不当成最新。
- 包信任：manifest/tag/安装包版本一致，两个平台正确匹配，缺签名/错误签名/截断下载/错误摘要拒绝；临时路径替换、过期候选和未保存草稿阻断。
- Pacman：隔离系统首次安装及依赖拉取、由同一新协议构造的旧/新版本升级、取消 Polkit、缺授权代理、数据库锁/过期、依赖冲突、事务退出非零以及完成后版本核对；确认事务提交期不强杀、不会清全局缓存。先用 mock 验证参数/拒绝路径，真实写系统仅在隔离 Arch/CachyOS VM；普通 MPVKIT_TEST_MODE 继续禁止真实包管理器。
- 重启：更新后旧进程显示待重启，退出后从桌面打开实际新版本，仍只有一个窗口，配置和系统 MPV 未被改写。不要求 alpha.1/alpha.2 通过旧索引升级。
- Windows：保留 updater 签名/下载失败测试；隔离用户/VM 实测 NSIS 更新及单实例退出/重启。Linux 运行与签名测试不能替代 Windows 安装结论。
- UI：现有 Mantine 弹窗与离线预览同步；两种语言、窄窗/长日志/固定操作区、焦点恢复、下载/授权/失败/完成状态；不加页面、持久开关或第二套进度系统。

## 本轮交接

已完成代码：共用 Releases 分页发现、标准 latest.json、双端官方 updater 下载验签与 SHA256SUMS/digest 核对、Arch 私有暂存/Pacman 预览和授权安装、WinUI 3 进度及就地错误。删除渠道/AUR 分支、发布配方/中间 tar 分发；仓库 PKGBUILD 只供 CI 打包。正式签名资产与 manifest 最后上传、不可变重试均已接入 Actions。

已完成本地验证：前端 TypeScript/Vite 与翻译/组件测试；发布 17 项、CI 分类 1 项；Windows Rust 45 通过 / 4 项显式条件测试忽略、fmt/clippy；Windows debug 单程序构建；离线预览覆盖双平台进度/取消/失败/完成及既有布局，且无运行时错误/网络。两平台许可材料 collector 检查无缺失（Windows 331 包 / 587 材料；Linux 401 包 / 704 材料）；完整人工许可审阅仍未完成。Actionlint 通过。

最终 Windows EXE 原生回归 4/4 通过（115570ms）：更新、WinGet、设置页、预编辑/保存/过期保护；远端 CI 结果继续补充。截图在 .local/e2e-artifacts/ 与 .local/ui-v016/，已目视检查进度、固定操作区和完成状态。测试全用隔离配置根，未调用真实系统安装器/包管理器。

当前主机没有 Linux 发行版/图形会话，Linux Rust、makepkg、Pacman 事务和隔离根载荷检查由 PR/Release 的 Arch 容器执行。隔离包测试 --nodeps/--noscriptlet 只验证文件及包数据库，不代表首次依赖拉取、Polkit/KDE Wayland 或跨版本重启；Windows 真实 NSIS 跨版本安装也未在隔离用户/VM 实测。不重跑与本版无关的完整播放/HDR 矩阵。

远端状态：旧 Release/tag 均为空已复核；初次实现已提交 f8a0bd7 并推送 dev，[PR #8](https://github.com/Leovikii/mpvkit/pull/8) 检查中。用户随后要求仅保留按钮进度和精简文案，已同步产品/预览，Windows 原生更新专项再次通过（17017ms）；最终 CI、公开 alpha.3 及 dev/main 同步结果待补充。
