# v0.1.8 验证记录

以下为各批次原始验证记录，保留当时命令/产物/限制；当前结果与下一任务统一见 [交接](plan.md)和[测试入口](../testing.md)。旧批次的“本轮/最新/无 Linux 环境”只指当时状态。

2026-10-26 首轮：用户批准经典 uosc 布局方案及开发测试；实施前 fetch，HEAD 与 origin/main 均为 v0.1.7 `e41cfda94081cf34508fe0c4f66c9f9e4188ac5d`。本轮仅本地实现、隔离测试与单 EXE 构建，没有提交、推送、CI、安装包或历史开发版迁移。

以下至“WinGet 核心与控制台”章节前为早先自下载核心批次，保留作追溯；当前代码、产物及验证以末尾新批次为准。2026-10-26 最新 Linux 原生测试见末尾“本机 Linux 原生验证”；前文“无 Linux 环境”仅描述先前 Windows 批次。

## 早先环境与产物

- Windows 11 Enterprise LTSC 2024 x64，实际桌面 DPI 200%；固定 Windows MPV `0.41.0` 开发构建，归档 `mpv-x86_64-20261020-git-6a262d11f6.7z`；uosc 5.13.0。
- Rust/Cargo 1.98.1、pnpm 11.25.0；本机 Node 24.22.1 与仓库声明的 24.19.0 不同，pnpm 发出 engine 警告，构建/检查通过；未修改工具链要求或依赖版本。
- `pnpm tauri build --debug --no-bundle` 通过，`bundle.active=false`。完整删除截图功能后的当批单 EXE：`src-tauri/target/debug/mpvkit.exe`，31474688 字节，SHA-256 `0bfbd1b751cd7b744db6d267649d75aa28ecf40f0e9065cbb46305bfd1a44f4e`。此前加入启动窗口默认值的产物为 `18f5ee72c8a7711c1661dcada51ac32d3a90b261863fe0c70e1283eba48a6959`（31533056 字节）；首轮布局使用 `1cd0fc4d35a9104bb578e262bb6cc228a0df6eeb2f6e132eeabfa4625f4dd547`，禁用背景单击后为 `604ea5cd3652d6e14cb7236d316e701209b10b63dee0c094f2b21bf854c0d69d`。各批验证见下文。
- 所有安装和播放测试使用新建临时根，先验证 setup_status 确实指向当批隔离根，再做修改；WebView2、媒体、MPV 配置及命名管道独立。系统登记采用测试私有后端，没有修改真实默认播放器、配置或注册表。

## 检查结果

| 检查 | 结果 |
| --- | --- |
| TypeScript / Vite 与单 EXE 构建 | 通过，869 模块 |
| `pnpm test:frontend` | 2 项通过 |
| `cargo fmt --check` / clippy `--all-targets -- -D warnings` | 通过 |
| `cargo test --locked` | 57 项通过，6 项默认忽略 |
| 显式固定归档部署/卸载测试 | 1 项通过，67 文件 / 158897147 字节；校验、解包、部署、卸载 |
| Windows 原生安装、保存、界面、播放、预编辑回归 | 原有 4 项通过，新增 v018 专项通过，详见下文 |
| Windows/Linux 依赖材料 collector `--check` | 321 包 / 567 材料、358 包 / 620 材料，无缺失 |
| 两平台 uosc 固定 ZIP、原版前缀、扩展、部署结果与覆盖配置摘要 | 与真实字节一致；Linux helper 为归档内原版 ELF，仅核对材料，未执行 |
| v0.1.6 独立配置预览构建与 check.mjs | 通过，无运行时错误或外部网络请求；仍仅为设计预览 |

默认忽略的 6 项为固定归档、在线下载、图形样本、媒体矩阵、显式真实核心契约及私有 Offreg；其中固定归档已单独显式通过。真实图形播放由本轮原生 Node 测试覆盖，不将其冒称全部历史媒体矩阵/注册表 API 验收。首次普通沙箱运行会阻止测试的原子文件替换与回环连接，最终检查在获准的隔离本地进程中完成。Rust 链接器库生成提示及缓存 hard-link 回退不影响结果。

## Windows 实播与配置保护

执行 `node --test --test-concurrency=1 scripts/native-v016.test.mjs`：原有 4 项通过；新增随机 EOF 用例曾在文件加载完成前 seek，导致测试命令失败。修正测试等待后，以 `--test-name-pattern="native v018"` 单独复验通过，没有放宽产品校验。此前 3 个 Rust 启动测试的模拟 MPV 缺少 v0.1.7 要求的 target-colorspace-hint / target-trc / target-prim，已补齐模拟能力列表，产品能力保护保持。

- 配置实际生成于 portable_config/script-opts/uosc.conf，可完整预览；未改文件保持 mtime。手改后的旧计划拒绝执行，新预览保留手改。重置与恢复也遵守既有脚本选项手改保留规则，不把“重置 GUI 管理区”当作覆盖全部用户布局。
- 实际安装的 uosc main.lua 与来源清单的部署摘要一致；原版前缀和附加桥接分别核验。测试只在额外私有副本追加布局坐标观察代码，安装文件没有测试接口。
- 640/900/1200 原生宽度下，菜单、上下集、播放/暂停和全屏保留；中央播放按钮中心与窗口中心偏差小于 2 像素。200% DPI 下次要入口先收起；运行时改变 uosc 缩放并触发窗口尺寸更新，scale=4 时核心按钮仍可用。全屏截图及正常/高 DPI 控制栏已目视检查。
- 鼠标点击播放/暂停与空格一致；点击原生播放模式按钮打开四项菜单，菜单键盘选择不触发背景暂停。四模式实际 loop-file / loop-playlist / shuffle 均与选择一致；有限次数循环显示自定义，外部 uosc/shuffle 立即反映到实际模式。
- 640 宽下，收起的信息、播放模式和倍速仍可从原生菜单访问。倍速菜单列出六项并标记当前值，实际键盘选择 0.75× 后 MPV speed 与之相符。
- 无显式多项列表时按同目录上下集，第一项向前保持边界；仅一个文件的目录中前后操作保持当前文件。多项列表随机手动和 EOF 切换均不打乱列表顺序；顺序结尾停留、列表循环回首、单集重复、关闭 keep-open 后退出均通过；重启进程恢复默认顺序模式。
- 信息按钮打开实际 MPV stats，Shift+i 关闭，再由 Shift+i 打开并由信息按钮关闭，截图显示同一内置面板；未增加另一份信息窗口。
- 原有回归包含组件与配置独立安装、预览取消、手改/过期保护、备份恢复、GUI 关闭后的原生 helper 保存、52 个真实 MPV 轨道偏好用例、快捷键/原生菜单优先级、续播仅保存位置、PNG/JPEG 截图与字幕开关、文楷字体摘要/加载和 thumbfast 实际缩略图。

本地证据位于 `.local/e2e-artifacts/`：

| 场景 | 子目录 |
| --- | --- |
| v018 最终控制栏、模式、窄窗菜单、倍速与 stats | `mpvkit-v018-00jaGc`，含 result.json、layouts.json、3 次启动完整日志及原生截图；从随机模式退出再启动恢复顺序 |
| 完整按钮栏 / 信息面板目视检查 | `mpvkit-v018-Qq3IDI` 的 layout-full.png / stats-button.png；同版同一 EXE 的实际播放器截图 |
| 安装、手改与原生保存 | `mpvkit-v016-07pu6B` |
| 双语/主题/导航 | `mpvkit-v016-ui-l9i7ju` |
| 真实播放与截图 | `mpvkit-v016-player-MumISe` |
| 预编辑、mtime、过期与响应布局 | `mpvkit-v016-draft-3qOTZS` |

## Linux 与剩余边界

本机没有 Linux/Wayland 环境，Rust 只安装 Windows MSVC target。未构建 Linux ELF，未运行 Linux 专用 Rust 测试、GTK/WebKitGTK、Unix IPC、系统 MPV 或 Wayland 实播。保留 Linux 原生目录检测和 helper 规则；共享 uosc.conf、input.conf、Lua、组件扩展及两平台来源材料已核对。Linux 配置测试已同步第四个预览文件与共享 uosc 配置断言，留待原生环境运行。Windows 运行共享 Lua 不能替代这些 Linux 检查。

Windows 仍未覆盖全部实体 DPI/显示器组合、屏幕阅读器、长期媒体、实际多语言字幕/复杂轨道、真实流媒体、HDR 效果及能耗。本轮缩放检查不等于切换系统 DPI 的全矩阵测试；极窄窗口仍受 uosc 原生最小空间约束。upstream uosc 运行时改变 scale 需尺寸更新后重新计算，测试显式触发窗口重排；没有为此扩展一套布局引擎。完整许可/干净系统等既有验收缺口保留，releaseReady=false。

下一步：在 Arch x86_64 / KDE Wayland 环境执行现有 Linux 配置与原生集成测试，补齐本版控制栏、stats、模式/EOF、缩略图和缩放检查，再继续设备与真实媒体验收。

## 追加：禁用视频背景单击暂停

2026-10-26 用户要求消除双击全屏伴随暂停的问题。共享 resources/input.conf 的 MBTN_LEFT 改为 ignore，双击仍为 cycle fullscreen；空格和 uosc 中央按钮继续切换暂停。Windows/Linux 生成器及独立预览读取同一个输入资源，没有引入延迟或自定义双击调度。既有 input.conf 继续遵守手改保留，不自动覆盖真实用户配置。

重新执行 `pnpm tauri build --debug --no-bundle`，TypeScript/Vite 与单 EXE 构建通过；该批产物摘要保留在本文开头。`node --check scripts/native-v016.test.mjs`、`git diff --check` 通过。

