# v0.1.8 验证记录

2026-10-26。用户批准经典 uosc 布局方案及开发测试；实施前 fetch，HEAD 与 origin/main 均为 v0.1.7 `e41cfda94081cf34508fe0c4f66c9f9e4188ac5d`。本轮仅本地实现、隔离测试与单 EXE 构建，没有提交、推送、CI、安装包或历史开发版迁移。

以下至“WinGet 核心与控制台”章节前为早先自下载核心批次，保留作追溯；当前代码、产物及验证以末尾新批次为准。

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
