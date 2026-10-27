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