显式原生 v018 专项重新通过，证据 `.local/e2e-artifacts/mpvkit-v018-WSJqQZ`。使用实际 MPV 的连续按下/释放序列，由播放器自身识别双击；分别在播放和暂停状态下确认背景单击无效、双击进入/退出全屏，测试用私有脚本观察到 pause 属性没有任何额外变化（排除“先暂停再恢复”）。随后中央按钮与空格均正常切换，模式、信息、菜单及原有专项其他断言继续通过。初次注入测试的半像素鼠标坐标已改为整数；也去除了重复注入的显式双击消息，避免与 MPV 自动识别造成两次全屏切换。Linux 原生测试边界保持上述说明。

## 追加：统一启动窗口与切集保持尺寸

2026-10-26 用户授权实施。新建配置的共用默认区加入 autofit=75%x75%、auto-window-resize=no，能力检查同步要求这两项；未新增 GUI 字段或 Lua。已在线只读核对 [MPV v0.41.0 官方参数手册](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst)，并使用本版固定 Windows MPV 实播。用户手改默认区在 GUI 保存时继续保留，Rust 回归显式验证自定 60% / 自动调整开启不会被覆盖；旧开发环境不做字段补齐，真实配置没有改动。

当前 EXE 摘要见本文开头。重新构建 TypeScript/Vite 与单 EXE 通过；cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings 通过；cargo test --locked 为 57 项通过、6 项默认忽略。独立预览构建通过；脚本语法和 git diff --check 通过。Linux 配置断言同步两项默认值，但本轮没有 Linux 原生环境，未运行 Linux 专用测试或 Wayland 播放。

新增可复现测试 `native v018 startup window sizing and stable resize` 通过，使用新隔离根、固定 FFmpeg 合成媒体与实际 MPV，不传 geometry 覆盖默认值。测试 helper 在核对精确 PID/可执行文件后，以物理坐标读取原生客户区、DWM 可见边框、显示器与工作区；只在测试中调整该窗口，不修改桌面 DPI、任务栏或用户偏好。

本机显示器为 3072×1920、桌面 200% 缩放，实际结果：

| 媒体 | 初始客户区 |
| --- | --- |
| 640×360 / 1920×1080 / 3840×2160 | 均为 2301×1294 |
| 1080×1920 竖屏 | 784×1395 |
| 2560×1080 超宽 | 2301×970 |

以上保持各自画面比例，完整可见窗口边框均在桌面工作区内。75% 的实际尺寸受任务栏与原生边框影响：首次测试以整块显示器作为精确填充基准，竖屏断言失败；核对 MPV 日志的 max content size 与 Win32 实际边框后，测试改按工作区扣除原生边框核算，产品参数没有因此改变。

4K 窗口连续切换到竖屏、超宽及低分辨率视频后，客户区保持 2301×1294。通过原生窗口 API 调整外框至 1200×800，客户区为 1174×729，随后切换 1080p/竖屏以及进入/退出全屏仍保持该尺寸。各媒体重新独立启动又恢复各自默认适配尺寸；没有跨进程记忆窗口。五份播放器日志未发现 Lua/选项解析错误。

证据目录 `.local/e2e-artifacts/mpvkit-v018-window-z8hHME`：result.json、window-rects.json 保留实际边界，五份日志与原生窗口内容截图保留媒体结果；4K 与竖屏截图已目视检查。该批只重跑新增窗口用例及相关 Rust/静态检查，不冒称旧版完整原生回归在此 EXE 上重新执行。多显示器、任务栏自动隐藏/特殊停靠、其他 DPI 及 Linux Wayland 仍待相应设备验证。

## 追加：完整删除截图功能

2026-10-26 用户明确要求删除全部相关代码。删除 GUI 三项设置、双语文案、前后端设置字段与格式枚举、路径验证/两平台图片目录查询、配置生成/GUI 管理项、MPV 能力要求、Lua 消息与 uosc 菜单项。仅供截图目录使用的 choose_path IPC、Windows/GTK 选择器及 Windows 两项 API feature 一并删除；Linux 不再调用 xdg-user-dir。thumbfast 缩略图保留，无旧字段读取或迁移，不删除既有用户图片。

生成配置的既有 Rust 测试增加断言：生成的 conf/Lua 不含 screenshot；原生回归验证 playback 仅有 hdr/keepOpen、GUI 无截图设置、uosc 菜单无截图入口。旧截图行为用例已删除，保留续播、快捷键、字体和真实 thumbfast 测试。测试取证使用的 page.screenshot、MPV screenshot-to-file 和 Win32 窗口抓取不进入产品功能；本文前述截图功能验证属于删除前历史批次。

本批 `cargo test --locked` 57 项通过、6 项默认忽略；fmt/clippy、前端 2 项测试、TypeScript/Vite、单 EXE 构建通过。Windows/Linux 依赖 collector 重新生成并 --check 通过（321/358 包，567/620 材料，无缺失），未新增依赖。独立预览构建及离线 check.mjs 通过，已检查窄窗暗色预览。普通沙箱的 Edge 启动/临时文件替换受限，改在获准的隔离进程运行后通过；首次 collector 因 shell 缺少 Cargo PATH、随后因 Cargo.toml 输入摘要变化失败，补齐 PATH 并同步摘要后通过，没有跳过校验。

当前 EXE 上完整运行 `node --test --test-concurrency=1 scripts/native-v016.test.mjs`，6/6 通过（约 236 秒）。每项从全新隔离根开始，仍使用私有登记后端；覆盖启动尺寸、uosc/信息/模式、独立安装及配置事务、52 个真实轨道用例、双语主题、快捷键/菜单、续播/手改、字体/缩略图、预编辑/文件时间戳/过期计划及响应布局。证据均位于 `.local/e2e-artifacts/`：

| 本批验证 | 目录 |
| --- | --- |
| 启动尺寸与切集 | `mpvkit-v018-window-Qao2bp` |
| uosc、原生菜单无截图、模式及 stats | `mpvkit-v018-C7XTDj` |
| 安装、手改与保存 | `mpvkit-v016-b9gXYO` |
| 双语、主题与目录 | `mpvkit-v016-ui-QKuwpA` |
| 播放、精简设置、续播、字体及缩略图 | `mpvkit-v016-player-V6pSU1` |
| 预编辑、时间戳、保护与响应布局 | `mpvkit-v016-draft-VQuLEb` |

Linux 原生构建、专用 Rust/GTK 测试和 Wayland 实播仍未运行；本批仅删除两端相应实现并核对共享生成配置与来源材料，Windows 成功不能替代 Linux 验证。


## WinGet 核心与控制台（2026-10-26，当前批）

本批替换 Windows 核心管理和配置根，以下结果对应新 EXE，不能套用前面每日构建的结果。当前单程序 `src-tauri/target/debug/mpvkit.exe`：**30798336 字节**，SHA-256 **`030cd51d82e452903b373d75b9d4cda8ee13e0221b6c39a18c26382a3c8e6829`**；通过 `pnpm tauri build --debug --no-bundle` 生成，`bundle.active=false`。没有安装包、CI、提交或推送。

### 真实渠道与二进制核验

