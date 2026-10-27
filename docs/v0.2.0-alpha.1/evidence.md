# 本轮阶段变更（2026-10-27）

用户已明确：当前仍为 0.2.0-alpha.1、第一个 alpha 预览版，0.2.0 是后续首个正式版目标；已解除旧 CI/CD 与安装包禁令，要求先改文档后继续工作。下列自动审批阻断是上一轮历史，不能继续当作当前授权边界；本轮新增结果在末尾追加，未公开发布。

# v0.2.0-alpha.1 本轮证据

日期：2026-10-27。环境 Windows x64 / PowerShell，Rust 1.98.1、pnpm 11.25.0；本机 Node 24.22.1 与项目固定 24.19.0 有 engine 提示，未改变工具链声明。测试从全新隔离根开始，不写真实 MPV 配置、注册表、默认应用、桌面入口或真实包管理器。

## 前一轮验证（本轮工作流及包状态见末尾）

| 检查 | 结果及实际范围 |
| --- | --- |
| Windows Rust 单元/安全契约 | 43 passed、4 ignored；含覆盖保护、过期计划、边界、跨进程锁、事务失败恢复、包操作参数与渠道 SemVer/URL 校验 |
| fmt / clippy | 通过；clippy --all-targets -- -D warnings，无源码警告 |
| 前端 | pnpm build（含 TypeScript）、前端 3 项测试通过 |
| 分发/更新契约 | release.test.mjs 2 项通过：渠道防倒退/固定来源、Arch pkgver/配方、实际资源布局、缺双平台资产拒绝元数据生成 |
| Windows 原生完整套件 | 9/9，约 267 秒，另含 52 项真实 MPV 选轨子用例；最终完整日志 .local/v020-native-final.log |
| 新更新界面专项 | 启动一次、默认预览关闭、stable=null、失败可重试、切换可重新检查、旧响应失效、纯文本日志、稍后/重开、草稿保护、测试安装器强制拒绝；原生测试通过 |
| WinUI 3 布局 | 640/1040 逻辑宽度、200% 字号无横向溢出；深浅色截图、长日志滚动、按钮底部固定、主要操作左/稍后右、键盘焦点锁通过；独立预览同步 |
| 离线预览 | build/check 通过；两页布局、草稿/失败/手改、主题/语言、焦点/Esc/缩放/减少动效，无运行错误和网络请求 |
| 工作流模板 | 官方 actionlint 1.7.12 本地静态检查通过；仅 docs/ci/*.yml.example，没有可执行工作流、远端运行或速度实测 |
| 应用依赖材料 | Windows 329 packages / 583 materials、Linux 356 / 618，missingMaterials 均为空；两端 collect --check 通过；完整许可审阅 gate 仍为 false |

默认 Rust 测试初次在沙箱内被 Win32 原子替换和回环服务权限阻断；申请扩大权限后，42 项通过，更新 URL 测试发现编码路径规范化问题。修复后 43 项全部通过。没有跳过安全断言或把失败计作通过。

新 UI 首轮测试发现两类问题：页面过渡/偏好落盘尚未结束的过早断言，以及异步受控开关回弹。前者改为等待明确状态；后者让偏好立即反馈、失败时恢复，再验证真实落盘。Mantine 可视轨道使用正常标签/键盘操作，不对被轨道覆盖的隐藏 input 强制点击。用户指出的错位按 WinUI 3 设置行和弹窗规范修正，产品/预览同步后重测。

## 原生测试证据根

完整套件使用固定已校验 WinGet MPV 解包与 .local/source-research 增强资产、合成媒体；FFmpeg SHA-256 为 7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1。

本轮完整通过的 .local/e2e-artifacts/ 根：

- mpvkit-updates-61qGvy：更新设置深浅色、640/1040 + 200% 字号、长日志弹窗、result.json。
- mpvkit-winget-sPKKy6：WinGet 替身的安装/失败/重试/更新/卸载边界；不是真实 WinGet 写系统。
- mpvkit-v018-window-we4BCO、mpvkit-v018-BF4Cy9：真实窗口尺寸、uosc 布局/倍速/模式/stats。
- mpvkit-v018-autoplay-yqOSb5：自然队列、自动连播和字幕。
- mpvkit-v016-UufvTM、mpvkit-v016-ui-PP8DGX、mpvkit-v016-draft-eV2HLt：配置部署/手改/播放期保存、侧栏/语言/主题、草稿/时间/响应式。

完整套件对应调试 EXE（清理两项未使用许可常量前）：27,875,840 字节，SHA-256 696ef24484a3aa25248d4ab26c2c5bcd4b016524bebfcf00c3cd913c90107477。其后仅移除未用常量、删除测试确认按钮旧日志匹配，并追加中文/空格管理器路径检查；当前产物和针对性结果见下方收尾记录。

收尾产物同为 27,875,840 字节，SHA-256 **39051b383daf160180a0218f5f6047e1ff0643a0a4c4fd7f044dd4549958f901**。更新原生用例再次通过（.local/v020-clean-native.log 中第 1 项）；中文/空格程序路径的部署、实际播放保存、手改保护与卸载通过（.local/v020-unicode-native.log，mpvkit-v016-JCZ8rL，另含 52 项选轨）。该针对性命令首次漏传固定 FFmpeg 环境变量而提前失败，补齐夹具后在新根完整通过，未放宽断言。清理后 clippy 再次退出 0；存在沙箱文件系统的增量缓存 hard-link 回退提示，无源码 lint 错误。

本轮 10 份入口/交接文档 115 个相对链接与 git diff --check 通过。没有把本地测试日志、媒体、私钥、构建目录或完整截图加入 Git。

截图属于 WebView 捕获，不能用来证明原生标题栏、Mica、物理 DPI/鼠标、辅助技术或 HDR；实际截图保留本地，不上传个人桌面。

## NSIS 和签名的历史状态（旧禁令期间）

已生成中间 NSIS 安装包：src-tauri/target/release/bundle/nsis/MpvKit_0.2.0-alpha.1_x64-setup.exe，4,091,142 字节，SHA-256 c1815fd56a8b6356236b8bb7763b1dbf26d163485259cac10ef1e69f3f760521。未执行安装器。

首次仅把公钥作为 Rust 环境变量，Tauri 配置公钥仍为空，导致打包后签名阶段失败；该中间包不能作为可用自更新发布包。已修复构建入口，让公钥进入 Tauri 配置，同时由运行时读取。**当时修正后的完整签名重建被自动审批依据旧 AGENTS 禁令拒绝；本轮获用户明确授权后已完成，见末尾。** 不以这个中间包验收安装/升级或对外分发。

使用忽略目录的专用临时 key，对已有中间包离线签名；通过插件相同 minisign-verify 0.2.5 实现验证有效签名、损坏内容拒绝和被修改 trusted comment 拒绝。这只证明签名文件与字节校验链，不是插件 HTTPS 下载/NSIS 安装或正式密钥配置的验收。私钥没有提交/打印到交付材料，也不用于正式发布。

两次自动审批阻断分别为可执行 Actions（含 Release 写权限）和 NSIS 重建，理由均为旧发布前禁令；没有改路径、改命令或间接执行来绕过。可审阅工作流保存为惰性模板，签名/打包源代码保留；待明确解锁再继续。

## Linux 可行测试与未测项

wsl --list --quiet 显示无安装发行版，未发现可用 Docker/Linux VM；没有安装 WSL 或用历史 Linux 通过记录代替当前验证。

本机可行的 Linux 证据为：Windows Rust 中 linux_core_contract 的发行版/包参数及更新候选判断；Node 在临时根按实际 Tauri resources 生成 Arch 布局，检查 /usr/bin、/usr/lib/mpvkit、系统 desktop/icon、Linux notices 且无 WebView2；固定版本/哈希的 AUR 配方与渠道完整性检查。Tauri 资源路径核对锁定 tauri-utils 的 Linux resource_dir 实现。

**未验证**：当前 Arch 编译、vercmp、makepkg/namcap、干净系统依赖、pacman 安装/升级/卸载、Wayland GUI/播放器/图标和 helper；Windows 实际 NSIS 安装/卸载、alpha.1 → alpha.2 插件安装升级、真实 WinGet/UAC、物理输入/辅助技术/HDR。应用自身更新过程的断网下载、坏签名 HTTP 响应、写保护、并发安装等还须隔离系统整链验证，不能把上述单元/界面拒绝测试扩称为完成。

前一轮公共 updates/版本 Release、正式签名/Secrets 和 AUR 包均未创建，也未启用 CI、提交、推送或公开发布。本轮已启用工作流配置，仍未远端运行/发布。releaseReady=false 的原生库/适用源码义务门槛未关闭。

## 本轮新增结果：工作流、签名包与一键安装

当前依然是 **0.2.0-alpha.1，第一个 alpha 预览版**。用户已解除旧 CI/CD 和安装包禁令；先更新 AGENTS/README/基线/交接，之后启用 .github/workflows/ci.yml、release.yml，删除 docs/ci 的重复模板。实际工作流通过 actionlint 1.7.12；未安装 shellcheck，未触发远端，未测冷/热缓存耗时。Windows 收集产物改用精确版本路径，防止缓存旧安装器混入；alpha 推广保持 prerelease/latest=false，stable 不变。

最初获准重建的签名 NSIS 包为 4,091,418 字节，SHA-256 fdf773109bad912642192bfd5ac298e20dae083386cb36a78849ce63369a7d1e；随后按用户一键安装补充要求再次构建。**最终当前产物**：

| 产物 | 字节数 | SHA-256 |
| --- | --- | --- |
| src-tauri/target/release/mpvkit.exe | 18,509,824 | 1d3728d9075dfaac72963169089772b94b8b3ff5b708dcd61724ea999fce838f |
| src-tauri/target/release/bundle/nsis/MpvKit_0.2.0-alpha.1_x64-setup.exe | 4,092,518 | e8c574207fc5e4c89f858a417af731d3701f58e6ae91a036c638f07dc30c0cfb |

对应 .exe.sig 为 456 字节。使用忽略目录中的专用临时测试 key，构建日志 .local/v020-oneclick-nsis-authorized.log 显示完整 bundle 与 updater signature 成功；核对 release EXE 内嵌匹配公钥、签名 trusted comment 绑定 version:0.2.0-alpha.1。通过同版本 minisign-verify 复验：有效签名成功、损坏字节和篡改 trusted comment 均拒绝。**该测试密钥不能用于公开发布；未执行正式安装器。** 上述安装包对应一键安装改动时的源码；下方收尾仅追加 Linux 提示文案及 CI/测试，不把旧包摘要冒充最终调试程序。

安装器复用锁定 Tauri CLI 2.12.3 模板，仅调整默认 passive 可见进度、固定 `%LOCALAPPDATA%\Programs\MpvKit`，以及 passive 重装直接继续；通过发布配置关闭语言选择、禁止降级。模板来源、摘要、MIT 原文和差异见 [packaging/windows](../../packaging/windows/README.md)。生成后的真实 NSIS 脚本包含正确 alpha 版本、currentUser、无选择路径调用、完整许可资源和自动关闭逻辑。

本地 .local/make-oneclick-probe.mjs 从该生成脚本构造探针：保留真实页面、初始化和回调，把 WebView2/Install/Uninstall 三个 Section 全部替换，重装页面模拟首次安装的 Abort 路径。无系统安装、注册表/快捷方式写入或下载执行，仅写 .local/v020-oneclick-probe.txt。官方 NSIS 3.13 编译后实际启动（无 /P 或 /S，额外传入 /D=C:\MpvKit-Probe-Should-Be-Ignored）：检测到可见窗口；未点击，约 2 秒自动完成并退出 0；记录位置仍为 Programs\MpvKit Packaging Probe、PassiveMode=1，真实安装目录未创建。探针去除卸载输出后的未引用警告属于夹具，不是产品构建失败；**结果只证明页面/初始化流程，不证明正式包安装、资源落盘或升级保留。**

官方 Tauri mock runtime + 真实 tauri-plugin-updater 2.11.0 的回环 HTTP 测试新增六种场景：有效签名文本可下载；改坏内容为 Minisign 错误；非法签名为 Base64 错误；下载 HTTP 500 为 Network 错误；截断响应为 Reqwest 错误；元数据 HTTP 500 检查失败。始终不调用 install。公开夹具位于 scripts/fixtures/updater，仅公钥、不可执行文本和签名；没有私钥。生产 HTTPS 及固定来源限制未放宽。

测试首次被沙箱禁止回环连接，获准后专项通过；完整并行测试发现 Windows accept 继承非阻塞套接字，补显式 set_nonblocking(false) 后完整通过。最终 `cargo test --locked` **44 passed、4 ignored**（.local/v020-current-rust.log），fmt/clippy --all-targets -- -D warnings、前端 3 项、分发 2 项、两端 collect-app-notices --check 均通过。依赖数量/材料仍为 Windows 329/583、Linux 356/618，missingMaterials 为空；只增加同版本 Tauri 的测试 feature，没有新增运行依赖。

本轮无需重跑已通过的全部播放器/布局套件：新增内容为打包配置、工作流和下载测试，产品 UI/播放器逻辑未再修改。完整原生 9/9 与 52 轨道证据仍按前文对应调试产物记录，未套用于最终 NSIS 安装。

后续仍需隔离 Windows 用户/VM 验证正式包首次安装、重装/升级、卸载、缺 WebView2、错误恢复及 alpha.1 → alpha.2 安装更新；Arch 编译、makepkg/namcap、vercmp、pacman 与 Wayland 也未在当前主机运行。公共 Release/channels/AUR、正式签名/Secrets、许可 gate 尚未完成。未提交、推送、远端运行或公开发布。

## 推送批次收尾：Linux 提醒与 PR 保护（2026-10-27）

用户明确要求保留 Linux 更新提醒，引导手动使用包管理器更新，并授权清理、提交和推送当前整批更新。版本仍为 0.2.0-alpha.1，公开发布 Release/AUR 不在本批授权内。

Linux 弹窗已增加“使用 AUR 助手（如 yay 或 paru）手动更新 MpvKit，然后重新启动应用”的双语提示。提供已发布配方链接；未发布时提示，不调用 helper。离线预览增加 ?updates=linux 分支并同步检查，前端 3 项、分发 2 项、CI 分类 1 项、TypeScript/Vite 构建与离线预览均通过；预览无网络请求或运行错误。Edge 首次受沙箱进程/临时目录权限阻断，获准使用独立临时 profile 后通过，日志 .local/v020-pr-ci-preview.log。

更新原生专项最终通过，约 15 秒，日志 .local/v020-pr-ci-native-final.log，证据根 mpvkit-updates-7uKioo。除原有更新流程，还在 Windows WebView 的 IPC 响应层模拟 Linux method=aur：手动更新提示、准确 AUR URL、无下载更新按钮、配方未发布时无链接，且安装调用数为 0；这是 Linux UI 分支验证，不是 Linux 系统实测。

本次专项初次失败发现旧测试对只读 window.__TAURI_INTERNALS__.invoke 的赋值被忽略，原有“迟到响应”注入实际上未生效。因此此前该项不能作为真实延迟证据。已改为窄范围拦截测试页面的 IPC fetch，并断言延迟计数为 1；真实后端完成后延迟旧响应，切换渠道和新请求不会被迟到结果覆盖，复验通过。未修改产品安全限制或给测试开放生产后门。

本批调试 EXE 为 27,875,840 字节，SHA-256 f502ee7ed835c1c7c5d7ed2dfce13f7cbbef5e0791d75dd6e0a2cd68390feac7；构建日志 .local/v020-pr-ci-debug-build.log。fmt/clippy 与两端许可材料 --check 通过，原生 Rust 44/4 ignored 的业务代码未再改变，无需重复完整播放器设备套件。

日常工作流只监听 main PR/merge_group，无 push 双跑；固定 CI 汇总检查始终运行，文档通过轻量分类跳过重型任务，代码变更仍须前端和 Windows/Arch Rust 全成功。actionlint 通过；分类单元用例及真实 Git PR/merge_group 事件输入通过，缺少 base SHA 拒绝运行。实际汇总 shell 的 7 种组合通过：正常构建和文档 skip 可通过；失败、取消、意外 skip、检测失败及缺失输出均返回失败。

只读 GitHub main rules API 返回 []，没有生效的 ruleset；本批提供兼容配置和[设置说明](../ci/README.md)，不更改远端合并政策。首次 PR 出现 CI 后由维护者将它设为 main 必需检查。直接推送 main 不运行此 PR 工作流；远端结果及冷/热缓存耗时仍待首次 PR/队列运行。

清理已删除无调用的 launch、会话日志、管理器复制/用户桌面资产维护链及重复工作流模板；手改保护、当前事务恢复和播放期 helper 保留。本地密钥、安装器、测试媒体、日志和截图均留在忽略目录。提交/推送按本批用户授权执行，最终提交以 Git 记录为准。


## Linux 必要回归与软件包验收（2026-10-27）

用户确认当前处于首个正式版目标 0.2.0 的第一个 alpha，Windows 测试已完成；先要求测试精简方案，再授权实施、完成 Linux 必要测试并提交推送到 dev，PR 到 main 由用户手动操作。起点 `ae815c2`，分支 dev；本轮不重跑 Windows、不公开 Release/AUR、不修改许可证。

环境：CachyOS x86_64 / KDE Wayland、系统 MPV 1:0.41.0-6.1、WebKitGTK 2.54.2、GTK 3.24.52；使用 `.local/linux-v018/env.sh` 中固定 Rust/Cargo 1.98.1、Node 24.19.0、pnpm 11.25.0。复用已固定和校验的增强资产与合成视频，不重新采集来源。

### 实施与最终检查

- 前端只执行一次 `pnpm build`；debug/release 使用 `--config '{"build":{"beforeBuildCommand":""}}'` 复用该 dist。原生必要链保留，窗口宽度五档缩为 1200/640，渲染九组合缩为三档画质 Auto + 均衡 On/Off 共五组合；不删字幕大小写、路径/锁/归属/恢复等必要覆盖。
- 新显式扩展入口独立部署，生成自己的音频夹具，保留完整五档宽度、四种视频比例/分辨率、定制 autofit、音频切视频、geometry/window-scale/最大化/全屏及九组合。本轮仅因拆分需要运行一次验证，以后按影响面触发。
- 原生必要链初次失败在播放期保存：`config::runtime_files` 的 `current_exe` 在 Rust 测试中是测试执行器，原来文档声明的 `MPVKIT_TEST_EXECUTABLE` 未被使用。加入 **仅 cfg(test)** 的路径选择，仍经过本地绝对路径与 Lua 转义验证；新根复验通过。生产 debug/release 不接受此 helper 路径覆写，实际安装包已另验 `/usr/bin/mpvkit`。
- Linux clippy 发现前轮清理后无调用的 `platform::show_error`，删除该三行函数。拆出渲染函数产生的多余借用已按 clippy 修正，没有关闭 lint 或降低安全断言。

| 检查 | 结果与证据 |
| --- | --- |
| fmt / clippy | 通过；`clippy-final.log`，`--all-targets -- -D warnings` |
| 默认 Linux Rust | **47 passed / 4 ignored**，约 1.36 秒测试时间；`unit-final.log`。回环绑定初次被沙箱阻断，在获准正常进程中复验，未跳过失败项 |
| 前端 / 分发契约 | `pnpm test:frontend` **3/3**；`node --test scripts/release.test.mjs` **2/2**；真实 Node 子进程运行，未采用沙箱只报告文件数的结果 |
| Linux 许可材料 | `node scripts/collect-app-notices.mjs --linux --check`：356 packages / 618 materials，missingMaterials 为空；releaseReady=false 未改变 |
| debug ELF | 构建通过，约 7.15 秒；`debug-build.log`，未重复构建前端 |
| 必要 Wayland 链 | **1/1，16.53 秒**；`wayland-necessary.log`，根 `/tmp/mpvkit-alpha1-native-20261027-b/config/mpv` |
| 独立扩展入口 | **1/1，17.50 秒**；`wayland-extended.log`，根 `/tmp/mpvkit-alpha1-extended-20261027-a/config/mpv`；此根从未运行必要链 |
| KDE 偏好 | **1/1**；`kde.log`：语言 zh-CN、系统 dark=true，故意改变当前进程 GTK 偏好后仍正确读取 portal |
| 原生更新 GUI | 真实 WebKitGTK 短验收通过；`gui-result.json`、`gui-native.log`，根 `/tmp/mpvkit-alpha1-gui-Rwv6Ob/data` |
| release ELF / makepkg | 构建约 **43.34 秒**，makepkg 封装约 **1.21 秒**，未再次编译 Rust；`release-build.log`、`makepkg.log`，归档 SHA-256 检查通过 |
| vercmp / desktop / readelf | alpha1 < alpha2、alpha2 < beta1、rc1 < stable 均为 -1；desktop 验证退出 0，提示多主分类；readelf 的直接动态库在隔离运行依赖根中全部可用 |
| namcap 3.6.0 | PKGBUILD 和真实包完成检查，**无 E 项，有 W 项**；详见下方，不把退出 0 写成零警告 |

原生链实际验证系统 MPV 原生配置目录加载、gpu-next/waylandvk、画质切换和当前管理器保存、thumbfast 缩略图、位置续播不恢复音量、自然队列和大小写字幕搜索、uosc 菜单/倍速/模式、启动尺寸与显式尺寸保护、重置/卸载清理及系统核心/未知文件/续播保留。实际图形日志仍有驱动的 radv 非一致性警告；本轮不据此宣称画质、功耗或 HDR 硬件输出达标。

GUI 短验收使用本地 WebKit Inspector 回环接口和 Node 内置 WebSocket，无新依赖，脚本 `.local/linux-v020/gui.mjs`。连接实际 Tauri 窗口与后端，不用 Chromium/mock 响应冒充。观察启动请求计数为 1，切页后不增加；默认预览关闭、坏元数据失败可重试、预览保存后立即重查、纯文本长日志/无脚本节点、稍后与重开、AUR 准确链接/未发布时无链接、无下载按钮、install 调用数 **0**。两页及英文/深浅色偏好落盘正常，1040×760 逻辑窗口无横向溢出。脚本早先的固定中文假设和保存完成前操作已修正为等实际状态，未改产品。alpha.2 仅为本地检查夹具，不是软件包升级或实际发布。完整 640 宽/200% 字号 WebKit GUI 和物理输入仍不冒称本轮新增通过，uosc 640 宽则有原生断言。

### 实际 Arch 包安装与播放期保存

产物目录 `.local/linux-v020/release-native/`：版本 tar.zst、PKGBUILD/.SRCINFO、makepkg 输出包与 stage。包版本 `0.2.0alpha1-1`，mpv 仍为 optdepends。没有将 debug ELF 打进软件包，也没有执行 AUR helper。

创建全新 `.local/linux-v020/rootfs`，通过 Bubblewrap 用户命名空间给隔离根内的 pacman 必要权限，主机根和数据库只读；仅显式绑定隔离根可写。先从本机包缓存装入 211 个运行依赖包，不包含 mpv。逐包用主机公钥/信任环的本地副本执行 `gpg --verify` 后导入种子根；由于引导根未包含 gpg，种子导入临时配置在外部验签后关闭重复内部验签，仅用于引导。**当前 MpvKit 包的安装/重装/卸载仍用真实 pacman、正常依赖检查和本地包 Optional 签名策略**，未对主机或产品降低校验。

初次引导的 chroot/网络隔离钩子被容器权限阻止；给新用户命名空间增加 CAP_SYS_CHROOT、由外层禁网并在临时 pacman 配置禁用重复内层网络隔离后，字体、GSettings、动态链接器、MIME 与图标缓存钩子成功。种子环境的系统总线重载/Perl 维护钩子仍受无服务/基础维护工具限制；它们不是 MpvKit 的安装脚本。MpvKit 安装、重装、卸载的两个图标/desktop 钩子最终均成功，见对应日志，不以 pacman 单独退出 0 掩盖钩子失败。

1. **无核心启动**：真实 release 包在 `/usr/bin/mpvkit` 启动，窗口显示核心未安装但管理器可用；后端 root 为 `/home/test/.config/mpv`、版本 0.2.0-alpha.1，许可正文可读取（约 2.9 MB）。显式设置 `MPVKIT_TEST_MODE=1` / `/should-be-ignored` 后仍走原生目录，测试根未创建。结果 `package-no-core-result.json`，应用窗口图 `package-no-core.png` 已目视核对；不是桌面全屏截图。
2. **实际 helper 保存**：将独立扩展根中的已校验增强文件复制进隔离用户目录，补入缓存中 81 个 MPV 运行依赖及 3 个 AMD Vulkan ICD 依赖；均先验签。初次播放仅有 loader 而缺 ICD，GPU context 未建立，补齐项目已声明的 Vulkan 驱动前提后复验。由 release 后端预览/保存配置，Lua 路径确为 `/usr/bin/mpvkit`；真实 `/usr/bin/mpv` 在 waylandvk 下切换高画质并保存，revision=12、profile=high-quality。证据 `package-save-result.json`、`package-helper-result.json`、`package-playback.log`。
3. **真实 desktop 入口**：隔离根中 `gio launch /usr/share/applications/io.github.mpvkit.desktop` 启动当前 release 窗口，后端核心/增强健康，Wayland 日志 app_id 为 io.github.mpvkit。软件包图标字节与源图相同；desktop 无 MIME/媒体参数，系统 mpv.desktop 独立存在。证据 `package-desktop.log`、`package-desktop-result.json`。
4. **同版重装/卸载**：真实 `pacman -U` / `-R mpvkit-bin`，逐文件核对用户配置和未知/续播夹具，**71 个文件哈希全部相同**；系统 `/usr/bin/mpv` 哈希不变。管理器 ELF、desktop、图标、自有许可与 `/usr/lib/mpvkit` 资源均移除，系统 mpv.desktop 仍在。结果 `package-lifecycle-result.json`、`package-install.log`、`package-uninstall.log`。同版重装不当作跨版本升级。

该环境是全新用户空间根，图形连接主机 Wayland，未运行完整虚拟机或独立 polkit/systemd 桌面服务。只读共享运行时目录导致 dconf 写入提示，主机专用 GTK 模块在新根缺失也有提示；实际窗口和后端检查正常，不改主机设置来消除这些容器提示。

namcap 工具及三个 Python 依赖来自当时 pacman 仓库列出的固定版本，下载后逐一验证官方签名，只在 `.local/linux-v020/namcap-root` 解压运行，未安装到主机。警告包括固定 x86_64（当前仅支持该架构）、尚无 Maintainer 标签（公开 AUR 前由实际维护者填写）、按现有 !strip 方案保留未 strip ELF、loader 的 unused 提示、glibc/glib2/cairo/dbus/gdk-pixbuf/libsoup/图标主题通过已声明 GTK/WebKit 依赖间接满足，以及 xdg-utils 未被 ELF 静态扫描识别（实际供打开目录使用）。本轮干净运行依赖根启动验证未发现缺库，不为消除扫描警告罗列整棵传递依赖。日志 `namcap.log`、`namcap-signatures.log`。

### 产物与后续边界

| 本地产物 | 字节数 | SHA-256 |
| --- | --- | --- |
| src-tauri/target/debug/mpvkit（收尾） | 259450536 | b7deeeb6118520cf821751c640a441991ab412b9b83388419b54aa3134e11461 |
| src-tauri/target/release/mpvkit | 22737584 | 0cfcf26956021f66200229bd65370e4250749a3d705951f8f7cb2f5a6b934876 |
| .local/linux-v020/release-native/mpvkit-0.2.0-alpha.1-linux-x86_64.tar.zst | 5550950 | 1557ca646100d9154b81140808909a524fa815f5b66071284afbf51a8e987ac9 |
| .local/linux-v020/release-native/mpvkit-bin-0.2.0alpha1-1-x86_64.pkg.tar.zst | 5047867 | 3fe5289cd1dda4b27afcafe63f8415e146d18dc754a7d3b242a7067aa1503a50 |

必要链/原生 GUI 使用首次 debug 产物，SHA-256 为 d32b7d2432a8f089488a7e3261ae15ed0dfc6a2962a8780d986889ece8e29109。测试专用路径修改后，收尾重新构建两种模式核对源码/产物一致性（`debug-build-final.log`、`release-build-final.log`）；release SHA-256 与已安装验收的包内 ELF 完全相同，无需重新打包。debug 调试信息摘要变化，针对最终 debug 在新根补验实际 `--save-playback` 成功，见 `final-helper-result.json`；未重复整个 GUI/播放器套件。

所有日志/短验收脚本与包根保留在 `.local/linux-v020/`，必要/扩展原生根另在上述 /tmp 路径。只有源码、当前文档与证据摘要进入 Git；不提交 ELF、归档、用户配置、媒体、密钥或 Inspector 状态。

必要测试已完成。未新增验证真实 polkit 授权/取消、真实后继版本升级、完整 WebKit 640/200% GUI、物理鼠标/辅助技术、混合 DPI/HDR/功耗/长期媒体；这些与公开 AUR/Release、正式签名、许可 gate 保持独立，不能把本轮通过扩称为正式版发布验收。用户按计划手动 PR，远端 CI 与 main ruleset 后续由维护者跟进。

## 首次 CI 失败定位与 workflow 组件更新（2026-10-27）

用户提供 `logs_109756496328(1).zip` 与 `(2).zip`，SHA-256 均为 `39a17ed6bb9db0086e6c5570d6b3a28fa92c1fbf45e7505c89279565c7e3d1b7`，内容相同。原始运行 Windows Rust **44 passed / 4 ignored**；Arch 为 **39 passed / 8 failed / 4 ignored**，退出 101，固定 `CI` 汇总按预期拒绝通过。附件只作为失败证据读取。

Arch pacman 安装列表没有 `mpv`。失败包括配置保护、设置事务及原生目录加载，实际需要固定 `/usr/bin/mpv`；首个断言是 `player_path(...).exists()`，其他出现 `components_required` 与 `probe_failed`。不是程序编译错误，日志中的 Node 20 弃用警告也不是退出 101 的原因。本地宿主已装 MPV，前轮本地通过不能证明空容器依赖完备。

本轮只在 `.github/workflows/ci.yml` 和 `release.yml` 增加系统 MPV 测试依赖，并更新全部 5 种 Actions。官方 GitHub API `releases/latest` → tag 解引用 → 完整 commit SHA，与该提交 `action.yml` 的 Node 24 运行时交叉核验；版本为 checkout **7.0.1**、setup-node **7.1.0**、cache **6.1.0**、upload-artifact **7.0.2**、download-artifact **8.0.2**。版本来源链接与兼容要求见[流程说明](../ci/README.md#工作流依赖与版本)。Docker 官方 Registry 当前 Arch base 摘要与既有固定值相同。未调整应用工具链、CI 触发政策、分发包 MPV 可选依赖或下载摘要校验。

本地针对性验证保存在忽略目录 `.local/ci-fix-20261027/`：

| 验证 | 实际结果 |
| --- | --- |
| 缺 MPV 反例 | Bubblewrap 根只读，用独立 bin 链接目录隐藏 MPV；同一当前测试执行器精确复现原日志的 8 个失败，**39 passed / 8 failed / 4 ignored**，1.06 秒；`missing-mpv-reproduction.log` |
| 恢复 MPV 对照 | 相同只读隔离参数，恢复原 `/usr/bin`，**47 passed / 0 failed / 4 ignored**，1.35 秒；`with-mpv-reproduction.log` |
| Cargo 默认测试 | 固定 Rust 1.98.1，`cargo test --offline --locked --manifest-path src-tauri/Cargo.toml`，**47 passed / 4 ignored**，1.40 秒；`linux-tests.log` |
| CI 分类 / 分发契约 | 固定 Node 24.19.0，**3/3**；`ci-release-tests.log` |
| 工作流语法 | 官方 actionlint **1.7.12**，下载归档匹配官方 SHA-256 `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`，两条工作流通过；未安装 shellcheck |

首次反例脚本的挂载目标在只读 `/run` 下无法创建，Bubblewrap 启动即拒绝；改为事先建立的隔离目录挂载后才得到上述有效反例。没有删除、改名或覆盖主机 `/usr/bin/mpv`，没有改真实配置/包数据库。未新增测试跳过项，也未重跑与工作流环境修复无关的完整 GUI、播放矩阵和安装包验收。

用户随后明确授权修复完成推送 dev 并 PR 到 main；已存在 [PR #1](https://github.com/Leovikii/mpvkit/pull/1)，沿用该 PR 更新并由 push 触发同步检查，不重复建 PR、不合并 main。此节记录本地定位和验证，远端各次执行以 [PR Checks](https://github.com/Leovikii/mpvkit/pull/1/checks) 的提交及结论为准；未手动触发 Release，不将本次修复写成公开发布或正式版验收。

## main 自动 CD 与用户文档（2026-10-27）

前轮修复提交 9ff35f4 的远端 [Checks #40497998384](https://github.com/Leovikii/mpvkit/actions/runs/40497998384) 全部 success：Arch Rust **47 passed / 4 ignored**（5.40 秒）、Windows **44 passed / 4 ignored**（10.98 秒），前端 / changes / 固定 CI 汇总成功；完整日志中无 warning，Node 20 弃用提示消失。原始成功日志留在 `.local/ci-fix-20261027/remote-success.zip`。PR #1 随后由用户合并，当前基线 84fd7b7；之前的 main 合并没有 CD 是因为 Release 仅支持 workflow_dispatch。

用户追加要求 main 每次推送 / PR 合并后检测版本与 tag，缺 tag 自动创建并进入 CD，日志采用极简英文 Markdown；主 README 面向用户，与 AI 开发文档分开，提供预设设计链接。

本轮新增 `scripts/prepare-release.mjs`，复用原有 versionCheck，核对 package.json / Tauri / Cargo 与 notes；GitHub API 固定官方端点、15 秒超时，只有 404 可进入建 tag，提交 SHA 来自事件且校验格式。创建权限只授予 prepare，构建仍只读；Release 草稿与推广各保留必要写权限。脚本不接受外部 API URL，也不打印 token；checkout 继续不持久化凭据。新 tag 直接作为 needs 输出传给后续任务，避开 GITHUB_TOKEN 创建事件不递归触发工作流的限制。

首次建 tag 后构建失败，不自动移动 / 删除 tag；手动选择同 tag 的 build-draft 或重跑失败任务。并发 API 创建冲突需重新确认 tag 存在才跳过。自动版本流程使用每个 main 事件独立的 concurrency group，避免不同待执行版本被新提交取消；公开推广另设全局串行 group，防止 channels.json 并发覆盖。

日志 `docs/v0.2.0-alpha.1/notes.md` 改为 `# 0.2.0-alpha.1` 与四条英文要点，Release / updater 仍共用一个来源。README 按用户补充要求改用英文，删除开发栈、测试命令、历史授权和 agent 接手说明，保留用户功能、Windows / Arch 安装及许可；`docs/presets.md` 解释三档预设、默认行为、配置预览 / 手改 / 重置边界。开发入口继续由 AGENTS 和 docs/README 提供，不删除历史证据。

本地验证：

- 固定 Node 24.19.0，`node --test scripts/release.test.mjs scripts/ci-changes.test.mjs` **10/10**，约 68 毫秒；包含新建 tag 绑定准确 SHA、已有版本跳过、403 查询失败、500 创建失败、422 冲突后仍不存在拒绝、422 后确认存在去重、手动重试 / 推广 / 错 tag / 缺 tag，以及原有资源布局 / 完整性 / 渠道防倒退与 CI 分类。HTTP 响应使用 Node 原生 mock，不调用真实写 API；日志 `.local/auto-release-20261027/release-tests.log`。
- actionlint **1.7.12** 验证两条实际工作流；JS 语法、notes 标题 / 英文文本、本地链接及 `git diff --check` 通过。无新依赖或应用行为改动，不重复 Rust / 原生 GUI / 设备 / 安装包全套测试。

只读 GitHub 环境 API 返回 `[]`，尚无 release-build / release-publish 配置。需维护者放入正式签名公钥变量、私钥 Secret 与所需密码并设置环境保护；部署策略需允许 main 事件，不能因为任务检出版本 tag 就只允许 tag 事件。许可 gate 仍为真实 releaseReady=false，公开推广未验收。本轮本地验证不创建真实 tag / 草稿，不修改密钥或远端环境，不将 PR 检查当作 main push 自动 CD 的端到端验证。

## 正式签名重试与草稿验收（2026-10-27）

PR #2 合并后 main 提交 2f3be38 触发 [Release #40502127238](https://github.com/Leovikii/mpvkit/actions/runs/40502127238)，prepare 创建 `v0.2.0-alpha.1`，Arch 构建 / 归档 / 软件包成功，Windows Rust 44/4 ignored 通过，但因缺公钥停在签名构建前。用户提供新正式公钥后，核验 Tauri Base64 / Minisign 结构，写入两个发布环境的 `MPVKIT_UPDATER_PUBLIC_KEY`；正式公钥原始 42 字节的 SHA-256 为 `db97b55d9ee531dd561fbcfe1eb7c3c49b0b50089349c4779912a0659964b4c8`，未把正式私钥或本机路径写入源码。

用户要求检查文件并重试时，名称核对发现私钥误存为 build 环境 Variable，Secret 列表为空。请求用户重新保存后，核对 `TAURI_SIGNING_PRIVATE_KEY` Secret 名称存在且同名 Variable 消失；没有读取私钥内容。无密码，未添加密码 Secret。源码开发配置的空公钥与独立测试公钥夹具均保留：正式公钥由打包脚本注入，不需要修改业务代码。

仅调用 `gh run rerun --failed`，第二次执行成功：Windows NSIS / updater 签名、draft success；第一次成功的 prepare / Arch 结果复用，promote skipped。没有移动 / 删除 tag，没有重复跑 Linux 构建或执行安装器。签名对应版本注释为 `version:0.2.0-alpha.1`。

成品验收另发现工作流 `gh release create ... *.*` 没有上传无扩展名 `SHA256SUMS`，否则未来推广会因缺文件失败。新增测试直接在临时目录创建真实文件并用 Bash 展开实际工作流的两条上传命令，先得到 `Missing upload: SHA256SUMS`，再对稳定 / 预览两分支显式加入清单文件参数；完整本地分发与 CI 分类 **11/11**（约 65 毫秒）通过，actionlint 1.7.12 通过。只修上传列表，不改产物生成 / 签名 / 许可门槛。

本次草稿已经用旧 tag 中的工作流创建，不能移动 tag 后重建。将其 9 个原有资产逐一核对 GitHub REST 提供的 SHA-256 上传摘要，再按既有 release.mjs 的排序 / 哈希算法从这些未变字节重建 `SHA256SUMS`，通过精确草稿 Release ID 仅补传这一文件，并核对上传回执摘要。按 tag 的 REST 查询及本机 CLI upload 不返回草稿，改用认证可见的草稿列表和上传 ID；没有公开发布来绕过此限制，也没有替换任何已有资产。

独立验证工具 Minisign 0.12 来自 Arch 官方包 `minisign-0.12-2-x86_64.pkg.tar.zst`，包 SHA-256 `16a98b33121b1bf1b4211d868cbba45020bc98306612134d3d2e19e2b9a2f0f5`，先用既有 Arch 公钥环验签，再只提取到忽略目录，未安装系统包。Minisign 输出 `Signature and comment signature verified`；9 项清单均匹配，`node scripts/verify-release.mjs <assets>` 元数据 / 签名正文 / 资源完整性通过。

| 本次正式签名产物 | 字节数 | SHA-256 |
| --- | --- | --- |
| MpvKit_0.2.0-alpha.1_x64-setup.exe | 4092195 | 713042bfff5a3953fa03ce112ca324eb4f49ef96f02d9c581721887b336eab51 |

证据与下载资产保留在 `.local/release-retry-20261027/`：`draft.json`、`release-rest.json`、`checksum-upload.json`、`verification-result.json`、`verify-signature.log`、工具包验签日志、针对性测试与 actionlint 日志。当前 [Release 草稿](https://github.com/Leovikii/mpvkit/releases/tag/untagged-2ed9aec3aec4e12a0ab2) 保持 draft / prerelease；没有运行 promote、公开 Release/AUR 或修改真实配置。发布环境保护规则、许可 gate 与实际安装升级验收仍未关闭。

## 本机旧测试配置清理（2026-10-27）

用户明确授权本机清理。检查范围为已知 MPV / MpvKit 配置目录及固定用户菜单入口，未扫描媒体目录。系统已安装 `mpvkit-bin 0.2.0alpha1-1`；`$HOME/.config/mpv/.mpvkit/settings.json` 保留当前 `config::portable::State` 不接受的 `integrationEnabled` 字段，并有旧 launcher。`$HOME/.local/share/applications/io.github.mpvkit.desktop` 为指向旧应用资产的链接，其 Exec 指向仓库 debug 程序。

本机忽略目录 `.local/cleanup-20261027/` 保留一次性 `cleanup.py`、具体 `plan.json` 和 `result.json`，不含配置内容备份。执行前确认无 MPV / MpvKit 进程，核对旧 owner/install 记录、路径无符号链接跳转、73 个 MPV 文件和 2 个桌面资产摘要全部匹配；持有两个既有 `operation.lock`，重新校验完整预览及保护文件，再逐项删除。

- 删除 **79 个文件/链接，303713038 字节**，仅移除 **13 个空目录**；包括旧增强/预设、launcher、设置及安装清单、旧应用偏好和旧用户菜单链接。没有删除整个 MPV 根或跟随菜单符号链接递归删除。
- **27 个保护文件**执行前后摘要与身份一致，含 **11 条 watch_later**、系统 MPV、系统 MpvKit 包文件、默认应用设置及两个根的 owner/lock。清理后 MPV 根仅保留 `.mpvkit` 归属/锁和续播目录，旧应用根仅保留归属/锁；当前 `io.github.mpvkit` 数据根没有偏好文件，WebKit 缓存不涉及旧配置而保留。
- 新版系统 desktop 入口保留，`kbuildsycoca6 --noincremental` 退出码 0。没有改动系统包数据库、默认应用、源码、密钥或 Release；未启动真实 GUI / 重新部署增强，需用户重新打开新版并安装增强、保存设置。

## 自动公开发布修正（2026-10-27）

更新失败诊断：请求应用固定地址 `https://github.com/Leovikii/mpvkit/releases/download/updates/channels.json` 返回 **HTTP 404**；认证 GitHub API 只列出 `v0.2.0-alpha.1` 草稿（ID 425452414），没有 updates Release。不是旧配置清理造成，也不能把服务器缺少索引伪装成已是最新。

用户要求公开版本并修正 workflow。旧 `draft` 任务删除，替换为纯 Actions 资产收集及自动 publish；公开创建命令不传 `--draft`，已有草稿重试明确转为公开。新增 `scripts/publish-release.mjs` 在发布前检查远端既有资产和渠道，公开版本后创建/更新索引并读回核验。`release-publish` 原有许可 / 签名门槛保留，构建和发布只使用各自需要的权限。

正常 Node 进程中执行 `node --test scripts/release.test.mjs scripts/ci-changes.test.mjs`，**20 passed / 0 failed**，覆盖公开上传含无扩展名校验单、首发初始化、既有草稿公开、幂等元数据重试、保留另一渠道、资产篡改、索引缺失/HTTP 404/错误 schema/倒退、API 403、tag 并发与 main 已有 tag 跳过。全部网络及 gh 发布命令在测试中 mock，没有真实发布。actionlint 1.7.12（无 shellcheck）、JS 语法与 Git diff 检查通过。

现有 9 项 `SHA256SUMS`、updater 元数据和 Minisign 的签名 / 版本注释重新验证通过。随后 PATCH 草稿为公开预发布的操作被自动审批拒绝：可信项目指令要求许可审阅及验收先于公开，而当前许可 gate 的未完成记录未关闭；公开指令未明确豁免该前提。没有换工具、改工作流或翻转布尔值绕过这次拒绝。公开操作尚未执行，当前改动仅在本地。