- 本机 WinGet `v1.29.380`；App Execution Alias 精确位置 `%LOCALAPPDATA%/Microsoft/WindowsApps/winget.exe`。普通工具沙箱不可见，但获准只读主机查询可见并返回版本；没有因此改变产品路径规则或回退自下载。
- 使用真实 WinGet 的 `show` / `download --id shinchiro.mpv --exact --source winget`，只查询和下载；**没有执行真实安装器或安装脚本**。当前包版本 0.41.0，源安装器 [0GMou/mpv2winget v0.41.0](https://github.com/0GMou/mpv2winget/releases/download/v0.41.0/mpv_installer_x64.exe)，清单见 [winget-pkgs](https://github.com/microsoft/winget-pkgs/blob/master/manifests/s/shinchiro/mpv/0.41.0/shinchiro.mpv.installer.yaml)。安装器 SHA-256 `159441c52fd74755cb74c830f661fcd9517c77056887c27ac0569876b600f0c9`，WinGet 验证和本地复核一致。
- 固定来源的 innounp 2.67.11 仅用于研究提取（本地 ZIP SHA-256 `ac1d98bba6588072ade06163781938df274f444bec7318069668608d1e5faae8`，下载自作者仓库固定 Git blob `38095659b9f86bdeee339a56b4d8cd4485dabece` 并校验 Git 对象）；不成为产品依赖。旧 innoextract 1.9 不支持此 Inno 6.7 格式，改用可识别格式的提取器，没有运行安装器。
- 提取的 **mpv.exe 120308736 字节**，SHA-256 **`1645e57569c0af58a70c61257f3716f2ee6eb31bb170edc51f3f25a0b8d0d99c`**；实际 `mpv v0.41.0-244-gaf9c81fa1`（2026-03-02 构建）、libplacebo 7.360.0、FFmpeg N-123099-g862338fe3。这证明包标签不是上游 v0.41.0 精确提交，完整上游提交仍未核对。
- 提取 Inno 脚本显示固定 AppId `MPV_Player_Automated`、默认 Program Files/MPV Player、管理员安装与系统 Open with 登记；产品只读对应 `_is1` 卸载键的 InstallLocation，允许 HKLM/HKCU 及两个注册表视图。此为包内容核验，**没有实际主机安装/UAC/登记/升级验收**。包可能添加自己的打开方式，MpvKit 的开关只管理自身登记。

### 当前检查结果

| 检查 | 结果 |
| --- | --- |
| TypeScript / Vite / 单 EXE | 通过，869 模块；无 bundle |
| Rust `cargo test --locked` | **59 通过，6 默认忽略** |
| fmt / clippy `--all-targets -- -D warnings` | 通过 |
| 前端 `pnpm test:frontend` | **2 通过** |
| Windows 原生完整回归 | **7/7 通过，256244ms**；轨道子用例 **52 通过** |
| 离线预览 build / check | 通过，无运行时异常或网络请求；已目视检查 |
| Windows / Linux 依赖 collector `--check` | **319 包 / 565 材料；356 包 / 618 材料**，无缺失 |
| 当前文档本地链接 / `git diff --check` | 通过 |

6 项默认忽略仍为显式固定资产、在线资产、图形/媒体矩阵、真实核心契约和私有 Offreg。当前原生 Node 套件已覆盖本批实际核心/增强部署与实播，未把它等同所有历史专用矩阵。Node 本机 24.22.1 与仓库 24.19.0 声明不同，engine 警告保留；缓存 hard-link 回退和 MSVC 创建导入库信息不影响结果，没有升级固定工具链。

最终命令：`node --test --test-reporter=tap --test-concurrency=1 scripts/native-v016.test.mjs`。显式 `MPVKIT_CORE_SOURCE` 指向上述校验后提取的核心目录，`MPVKIT_FFMPEG` 为固定 SHA-256 `7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1` 的合成媒体工具。所有配置、WebView、媒体和登记在新隔离根；Debug WinGet 只用编译的本地进程夹具，不会退回真实包管理器。

| 本批证据（`.local/e2e-artifacts/`） | 目录 |
| --- | --- |
| WinGet 失败/重试/安装/升级/归属 | `mpvkit-winget-tvEpW6` |
| 启动尺寸、4K、比例切集、手调及全屏 | `mpvkit-v018-window-lIrtOk` |
| uosc 布局/模式/信息/右键控制台 | `mpvkit-v018-ktfSyP` |
| 安装、手改、保存、52 轨道用例、原生 profile/helper | `mpvkit-v016-tEQTPg` |
| 双语、主题、目录与焦点 | `mpvkit-v016-ui-O5c7fS` |
| 原生快捷键/控制台/续播/字体/缩略图 | `mpvkit-v016-player-9HLfBi` |
| 预编辑、文件时间、过期保护及响应布局 | `mpvkit-v016-draft-mfJziz` |

### 关键覆盖与修正

- WinGet 缺失且 App Installer 入口可操作；源查询失败、输出超限、45 秒源超时、用户取消、授权不足、一般失败、成功退出但没有核心均被拒绝，配置目录未创建。预览取消没有安装；重新预览重试后可继续增强安装。
- 核心完成后重新探测实际版本/能力，增强清单不含 mpv.exe/DLL，系统核心健康独立于增强归属；升级无更新成功返回，核心改变使旧计划失效。固定包与源参数每次一致，没有核心卸载调用。移除增强后核心及配置字节保留。
- 单元覆盖 Windows 配置优先级、相对路径、额外配置歧义、多核心歧义/重复位置以及错误码。原有目录重解析点、覆盖保护、恢复、跨进程锁和过期文件测试保持通过。共享配置改为直接使用检测根，Linux 的无安装记录/移除增强后的核心状态断言同步。
- 真实 MPV_HOME 无 --config-dir 启动可读隔离配置；GUI 关闭/原程序移动后 profile helper 保存仍可用；托管启动、手改/结构破坏、重置/恢复及安装登记事务继续通过。
- 反引号绑定 `commands/open`；控制台打开属性为真，空间/上下键/Enter 不改变 pause/fullscreen/volume，Esc 后音量恢复。右键实际菜单键盘选中“控制台”打开同一原生面板，截图 `console-menu.png` 已目视检查。信息/模式/倍速与无截图菜单回归通过。
- 首轮失败来自旧测试目录/状态假设、直接启动测试的 windowsHide 导致播放窗口隐藏，以及异步消息未等菜单/输入绑定就绪；已修正测试。轨道脚本改为显式配置根与核心哈希，移除旧核心归属假设。最终完整套件无失败，没有放宽产品保护。

### 尚未验证

无隔离虚拟机，未在主机执行真实 WinGet install/upgrade、管理员 UAC 或第三方包实际登记；核心来自真实渠道但安装动作使用隔离夹具。正式使用前还应在干净隔离 Windows 验证这些环节，现有同版本损坏包可能需要用户通过 WinGet 重新安装，产品不会强制重装或移除已有包。

本机仅安装 Windows Rust target，没有 Linux/Wayland 环境：Linux 原生 ELF、平台专用测试、真实 `/usr/bin/mpv`、系统配置加载和 KDE Wayland 播放均未运行。可行范围已完成共享逻辑 Windows 实测、Linux 条件代码/断言及来源材料检查、全模块格式解析和 Linux 依赖材料验证；这些不代表 Linux 原生通过。实体 HDR/高刷/功耗、完整媒体矩阵、来源/许可审阅仍按现有验收台账推进。

## 追加：自动连播、队列结束行为与字幕搜索（2026-10-26，最新批）

本批在主配置区新增默认开启的“自动连播”开关，并将结束行为改称“全部播放结束后”。主区顺序为画质、HDR、自动连播、全部播放结束后；结束行为沿用 keep-open=yes/no。连播用 MPV 原生 autocreate-playlist=filter/no，字幕使用原生 sub-auto=exact 和相对目录追加，不增加 Lua 扫描器或新依赖。当前字段为 hdr/keepOpen/autoPlaylist，严格读取当前格式，无旧字段补全或开发版迁移。

本批单 EXE：`src-tauri/target/debug/mpvkit.exe`，**30801408 字节**，SHA-256 **`e935b4bf3e205ef7365c586ee2d6cdda792b734caf1189267c6d1d0766ec1c48`**。`pnpm tauri build --debug --no-bundle` 通过，bundle.active=false；之前章节的摘要属于历史批次。无安装包、CI、提交、推送或实际用户配置变更。

| 检查 | 本批结果 |
| --- | --- |
| TypeScript / Vite / 单 EXE | 通过 |
| Rust `cargo test --locked` | **59 通过，6 默认忽略** |
| fmt / clippy `--all-targets -- -D warnings` | 通过 |
| 前端测试 | **2 通过** |
| Windows 完整原生回归 | **8/8 通过，286839ms**；另含 **52 个真实轨道子用例** |
| 离线预览 build / check | 通过；空格切换、关闭后的配置预览、草稿/保存/布局均通过，无运行时异常或网络请求 |
| Windows / Linux 依赖材料 `--check` | **319/356 包，565/618 材料**；无缺失 |
| 当前文档本地链接 / `git diff --check` | 通过 |

原生完整命令仍为 `node --test --test-reporter=tap --test-concurrency=1 scripts/native-v016.test.mjs`，使用前一节记录的真实 WinGet 提取核心及固定媒体生成器摘要。每项从新隔离根开始，模拟 WinGet、私有登记后端；真实安装器/UAC 和系统默认应用未参与。

| 本批证据（`.local/e2e-artifacts/`） | 目录 |
| --- | --- |
| WinGet 安装/失败/重试/升级/归属 | `mpvkit-winget-jE2Ara` |
| 统一启动尺寸与切集 | `mpvkit-v018-window-dE56WT` |
| uosc 布局/模式/信息/控制台 | `mpvkit-v018-DzFTZN` |
| 连播、目录队列与字幕语种 | `mpvkit-v018-autoplay-qWVv7H` |
| 安装/手改/profile/helper/52 轨道 | `mpvkit-v016-XbL5QH` |
| 双语/主题/路径 | `mpvkit-v016-ui-FjwkJH` |
| 快捷键/控制台/续播/字体/缩略图 | `mpvkit-v016-player-Rnugpa` |
| 预编辑/时间戳/过期保护/响应布局 | `mpvkit-v016-draft-Yn509D` |

新增实际 MPV 用例断言：同目录第1集.mkv、第2集.mp4、第10集.wav 按自然顺序组成真实队列，从所选第2集开始；封面、字幕、文本、归档、播放列表、隐藏媒体及嵌套目录不进入队列。默认关闭时第2集 EOF 进入音频，第10集结束后进程退出；保存“保留窗口”后，中间仍切集，最后音频结束后进程存活且暂停。关闭连播后当前文件队列仅一项；显式传入两项播放列表仍自动进入下一项。uosc 回归已取消手动重复追加同目录文件的旧测试假设，列表/随机/循环测试保持通过。

字幕测试在媒体目录以及 Sub、Subs、Subtitle、Subtitles 中放入匹配及其他集的字幕，精确核对实际 track-list：允许无语种、zh-Hans、zh-Hant、chs、en、ja、es 后缀，en.forced 标记 forced，en.sdh 标记 hearing-impaired；其他集字幕未加载，每个文件只加载一次。Windows 四项小写路径可找到首字母大写目录；Linux 生成代码另加首字母及全大写形式，专用断言已更新但未在 Linux 运行。逐项 append 不依赖平台的冒号/分号分隔符。字幕目录与目录过滤参数保持可手改，Rust 部署测试验证 GUI 保存后保留自定义字幕目录与 directory-mode。

新增用例开发中修正了切页动画等待、首装确认按钮匹配，以及把视频 eof-reached 属性用于最终纯音频的测试假设；纯音频完成以原生暂停状态和进程存活判断。最终完整回归全部通过，未放宽产品配置保护或改写 MPV EOF 行为。已目视检查本批 1200/640 宽度原生界面及离线预览，固定底栏和开关均正常，200% 文字检查通过。

Linux 无原生测试环境，`rustup target list --installed` 仅 Windows x64；本批没有 ELF 构建、Linux 专用测试或 Wayland 实播。可行范围为共享生成/部署逻辑的 Windows 测试、Linux 条件源码及参数审阅、Rust 全模块格式检查和 Linux 依赖材料检查。下一任务仍为可用 Linux 环境下的原生构建、大小写目录/字幕加载、实际队列及 KDE Wayland 实播；真实 WinGet 安装/UAC 仍需隔离 Windows 虚拟机，既有 HDR/高刷/功耗与来源许可缺口保留。


## 组件管理与自动清理（2026-10-26，本批）

基线为已推送的 eb8425005e9b182e31858c50d694ab4954ccb6f0。实现范围见[维护方案](maintenance.md)及[最新交接](plan.md#已完成与证据)。实现与验证完成后，用户已追加授权将本批提交并推送至 origin/main；没有 CI、安装包、真实系统包安装/卸载或开发者配置变更。

- Rust 62 通过 / 6 默认忽略；fmt、clippy all-targets -D warnings 通过。覆盖成功/失败/中断清理、当前事务恢复副本被改、未知暂存项保留、空目录边界、手改卸载保留、完整默认重置、日志活动/归属/预览过期保护，以及卸载后未受管理配置的明确覆盖预览和执行前复核。
- pnpm build / test:frontend 2 项、离线独立预览检查通过；含顶部组件管理、卸载取消、重置提示、无历史备份勾选、双语/主题/640 宽/200% 文字和焦点锁。两平台依赖材料检查通过（Windows 319 包/565 材料，Linux 356 包/618 材料）。Node 24.22.1 与声明 24.19.0 的既有 engine 提示保留，未升级依赖要求。
- Linux 核心调用模块完整纳入 Windows 测试编译，发行版/固定参数契约通过；不调用真实 pacman。Linux 专用集成测试更新为卸载删除未改配置、保留续播与系统核心并清理事务；因本机仅 Windows target，未执行 Linux 专用测试、ELF 构建、polkit 授权或 Wayland 实播。
- 最终 EXE 的 Windows 完整原生回归 **8/8 通过，290222ms**，另含 **52 个真实 MPV 轨道子用例**。核心安装/升级/卸载使用隔离 WinGet 进程夹具，播放使用此前校验的真实 MPV；没有调用主机包管理器或修改真实配置/登记。

最终单 EXE：`src-tauri/target/debug/mpvkit.exe`，**30661632 字节**，SHA-256 **`2f6e3aef909339349fb97ff5abe42589bd2b7ee51d7697362acc9e569c403ccd`**。`pnpm tauri build --debug --no-bundle` 通过，包含最终的日志归属 guard 和重装预览修正；bundle.active=false。

完整回归命令：`node --test --test-reporter=tap --test-concurrency=1 scripts/native-v016.test.mjs`。`MPVKIT_CORE_SOURCE` 与 `MPVKIT_FFMPEG` 沿用前文已校验资产；每个用例从新隔离根开始。

| 最终 EXE 证据（`.local/e2e-artifacts/`） | 目录 |
| --- | --- |
| WinGet 安装/升级/卸载、失败/取消/虚假成功及归属 | `mpvkit-winget-gf72OW` |
| 启动尺寸与切集窗口保持 | `mpvkit-v018-window-dOYBpV` |
| uosc 布局/模式/信息/控制台 | `mpvkit-v018-yv1VdP` |
| 连播、自然排序队列与字幕语种后缀 | `mpvkit-v018-autoplay-WSbmU5` |
| 安装/手改/profile/helper/完整重置及 52 轨道用例 | `mpvkit-v016-LIs6sN` |
| 双语/主题/路径 | `mpvkit-v016-ui-FUE1Rx` |
| 快捷键/控制台/续播/字体/缩略图 | `mpvkit-v016-player-Ox36AE` |
| 预编辑/时间戳/过期保护/响应布局/卸载重装保存 | `mpvkit-v016-draft-QR9VsF` |

原生用例开发中修正了旧“恢复上次操作/卸载保留全部配置”的测试假设，按钮短名称同步；控制台属性轮询允许异步出现，循环测试从最后媒体 EOF 前重新开始，避免把播放结束后修改循环属性视为自动重启约定。初轮失败不作为通过证据。

上一轮原生回归 7/8 通过，重装保存发现实际产品缺陷：卸载后保留的手改配置不再受安装清单管理，配置预览仍一律拒绝这些文件，不能进入差异确认。已在共享配置预览层按本次明确替换意图放行，继续保留预览指纹与执行前摘要检查；新增 Rust 回归并通过。最终 EXE 的独立重装用例先通过（40639ms，`mpvkit-v016-draft-vM9qO0`），随后完整套件 8/8 通过，证据见上表；之前的 7/8 不作为全套通过依据。

真实 WinGet 安装/卸载器、UAC、共享系统包缓存以及 Linux pacman/polkit 均未在开发者主机运行。包管理器缓存由其自身管理；自动清理只针对 MpvKit 有归属的操作文件。归属未知或已手改的文件与旧开发版实际数据不清理。保留两个最小 owner/lock 控制文件，不声称卸载后根目录必为空。已目视审阅 Windows 顶部组件区与重置/保存差异截图；完整路径、状态及操作按钮可见。

## 本机 Linux 原生验证（2026-10-26）

本次按用户要求，先 fetch 确认最新源码 HEAD / origin/main 均为 **f2b98806907c7c98594a825d47e4efbafc2df2c8**，再在本机 Linux 实测。接手时工作区干净；参考之前“开发 Linux 版本支持”聊天与 v0.1.7 记录，重新恢复固定工具和组件，不复用历史部署数据或 ELF 作为验证结果。本轮仅增加 Linux 显式集成测试及文档，产品行为与锁文件未改，未提交或推送。

### 环境与产物

- CachyOS x86_64，内核 7.3.0-5-cachyos，KDE Wayland；KWin 6.8.1-2.1、Plasma Workspace 6.8.1-1.1。
- 系统 mpv **1:0.41.0-6.1 / v0.41.0**；运行时 FFmpeg n9.0.2、libplacebo v7.360.1；GTK 3.24.52、WebKitGTK 2.54.2。
- 固定 Rust/Cargo 1.98.1、Node 24.19.0、pnpm 11.25.0。之前 /tmp 工具已不存在，本次将官方归档与对应官方 SHA-256 核验后直接提取到仓库忽略目录 .local/toolchain，没有运行远程安装脚本或安装系统软件。Cargo 按现有锁文件恢复依赖后离线测试。
- uosc 5.13.0、固定 thumbfast 和文楷 1.530 均重新从固定源取得，大小/摘要与 components.rs 一致；测试部署仍再次核验。
- `pnpm tauri build --debug --no-bundle` 成功；**src-tauri/target/debug/mpvkit，265745296 字节，SHA-256 `b9fbb9be3dd999733e50a274ab08784f886b84d6959ccc0443c44b8f3966bd1b`**。x86-64 动态链接调试 ELF，依赖系统 GTK/WebKitGTK 等库，未生成安装包，bundle.active=false。
- 系统 MPV 摘要 `390d76705f0af028775a140b4b51a61bd47fc77e8e39039354e14186ace65076`；合成 15 秒 1280×720 H.264/AAC 测试媒体摘要 `8e53d48376cd29be9d614cbc3af645fe9e6c3a1e69c9cf741591d3d1299b422c`。队列末项另由系统 /usr/bin/ffmpeg 在隔离目录生成 2 秒 PCM WAV。

### 本地检查与真实执行

| 检查 | 结果 |
| --- | --- |
| Rust 默认检查 `cargo test --locked --offline` | **49 通过 / 3 默认忽略** |
| fmt / clippy `--all-targets -- -D warnings` | 通过 |
| TypeScript / Vite / 单 ELF 构建 | 通过，869 模块 |
| 前端语言/文案测试 | **2 通过** |
| 显式 `arch_wayland_install_playback_and_uninstall` | **1 通过，32.23 秒**，包括下述扩展检查 |
| 显式 `kde_session_preferences_are_available` | **1 通过**，zh-CN / dark / blue；只改测试自身 GTK 偏好 |
| 原生管理器窗口启动 | Wayland 新隔离根下存活 10 秒，由测试关闭；未完成点击验收 |
| Linux / Windows 依赖材料 `--check` | **356/319 包、618/565 材料**，无缺失 |
| 真实数据保护 | 预先记录的 **105 个文件摘要不变**，含系统 MPV、真实 MPV 配置、应用偏好和用户 applications 文件 |

三项默认忽略分别为在线固定 HTTPS 下载、真实 KDE 会话和真实 Wayland 集成；后两项已显式执行。资产通过重新下载及摘要核验取得，但不把 curl 下载等同产品内在线下载完整验收。首轮默认测试被沙箱禁止创建本地回环监听器；在完整本地权限下重跑全套通过。Node 在沙箱中仅报告测试文件，完整本地权限下两项前端子测试均通过；没有修改测试断言来跳过这些限制。

原有集成先单独运行通过（24.25 秒），再加入本版原生风险检查。扩展测试首轮因控制台属性尚未异步创建而失败，读取 user-data 根并等待 open 状态后通过；日志确认 MPV 正常执行 commands/open。这是测试等待修正，无产品控制台回退或兼容分支。最终使用新根 `/tmp/mpvkit-v018-final-20261026/config/mpv`，与此前失败根分开。

真实集成覆盖：

1. 普通 /usr/bin/mpv 按系统 desktop entry 模式启动，只覆写测试子进程 HOME/XDG，**无 config-dir、无 MPV_HOME 覆写、无 helper**；实际加载原生根配置、gpu-next/waylandvk、uosc 和 Lua。健康/缺失组件/只读预览/过期保护、音量 +10、播放期全局保存、非空 thumbfast BGRA、退出位置保存及稳定 helper 续播通过；可选 desktop entry 经 desktop-file-validate 校验。
2. 自然队列为第1集.mp4 → 第2集.mp4 → 第10集.wav，从第2集开始。隐藏媒体、嵌套目录、封面、字幕、文本、播放列表、归档均未进入队列。保留窗口时，中间 EOF 继续至纯音频，最终暂停且进程仍在；关闭窗口时最终音频结束退出成功。连播关闭后只保留当前一项，但显式两个文件仍自动进入第二项。
3. `sub/subs/subtitle/subtitles` 的小写、首字母大写及全大写 **12 个独立目录**实际加载；同目录另加无语种、zh-Hant、chs、en.forced、en.sdh。精确核对 **17 条外部字幕**，各文件只加载一次，其他集字幕排除，forced/hearing-impaired 标记成立。原生截图中中英字幕可见；没有验证所有实际字体回退或复杂排版。
4. 四模式逐一核对 loop-file、loop-playlist 和 uosc 实际 shuffle 状态；外部 loop-file=2 显示 custom，恢复顺序播放成功。反引号打开原生控制台，输入空格不改变 pause，Esc 关闭后空格恢复暂停切换。截图取证仅用于测试，产品未恢复截图功能。
5. 实际读取 autofit=75%x75%、auto-window-resize=false，生成的 Wayland 视频输出尺寸为 1080×608；这只证明本次启动及参数读取，不等于所有屏幕/窗口边框、手动缩放和异比例切集矩阵通过。三画质 × 三 HDR 共九组参数由实际 Wayland MPV 渲染两帧通过，未据此认证物理 HDR。
6. 重置后结构化设置等于完整默认值，无历史备份或遗留 pending/staging/recovery。增强卸载删未改自有预设/helper/登记，保留 watch_later、未知 notes.txt 和系统核心；系统 MPV 版本及摘要不变。安装/更新/卸载核心预览可生成，Debug 隔离模式均拒绝执行包管理器且不创建目标根；没有运行主机 pacman/pkexec。

[机器证据](linux-playback-evidence.json)保留实际视频参数、队列、17 条轨道、uosc 状态、产物摘要及覆盖边界；绝对测试路径已改为隔离根标记。[原生渲染取证](linux-playback.png)为合成媒体和测试字幕。完整日志在本机忽略目录 .local/linux-v018/，不会上传真实媒体/用户配置。

### 复验入口

在固定工具已加入 PATH、组件资产已按 components.rs 核验后，使用全新的测试根；不预先创建 MPVKIT_TEST_ROOT。原生测试需要系统 ffmpeg，附加 WAV 在测试根内生成：

```sh
ffmpeg -v error -f lavfi -i testsrc2=size=1280x720:rate=24 \
  -f lavfi -i sine=frequency=440:sample_rate=48000 -t 15 \
  -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac \
  .local/linux-v018/synthetic.mp4
pnpm tauri build --debug --no-bundle
GDK_BACKEND=wayland MPVKIT_TEST_MODE=1 \
MPVKIT_TEST_ROOT=/tmp/mpvkit-v018-new-run \
MPVKIT_TEST_ASSETS="$PWD/.local/linux-v018/assets" \
MPVKIT_TEST_EXECUTABLE="$PWD/src-tauri/target/debug/mpvkit" \
MPVKIT_TEST_VIDEO="$PWD/.local/linux-v018/synthetic.mp4" \
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline \
  arch_wayland_install_playback_and_uninstall -- --ignored --nocapture
GDK_BACKEND=wayland cargo test --manifest-path src-tauri/Cargo.toml \
  --locked --offline kde_session_preferences_are_available -- --ignored --nocapture
```

### 未测与下一任务

真实 pacman/polkit 授权取消、依赖冲突、网络失败及安装/升级/卸载成功，需可回收的 Arch/CachyOS 环境；本轮不卸载开发者系统 MPV。GUI 两页完整点击、菜单鼠标、i/Shift+i 信息覆盖层、背景单/双击、640/900/1200 布局、DPI/辅助技术与异比例媒体窗口切集仍需 Linux 原生验收，不能由 IPC 模式检查或窗口存活代替。

驱动继续输出 `radv is not a conformant Vulkan implementation, testing use only`，未屏蔽。实体 HDR/高刷、音频设备、功耗、画质比较、长时真实多轨媒体、干净系统和完整来源许可审阅尚未完成；releaseReady=false。本轮未在 Linux 重跑 Windows 原生套件，保留此前 Windows 证据，不声称两平台全部验收完成。无 CI/CD、安装包、迁移或用户实际旧数据清理。

## 本机旧配置不兼容诊断与清理（2026-10-26）

用户反馈新测试程序没有检测到已有组件。本机原生配置根正确定位 ~/.config/mpv，系统 /usr/bin/mpv 的版本和能力检查成功；旧版归属清单所列 70 个文件均存在且摘要匹配。失败来自旧 install.json 文件项中的 originalBackup/originalSha256 字段，当前严格读取返回 invalid_state；旧 settings.json 另含已删除的截图字段并缺少 autoPlaylist，返回 settings_invalid。setup_status 在读清单时提前失败，界面因未获得报告而显示未就绪；不是系统 MPV 未安装或目录未找到。

用户随后明确授权“如果是不兼容，不要改动程序代码，帮我清理本机的旧配置即可”。本次按此授权执行一次性本机清理，不增加产品兼容、迁移或自动清理旧版分支。先生成 80 项具体删除预览，依据旧清单、收据、归属和实际摘要逐项复核，持有既有操作锁后删除旧预设、增强、helper、设置/清单及有记录的旧备份/暂存，共 579164360 字节；没有删除整个 MPV 目录。14 条 watch_later 和 owner.json/operation.lock 共 16 文件摘要保持不变，未知文件未删除，系统核心未改。当前操作的临时失败恢复副本在成功后删除，不保留历史备份。

清理后实际 Rust setup::status 返回成功：playerReady=true，generation=null、componentsReady=false、settingsReady=false，设置回到空安装默认状态。现在应显示系统 MPV 已安装，增强和配置等待安装；需要重新打开管理器刷新状态。没有安装新配置或改变默认应用。临时只读诊断代码已移除，产品代码和测试程序未修改；本机清理预览及结果日志保留在忽略目录 .local/linux-v018。此前“真实配置未改”的表述对应上一批隔离测试，本节为用户后续明确授权的例外操作记录。


## 组件操作状态与最终 Linux 测试程序（2026-10-26）

按用户新要求修正组件交互：核心未安装→安装，已安装→检查更新，有候选→更新，无候选→暂无更新。是否安装与能力健康分开，前后端均防止对已存在核心执行 install。增强缺失/损坏/固定版本变化分别显示安装/修复/更新，当前健康组件不提供重复操作；缺核心不会重新安装健康增强，连续安装并保存流程补好核心后直接进入配置确认。明细常驻更新入口删除，预览标题/确认按钮使用具体动作。独立设计预览同步核心未安装、有更新及无更新场景。

更新查询：Linux `/usr/bin/pacman -Qu mpv`（15 秒超时），只读当前同步数据库，不刷新；退出 1 且标准输出/错误均空才判为无更新，数据库错误保持失败。Windows `winget list --id shinchiro.mpv --exact --source winget --upgrade-available --disable-interactivity --accept-source-agreements`（45 秒超时），精确包候选行决定是否更新；错误不复用旧结果。Linux 安装/更新使用 --needed，避免无更新时重复安装。没有新增依赖、后台查询或自动升级。

| 验证 | 结果 |
| --- | --- |
| Rust 默认测试 | 50 通过，3 默认忽略；含更新输出/错误解析 |
| 前端测试 | 3 通过；缺失、健康、损坏、缺少单个组件、版本变化及缺核心但增强健康 |
| fmt / clippy --all-targets -- -D warnings | 通过 |
| TypeScript / Vite / 单 ELF | 通过，870 模块，bundle.active=false |
| 实际 React 页面（隔离 Tauri 模拟数据） | 已安装/未安装、无更新/有更新/检查失败、缺失/损坏/版本变化、补核心后跳过健康增强安装、弹窗动作、640/900/1200 布局通过，无页面异常 |
| 独立 Mantine 预览 | 完整 check.mjs 通过，新增各组件动作断言；无页面异常或外部请求 |
| 最终 ELF + 系统 MPV 的 KDE Wayland 隔离回归 | 1 项通过，32.57 秒；本机实际只读更新查询 false，已安装 install 被拒绝，upgrade/uninstall 在隔离模式被拦截 |

原生回归复用本节之前的命令，新根为 `/tmp/mpvkit-v018-actions-final-20261026`；仍覆盖当前版实际播放、字幕/队列/结束行为、模式/控制台、配置保护/重置及增强卸载。默认 Rust 测试首次在沙箱内因本地 HTTP 测试无法监听而失败，获准运行同一隔离套件后 50 项通过，没有跳过该用例。

最终单 ELF：`src-tauri/target/debug/mpvkit`，**265910680 字节**，SHA-256 **`0a4546a4a3933a0fb6bf64dd7562d839d36380cf566cce2d986246e993595990`**。使用固定工具链执行 `pnpm tauri build --debug --no-bundle`，包含本节按钮/查询修正。前节 `b9fbb9…` 及 linux-playback-evidence.json 为修正前当批播放证据，保留追溯，不代表当前程序摘要。

本机忽略证据：`.local/linux-v018/actions-unit.log`、`actions-clippy.log`、`actions-build.log`、`actions-wayland.log`、`actions-ui.mjs` / `actions-ui.png`，独立预览截图在 `.local/ui-v016/`。实际 React 验证使用本地 HTTP 页面及模拟 IPC，不冒称 Linux WebKit 原生点击验收；原生播放验证使用真实 Wayland 与系统 MPV。

Windows 参数解析和原生测试脚本/夹具已同步，但本机没有 Windows，未重跑其编译和原生套件；Windows 真实 WinGet/UAC、Linux 真实 pacman/polkit 仍待可回收环境。Linux 更新结论仅针对当前同步数据库，若过期应先完成系统常规更新。依据：[WinGet list](https://learn.microsoft.com/en-us/windows/package-manager/winget/list)、[WinGet 列表实现](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerCLICore/Workflows/WorkflowBase.cpp)、[pacman](https://man.archlinux.org/man/pacman.8.en)。本节只进行隔离文件操作与只读包查询，无安装包、CI、提交、推送或历史开发版兼容。


## uosc 与 4K 启动窗口调查（2026-10-26，实施前记录）

本轮按用户要求只调查与设计，不实施产品改动。完整结论、两个布局的比较、推荐尺寸、倍速列表及启动窗口修复边界见[待审方案](uosc-window-review.md)，已整理[原生调查数据](uosc-window-investigation.json)。

本机 CachyOS / KDE Wayland，系统 MPV v0.41.0；主屏 3840×2160、150%，副屏旋转后 1440×2560、150%。使用合成 1280×720 视频、无配置/脚本的临时 MPV 进程，默认 autofit=75%x75% 与 auto-window-resize=no 得到 1080×608；日志先记录竖屏尺寸，再进入 DP-1。auto-window-resize=yes 仍为 1080×608；screen-name=DP-1 得到 2880×1620，额外 hidpi-window-scale=yes 仍为同尺寸。默认窗口实际落屏后，通过 IPC 重新设定同一个 autofit，实际由 1080×608 变成 2880×1620。仅证明一次原生重算有效，不证明待设计的自动时机与用户手动调整保护已经可靠。

uosc 调查使用固定 5.13.0 ZIP、当前配置与脚本的临时副本及只读布局观察代码；合成白底加/不加字幕各取一张原生窗口图。中央三控 enabled=true、visibility=1，按钮 48×48 物理像素、字体 34，uosc scale=1.5、controls_size=32、opacity.controls=0。字幕隐藏后中央控件可见，显示时与字幕交叠；刻度条由 controls 第 14 项 kind=speed 证实。

本机忽略证据：`.local/linux-v018/window-probe.py`、`window-reapply-probe.py`、`uosc-current-probe.py`；原始输出与截图在 `/tmp/mpvkit-v018-window-investigation/`。独立布局示意的播放/切集模拟、六项倍速、Esc 和 320/480/736 宽度检查通过；示意控件使用占位图标与本地模拟，不冒称原生渲染、完整可访问性或产品验收。

未改实际配置、桌面分辨率/缩放/规则、系统核心或默认应用。没有重建测试程序，上一批 ELF 摘要仍有效。Windows 原生、多屏跨屏/手动缩放竞争、首次纯音频转视频、不同视频比例与完整字幕避让仍待获批后实施验证。无提交、推送、CI、安装包或历史开发版迁移。


### 左侧仅三控的后续设计收敛（实施前记录）

按用户进一步反馈，仅更新[设计方案](uosc-window-review.md)和独立示意：左侧仅上一集/播放暂停/下一集，右侧功能分组，取消逐键深色方块，改为统一衬底及透明按钮。对比直接分组与精简设置入口，推荐前者，宽窗保留独立信息；窄窗从更多访问收起项。独立示意已验证左侧恰为三控、六项倍速/四模式选择、暂停/切集、窄窗更多的倍速入口及 Esc；320/480/640/736 宽度无左右按钮重叠或横向溢出，两案已渲染查看，无页面脚本错误。Git diff 空白检查及本轮文档本地链接检查通过。预览检查只证明示意交互和布局，不作为 uosc 原生样式可行性或实播验收。产品代码、资源、实际配置和测试 ELF 未在本批修改；Linux 调窗建议仍待审。


## uosc 布局、倍速和 Wayland 启动尺寸实施（2026-10-26）

用户批准直接分组，并明确随手测试的底色数值不作为参考。最终选择 55% 中性衬底、24 逻辑像素渐隐、透明次要按钮、12% 主按钮轻底及最多 18% 悬停；44/48/44 逻辑尺寸左三控，右侧按轨道、播放选项和工具/窗口分组。基于原生 uosc 坐标、命中和菜单，仅在现有 bridge 增加有限样式与文字倍速按钮，两平台来源/合并摘要已更新。完整实现见[设计与实施记录](uosc-window-review.md)。

[原生控制栏取证](uosc-native.png)依次为合成白底和深色画面的底部裁剪，非概念预览；[机器记录](uosc-native-evidence.json)包含实际坐标、尺寸矩阵和本次程序摘要。原始日志与菜单/ASS 图在本机忽略目录 `.local/linux-v018/` 和 `/tmp/mpvkit-v018-uosc-visual`。

| 检查 | 结果 |
| --- | --- |
| Rust 默认测试 | 50 通过 / 3 默认忽略 |
| 前端 3 项、fmt/clippy、TypeScript/Vite | 通过，无依赖或锁文件变更 |
| Windows 原生测试脚本 | 位置/点击断言同步，语法检查通过；未运行 Windows 原生 |
| 真实 Wayland 全套隔离回归 | 1 项通过，46.91 秒，根 `/tmp/mpvkit-v018-uosc-final2-20261026` |
| 左三控、窄窗、倍速/模式 | 320/480/640/900/1200 逻辑宽度无按钮重叠，三控/更多/全屏保留；六倍速、1.1×、列表重开、已打开列表当前值更新及 Esc 通过 |
| 720p / 1080p / 4K 16:9 初始窗口 | 均 2880×1620，实际单屏 DP-1；与 MPV 正确输出参考一致 |
| 竖屏 / 超宽 / 自定义 60% | 911×1620 / 2880×1215 / 2304×1296；保持原生比例与当前 autofit |
| 纯音频转视频、全屏往返及切集 | 首次视频正确适配；手动设置尺寸后，往返及切集保持宽度 |
| 显式 geometry/window-scale、最大化/全屏 | 不执行启动重算 |
| 实际策略的无窗口行为自检 | 一次性调用/注销、当前值、取整容差、初始尺寸干预、跨多屏永久跳过、Windows/X11 跳过、音频等待视频均通过 |

自动点击曾受到 Wayland hover 与原生菜单/遮罩动画的影响。最终测试在私有 Lua 副本中驱动原生 cursor 入口，并同步 MPV 原始鼠标坐标，再经 IPC 按下/松开验证 uosc 命中与延迟分发；不改变产品输入逻辑，也不移动用户物理鼠标。切集等待实际 file-loaded，避免读取卸载间隙暂不可用的 time-pos。早期失败未作为产品通过证据；原版 uosc 和实际用户配置没有为测试修改。

扩展矩阵另捕获初始窗口同时进入 DP-1/DP-2：MPV 当时重算仍可能采用竖屏，因而增加“唯一输出”条件，歧义时停止，不在后续拖屏后补强制调窗。该新分支的无窗口行为检查通过；最终原生矩阵全部实际落于 DP-1，未用模拟假称原生跨屏跳过已稳定复现。平铺依靠目标 MPV 原生 locked_size；极短物理尺寸干预、所有混合缩放、真正的鼠标/桌面快捷键移动窗口仍未验收。字幕不抬高，长行/双语/任意定位字幕仍可能与次要入口或衬底重叠；本轮只实播查看 SRT 和一份定位 ASS，不声称全字体和字幕矩阵通过。

复验继续使用前节固定工具链、组件资产和新隔离根，执行 arch_wayland_install_playback_and_uninstall；其中已集成 src-tauri/src/linux_tests/uosc.rs / uosc-inspect.lua。无窗口策略检查从仓库根执行：

```sh
/usr/bin/mpv --no-config --load-scripts=no --vo=null --ao=null \
  --resume-playback=no --save-position-on-quit=no \
  --script=src-tauri/src/linux_tests/window-policy.lua \
  -- .local/linux-v018/synthetic.mp4
```

本次构建使用 `pnpm tauri build --debug --no-bundle`，`bundle.active=false`。最终 Linux 单 ELF：**265918880 字节**，SHA-256 **f2743d00520576c4630910c623b94e09ad7c2b12e51ca518397d536e3ce723a8**，位置 `src-tauri/target/debug/mpvkit`；替代前节组件按钮批次程序摘要。本轮未修改真实配置或系统核心，无安装包、CI、兼容迁移、提交/推送。Windows 运行环境不可用，其共享代码已同步但不冒称原生验证完成。


## 品牌色与 Linux 图标及授权清理（2026-10-26）

用户要求 Linux 图标统一，并完整删除颜色偏好，采用现有图标 #A84BD6。产品/预览共用品牌色阶；颜色 UI、序列化字段及 Windows 系统强调色读取已删除，明暗和语言保留。Rust 严格格式不接受旧字段，不增加历史配置兼容。

| 检查 | 最终结果 |
| --- | --- |
| 默认 Rust / fmt / clippy | 50 通过、3 默认忽略；格式与 all-targets -D warnings 通过 |
| 前端 / TypeScript / Vite | 通过；Windows 原生脚本语法通过 |
| 实际 React 页面，隔离 Tauri 模拟数据 | 无颜色控件或提交字段；简中/英文、浅色/深色/系统跟随、640/900/1200 布局通过，无运行异常 |
| 品牌与对比度 | 主色 #A84BD6；白字对比度约 4.524；链接使用 Mantine anchor 变量，两种主题均检查达到 4.5 |
| 独立离线预览 | 完整 check.mjs 通过，包括草稿、确认、焦点、双语主题、200% 文字、动效及组件状态 |
| Linux 桌面注册 | 含空格/中文/%f 的根路径通过现有隔离注册测试；PNG 字节与仓库素材一致；图标被改后拒绝登记；新入口通过 desktop-file-validate |
| 最终原生 Wayland 回归 | 新根 /tmp/mpvkit-v018-brand-final2-20261026，45.37 秒，播放/配置/卸载及图标移除通过 |
| 最终 ELF 窗口身份 | 新 XDG 根启动实际程序，Wayland 协议为 xdg_toplevel.set_app_id("io.github.mpvkit")，窗口正常存活 |

原生续播复验曾在 IPC 接通后立即读取 time-pos，文件尚未就绪时返回 property unavailable；测试补充 MPV 原生 expand-text 缺省值的就绪等待，原有续播位置 >=4 秒断言保留，最终新根通过。产品播放逻辑未因此改变。GTK 初始化前以 GLib 原生程序名关联桌面身份，不新增会话总线单实例注册。

用户进一步明确授权“帮我直接清理所有不兼容旧配置”。本机检查范围仅为已知 MpvKit 应用目录、当前 MPV 配置根、此前已知旧私有根和固定 desktop 文件；当前 MPV 清单 73 文件均匹配，无需删除。清理具体预览保存在本机忽略目录 .local/linux-v018/brand-cleanup-plan.json，执行时持有应用数据根与配置根的现有操作锁并重新校验：

- 仅移除 $HOME/.local/share/mpvkit/mpvkit.json 的 settings.accent，保留 language=system、theme=system 和其他结构；新文件 SHA-256 为 7e1b09b1c63882ff4d977d3a4340006901dfabe7a934018076c21f14d8ed7d0a。
- 删除 $HOME/.local/share/applications/io.github.mpvkit.desktop：内容完整匹配旧生成模板，指向 $HOME/.local/share/mpvkit/player/.mpvkit/launcher/mpvkit，目标已不存在；当前配置的可选登记关闭。删除前摘要为 81407d98ce9bc2ba2b8ade6db65089cef5a9bf0014e6f0763d8d8ed127e9cd0c。
- 仅用 rmdir 移除 10 个确认为空的 backups/staging 目录；没有清空任何有内容的历史目录。当前配置/资源、续播、结构化设置及归属等共 84 个保护文件，执行后摘要全部不变。系统核心与默认应用未变，派生 desktop 索引已刷新。

这是用户授权的一次性本机清理，脚本与完整预览仅留在忽略目录；没有把旧目录、旧字段解析或迁移逻辑加入程序。当前兼容的播放器配置未覆盖，新可选桌面入口须经现有预览/登记流程应用。

最终单 ELF：src-tauri/target/debug/mpvkit，**266392272 字节**，SHA-256 **5e5bf9b0f72fed210f7ad01fb2682e32a930d425a4c90871a0189dd23f973963**。通过 pnpm tauri build --debug --no-bundle 生成，bundle.active=false；前节 uosc 摘要属于其当批程序。[机器证据](brand-evidence.json)。本机日志位于 .local/linux-v018/brand-*.log，无新依赖、CI、安装包、自动提交或推送。

Windows 原生不可用；浏览器模拟不能替代原生 WebKit/辅助技术验收。Wayland 应用身份已取证，KDE 任务栏图标缓存和固定入口未目视确认；本轮未自动创建新的真实桌面入口或更改当前可选登记意图。已有 HDR/功耗/混合缩放等边界继续保留。

## Wayland 管理器原生图标修复（2026-10-26）

前批只验证 app_id，真实管理器缺少同名桌面入口，不能据此认定图标正常。本轮补齐独立管理器入口，在实际 KDE 会话中完成查找与截图验收。用户明确授权本机登记；其余写操作和播放测试均隔离。

| 检查 | 结果 |
| --- | --- |
| Rust 默认套件 | 52 通过，3 默认忽略；包括独立入口更新、空格/%f 转义、未知入口/链接、手改资产与过期计划保护 |
| fmt / clippy / 前端 | 全部通过；clippy --all-targets -- -D warnings；前端检查；构建含 TypeScript/Vite |
| 完整原生回归 | 新根 /tmp/mpvkit-v018-manager-regression2-20261026，45.66 秒通过；媒体注册、播放、保存、续播、uosc、窗口及卸载保留管理器资产 |
| 实际桌面入口 | desktop-file-validate 通过；固定链接指向应用数据根 desktop/io.github.mpvkit.desktop；Icon PNG 与仓库字节一致 |
| 直接执行 / 桌面启动 | 实际 ELF 和 gio launch 均正常显示管理界面，无媒体缺失错误；KWin resourceClass、desktopFileName 均为 io.github.mpvkit |
| 原生图标 | 标题栏和 KDE 任务栏均确认显示品牌紫色图标；不是前端模拟或仅协议断言 |
| 实际数据保护 | 87 个 MPV 配置/应用偏好/默认应用相关路径快照不变；媒体登记意图未更改；仅结束自行创建的窗口 |

![管理器原生标题栏](linux-manager-titlebar.png)

![KDE 任务栏中的管理器](linux-manager-taskbar.png)

本机隔离复验入口：

```bash
source .local/linux-v018/env.sh
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings
pnpm test:frontend
pnpm tauri build --debug --no-bundle
# MPVKIT_TEST_ROOT 必须换成尚不存在的新绝对目录。
GDK_BACKEND=wayland MPVKIT_TEST_MODE=1 \
MPVKIT_TEST_ROOT=/tmp/mpvkit-manager-fresh-test \
MPVKIT_TEST_ASSETS="$PWD/.local/linux-v018/assets" \
MPVKIT_TEST_EXECUTABLE="$PWD/src-tauri/target/debug/mpvkit" \
MPVKIT_TEST_VIDEO="$PWD/.local/linux-v018/synthetic.mp4" \
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline \
arch_wayland_install_playback_and_uninstall -- --ignored --nocapture
```

真实 KDE 验证须让桌面会话能搜索到实际用户级入口，不能用子进程私有 XDG 目录冒充。已生成 `$XDG_DATA_HOME/applications/io.github.mpvkit.desktop` 及应用数据根的两个自有资产和归属清单。重复启动正常，不改 MIME 默认值。KWin 临时取证脚本执行后卸载；标题栏/任务栏截图只保留目标局部。最初两次活动窗口截图抓到了其他窗口，未作为通过证据；最终按本轮 PID 定位并核对 active=true 后截图。实际原生截图/窗口信息来自 /tmp/mpvkit-manager-icon-native4-20261026，[机器记录](manager-icon-evidence.json) 已摘要化。

原生套件首次在 src/linux_tests/uosc.rs 的暂停按钮即时断言失败（pause 仍为 true），第二个全新隔离根完整通过；未修改产品 uosc 或放宽断言。此处时序波动仍存在。Windows 原生与已有固定面板入口未测；无包管理器写操作或默认应用变更。

最终单 ELF：`src-tauri/target/debug/mpvkit`，**266315352 字节**，SHA-256 **c58bfd0e4c199cec83fa83761653bcdcf5e39bd9ba7ee91c4b474e39600d6cd5**；替代前批程序。bundle.active=false，无新依赖、安装包、CI、历史迁移、提交或推送。本机日志在 .local/linux-v018/manager-*.log。

## 接手材料与无用文件清理（2026-10-26）

基于已推送的 7fa64a2，用户要求更新文档并适度精简，方便后续 agent 测试/开发。本轮只整理接手材料、删除无调用的旧入口并搬移已有预览样式，不改变产品业务、Rust/Lua、依赖版本或实际用户配置。

- AGENTS 合并重复/被覆盖的历史决定，保留当前硬性规则；current 统一产品/目录/模块导航；plan 从逐批长记录改为完成表、最新验证、下一任务。原始批次证据、来源/许可、待验收项保留。补 docs/testing.md 作为 Linux/Windows/预览/来源工具的统一索引；历史方案顶部明确适用范围，旧链接更新。
- 删除 8 个文件：NSIS hooks、NSIS 目录选择测试、未引用的历史 logo、旧三页预览的 JSX/CSS/HTML/build/check。Tauri bundle 删除 targets/windows 安装器专属配置，active=false 保留；图标、许可资源映射保留。旧预览 README 保留历史跳转，旧测试命令标注不可执行。
- 当前预览仍使用 docs/v0.1.6/preview；其依赖的旧基础 CSS 原样搬入该目录后再删除旧文件。构建前/后生成 CSS SHA-256 完全一致：242f419011243adcc718c0f752bdc059869ea028ee4caf6041f58fee804c51a1。当前预览完整交互检查通过，无运行时异常或网络请求。

本轮验证：Rust 默认 **52 通过 / 3 忽略**（3.44 秒），fmt、clippy all-targets -D warnings、前端脚本、TypeScript/Vite、Windows 原生脚本语法及单 ELF --no-bundle 构建通过。两平台依赖材料只读 --check 通过，Windows 319 包/565 材料、Linux 356 包/618 材料，均无缺失；首次普通沙箱调用 cargo 子进程受限，获得运行权限后成功，未为通过而改脚本或材料。当前文档本地链接/锚点与 Git diff 检查通过，删除文件无现役调用残留。

本轮 ELF 与前一功能批字节相同：**266315352 字节**，SHA-256 **c58bfd0e4c199cec83fa83761653bcdcf5e39bd9ba7ee91c4b474e39600d6cd5**。当前运行窗口、播放、事务代码未变，因此未重复完整 Wayland/Windows GUI 或真实包管理器/设备矩阵；已有功能实测与限制继续有效。不会把本轮文档/预览检查当作新增设备验收。

日志在 .local/linux-v018/cleanup-*.log。保留本机工具链、研究资产、测试程序与历史 evidence；没有清理 .local/实际数据或启用 CI/安装包。用户随后明确要求提交并推送本批整理到 origin/main；下一任务仍按 [当前交接](plan.md#下一任务与未测边界)选择最小可验证项。

## Windows 当前批实机回归（2026-10-27）

用户在拉取更新后要求检查进度并完成 Windows 实机测试。本轮从干净工作区 **a24ff6c** 开始，覆盖此前 Linux 功能批对应的当前 Windows 程序。测试跨 10 月 26 日晚至 27 日，记录使用 Asia/Hong_Kong 日期。全程只构建单 EXE，bundle.active=false；没有 CI、安装包、历史开发版兼容、真实用户数据清理、提交或推送。

### 环境、资产与隔离

- Windows x64，系统 API 报告 **10.0.26200.0**；实测管理器窗口 DPI **192（200%）**，DWM backdrop **2**、查询结果 0。屏幕物理范围为 **3072×1920**。这只是当前一块屏幕，不扩展为混合 DPI/多屏验收。
- Rust/Cargo 1.98.1，pnpm 11.25.0；Node 实际 v24.22.1，与声明 24.19.0 的既有 engine 提示保留，未升级工具链要求或依赖。构建保留缓存 hard-link 回退和 MSVC 导入库提示。
- 复核已提取 WinGet 核心 mpv.exe：**120308736 字节**，SHA-256 **1645e57569c0af58a70c61257f3716f2ee6eb31bb170edc51f3f25a0b8d0d99c**，实际 **v0.41.0-244-gaf9c81fa1**，libplacebo 7.360.0。合成媒体 ffmpeg SHA-256 **7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1**。两者与前述渠道核验证据一致，没有重新下载或执行安装器。
- Node 套件每个用例新建私有临时根；显式 MPVKIT_TEST_ASSETS、MPVKIT_CORE_SOURCE、MPVKIT_FFMPEG。Debug 核心/WinGet 来自该根，WinGet 为本地编译夹具；启动后先通过 setup_status 核对隔离配置根再允许写入。登记使用私有文件后端；额外原生注册表测试使用 Offreg 私有文件 hive，不改真实 HKCU。桌面补查复用本轮已完成安装的隔离根，integrationEnabled=false。

### 检查结果

| 检查 | 结果 |
| --- | --- |
| pnpm tauri build --debug --no-bundle（含 TypeScript/Vite） | 通过，871 模块；收尾重建也通过 |
| cargo test --locked | **63 通过 / 6 默认忽略，8.39 秒** |
| fmt / clippy all-targets -D warnings | 通过；修正契约模块后专项 2/2 通过 |
| 私有 hive 原生登记/外改/失败恢复 | **1/1 通过，0.17 秒**；显式只运行该 ignored 用例 |
| pnpm test:frontend | **3/3 通过** |
| Windows 完整原生回归 | **8/8 通过，292549ms**；另含 **52 个真实 MPV 轨道子用例** |
| 浅/深色固定品牌与当前偏好字段专项 | **1/1 通过**；最终重建 EXE 另有侧栏冒烟记录 |
| Windows 依赖材料 --check | **319 包 / 565 材料**，无缺失 |
| 桌面管理器目视/输入 | 英文播放器页、英文/中文应用页、深色/浅色、紫色强调色、标题栏品牌图标通过；语言/明暗使用桌面原生点击及按键 |

完整套件运行 `node --test --test-reporter=tap --test-concurrency=1 scripts/native-v016.test.mjs`；原始日志在 **.local/windows-v018-20261026/**：build.log、build-final.log、unit.log、clippy-final.log、private-hive.log、contract-final.log、native.log（首轮）、native-final.log（通过）、brand-smoke.log、final-exe-smoke.log、notices.log、window-metadata.json。前端结果保留在本轮工具输出。机器摘要见 [windows-native-evidence.json](windows-native-evidence.json)。

| 最终完整回归证据（.local/e2e-artifacts/） | 目录 |
| --- | --- |
| 核心缺失/失败/取消/重试、更新检查与升级、卸载归属 | mpvkit-winget-Cn3Twp |
| 五种视频尺寸、4K 切集、手调及全屏 | mpvkit-v018-window-7KviWk |
| uosc 布局、四模式、倍速列表、stats、右键控制台 | mpvkit-v018-nwxHmJ |
| 自然连播、队列结束、字幕目录/语言后缀 | mpvkit-v018-autoplay-6gGf6D |
| 安装、手改保护、原生 profile/helper、52 轨道子用例 | mpvkit-v016-1Q0BN0 |
| 双语、主题、路径、键盘与焦点 | mpvkit-v016-ui-Ju5XSq |
| 预编辑、mtime、过期预览、布局与卸载保留 | mpvkit-v016-draft-ZehsIA |
| 新增品牌断言专项 | mpvkit-v016-ui-r3KcJf |

### 本轮修正及真实覆盖

1. Windows clippy 首次发现 linux_core_contract 中 check_update 未使用。沿用 prepare/execute 的现有类型引用检查补 **一行 check_update 引用**，继续在 Windows 编译 Linux 实现而不执行 pacman；未屏蔽警告或改变产品平台分支。
2. 首轮普通沙箱 Rust 为 33 通过/30 失败，错误集中于 Windows 文件替换权限及本地回环连接（10013）；普通沙箱 WebView 调试端口也未就绪。保留失败日志，获准使用隔离进程后 Rust 63 项通过，未修改事务/路径保护来绕过限制。
3. 首轮原生 **7/8（281049ms）**，uosc 在 playlist 循环回首集后，playlist-pos 已为 0 但新媒体尚未就绪，下一条 seek 返回 error running command。日志显示随后才 Starting playback。与既有主动切集等待保持一致，增加 **time-pos 为 number** 的就绪条件；没有放宽模式/seek/暂停断言，也未修改产品 Lua。新根完整 8/8 通过。此前 Linux 暂停即时断言的时序记录仍保留。
4. 侧栏专项增加原生 WebView 浅/深色 --mantine-color-brand-6 都等于 **#A84BD6**，并检查 read_app_settings 只有 schemaVersion/language/theme 三个当前字段。桌面补查也得到同样结果，没有旧字段迁移逻辑。
5. 已在实际 MPV 核对左三控、640/900/1200 宽度收起、六项倍速及当前项、四种模式和外部状态、目录/随机/列表边界、队列 EOF、stats 与 Shift+i、右键控制台、单击忽略/双击全屏不暂停、缩放重排、配置手改与重置、上游前缀/桥接摘要。实播使用 IPC 命令与原生渲染；已目视审阅 layout-1200.png、layout-640.png 和管理器桌面，不能等同物理鼠标输入。
6. 窗口专项用 640×360、1920×1080、3840×2160、竖屏和超宽合成视频验证 75% 范围与比例、切集尺寸稳定、手动窗口调整后切集及全屏返回；文件根和所有截图均为本轮测试资产。实际轨道、续播、字体、缩略图、手改/恢复/过期保护由完整套件覆盖。GUI 640×600、900×700、1200×800 与 200% 文本断言通过。

完整回归使用 EXE **30689280 字节**，SHA-256 **08b48e35cc2a5f8c7b9deeaaab4edc8d8ded6753e34ce37cf58ef2c6413c1248**。补测试专用类型引用后收尾重建的单 EXE 仍为 **30689280 字节**，SHA-256 **60d6b0a509de5abe0378251b513b7da0b5bdf7ed07a69acab65d4a9bd2ac13c5**；最终文件仅补原生侧栏冒烟，不把两份 EXE 摘要混为同一份完整回归产物。路径均为 src-tauri/target/debug/mpvkit.exe，没有安装包。

### 保留的未测边界

无可回收 Windows 虚拟机，本轮没有运行真实 WinGet install/upgrade/uninstall、UAC、包自身登记/依赖/取消或系统缓存操作；真实 WinGet 更新候选/源失败仍需虚拟机复验，现有检查为进程夹具。没有改变默认应用或真实配置/注册表；未额外制作真实用户数据前后全量快照，隔离结论依据启动路径复核与测试后端。

Computer Use 管理器首次应用授权等待超时，重试后成功；管理器两页原生树可读取，但未运行屏幕阅读器。MPV 桌面窗口被工具映射为已登记的 mpv-lazy 路径，实际 PID/映像核对为隔离 MPV；刷新绑定仍报 no longer belongs（错误两侧显示相同路径），按工具要求停止该窗口输入。因此未完成物理鼠标双击/控制台输入验收，未以 IPC 结果替代；额外测试播放器经自己的 IPC quit 退出，管理器正常 Alt+F4 退出，窗口列表确认本轮目标消失。

HDR/高刷/功耗、长期真实媒体、混合 DPI/多屏、完整辅助技术、高对比及系统减少动态效果映射、干净系统和完整来源许可审阅仍按既有台账推进。当前无新的已复现产品故障，releaseReady=false；本轮不重跑已完成的 Linux 实机检查。

## Windows 验证收尾与清理（2026-10-27）

用户已明确授权更新文档、清理无用文件/代码并提交推送。同步 README、文档入口、AGENTS、当前基线和交接，补充本机 Windows 结果及未测边界；原始批次证据与来源/许可材料继续保留。

- 全仓检查确认 component_catalog 无调用方，删除该 IPC 及 handler 登记；组件目录本体仍供部署、来源和测试使用。
- 原生测试删除未执行的 managed 启动分支和重复按钮匹配；将启动参数不含 --include/--scale 的断言移至实际托管启动路径，保留有效覆盖。
- 仅删除本轮生成的 .local/windows-v018-20261026/manual.mjs 与 manual-state.json；删除前核对绝对路径在工作区内。保留测试截图、原始日志、研究资产、工具链和实际用户数据。

| 清理后检查 | 结果 |
| --- | --- |
| cargo test --locked | **63 通过 / 6 默认忽略，8.01 秒** |
| fmt / clippy all-targets -D warnings | 通过 |
| pnpm test:frontend | **3/3 通过** |
| 单 EXE 构建（含 TypeScript/Vite） | pnpm tauri build --debug --no-bundle 通过，bundle.active=false |
| 受影响的安装/手改/profile/helper/托管启动及侧栏原生专项 | **2/2 通过，56006.6419ms**；另含 **52 个真实 MPV 轨道子用例** |
| 原生脚本语法 / Git diff / 文档本地链接与锚点 | 通过 |

日志位于 .local/windows-v018-20261026/cleanup-{unit,clippy,build,native}.log。原生证据分别为 .local/e2e-artifacts/mpvkit-v016-5kHmi5 和 mpvkit-v016-ui-5kAdY4。当前单 EXE：src-tauri/target/debug/mpvkit.exe，**30683136 字节**，SHA-256 **baa00658a50ebb43a655ee4224bf076bbf372b1b9eb0737487aeb35cf8bb1fc9**；机器摘要见 [windows-native-evidence.json](windows-native-evidence.json) 的 postCleanup。

上节完整 8/8 是清理前的回归结果；本轮清理后仅重跑受影响原生专项，不把旧产物的完整结果套用到新摘要。未重复 Linux 实机和无关设备矩阵；真实 WinGet/UAC、物理鼠标、HDR/功耗等未测边界继续保留。无新增依赖、CI、安装包或开发版兼容逻辑。
