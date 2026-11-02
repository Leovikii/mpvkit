# v0.2.0-beta.3 实施与验证证据

日期：2026-11-02。接续 dev `c6d4e93` 的已批准方案；实施及必要测试完成，用户随后反馈测试通过、未观察到问题，并授权推送 dev、PR 到 main、CI 通过后合并发布及同步分支。公开 beta.2 的资产与标签不变，不继承其测试结果。发布执行结果另见本页后续记录。

## 实现结果

- 主播放设置仅保留默认画质，支持安装前选择；产品和离线预览同步删除 HDR、自动连播、结束行为及失效草稿字段。Settings 使用 schema 4，不增加旧字段迁移。
- 完整预设默认 HDR Auto、`autocreate-playlist=filter`、`keep-open=no`。默认值位于可手改区，普通画质保存保留手改，完整重新应用恢复默认值。全部三套画质 profiles 仍部署，默认画质沿用当前选择。
- uosc 菜单增加 HDR 自动/开启/关闭、自动连播开启/关闭、播放结束后退出/保留窗口；通过原生 profiles 和命令实现，三项只影响当前 MPV 进程。HDR profiles 完整写入各平台原有三项关联参数，画质业务保持原有临时切换和显式保存模型。
- 中英文共用单套菜单结构和文本表。安装/完整重新应用接收管理器实际解析语言，生成 input.conf、自有 Lua、uosc 控件提示及 `languages=zh-hans,en` 或 `languages=en`。语言记录随现有事务写入 `.mpvkit/settings.json`，普通保存、helper 和修复保留安装语言；管理器切换语言本身不改播放器文件。
- 源码版本统一为 `0.2.0-beta.3`，整套预设修订为 3；更新来源清单中自有 uosc 配置模板的大小、摘要和生成说明。固定 uosc 5.13.0 的部署 Lua 保持上游原文字节，不添加外观桥接。

## 环境与命令

Windows x64，本机固定 Node 24.19.0、pnpm 11.25.0、Rust 1.98.1、Mantine core/hooks 9.7.3；工具与固定资产通过 `.local/beta1-windows/env.ps1` 加载。实际 MPV 来自已校验 WinGet 研究样本 v0.41.0-244-gaf9c81fa1，FFmpeg 使用已有固定摘要夹具。没有调用真实 WinGet、修改真实 MPV 配置、默认播放器或系统登记。

构建和检查在仓库根运行：

```powershell
. .local/beta1-windows/env.ps1
pnpm build
node --test scripts/frontend.test.mjs scripts/release.test.mjs
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle --config .local/beta3/native.json
node docs/v0.1.6/preview/build.mjs
node docs/v0.1.6/preview/check.mjs
node --test --test-concurrency=1 --test-name-pattern='native two pages|native combined|native installation|native preset|native v016|native auto continue|native v018 uosc|native beta3' scripts/native-v016.test.mjs
```

本地测试构建的 `.local/beta3/native.json` 仅覆盖 identifier 为 `io.github.mpvkit.beta3test` 和 `build.beforeBuildCommand` 为空，避开宿主已安装管理器的单实例锁，并复用已生成 dist；该文件不入库，正式 identifier 不变。测试执行实际 debug EXE，所有配置部署、移除和 MPV 播放均位于新建隔离根。

## 通过结果

| 验证 | 结果与范围 | 本机日志 |
| --- | --- | --- |
| 前端构建 | TypeScript 与 Vite 通过；仅有既有大 chunk 提示 | 构建输出 |
| Rust | 56 passed、0 failed、5 ignored；含语言保留/重新应用、可手改默认值、覆盖保护、路径边界、过期预览、恢复、更新下载验签 | `.local/beta3/rust-test-final.log` |
| Clippy / 格式 | `--all-targets -D warnings`、`cargo fmt --check` 通过 | `.local/beta3/clippy-final.log` |
| 前端与发布契约 | 19 passed；中英文系统语言解析、翻译参数及模拟发布/更新契约，不操作远程 Release | `.local/beta3/js-tests.log` |
| Windows 原生构建 | debug EXE 构建成功 | `.local/beta3/native-build-final.log` |
| 离线界面矩阵 | 两页、安装前编辑/草稿、一次确认、安全流程、键盘与回焦；480/560/720、双语明暗、200% 文字、减少动态效果及按钮文字安全距离通过 | `.local/beta3/preview-final.log`，截图 `.local/ui-v016/` |
| Windows 受影响原生场景 | 9 项通过，含 52 个实际 MPV 选轨用例；详见下文 | `.local/beta3/native-regression-final.log` |
| 文档与差异 | 本批本地链接/锚点、官方快照摘要、`git diff --check` 通过 | 本地检查输出 |

Windows 原生回归覆盖：

1. 两页语言、主题、日志和仓库入口；GUI 不再出现三个已删除控件。
2. 安装前具体预览不获取资产，确认后下载及部署；预览后文件变化阻止继续。同一弹窗保留确认、进度和结果。
3. 完整安装、画质独立保存、配置手改保护、过期计划、完整移除；重新应用取消、预览后修改、保留当前画质和未知文件、成功才记录修订 3。失败恢复与路径边界另由 Rust 事务测试覆盖。
4. 三套画质 profiles、菜单临时切换、切集恢复、helper 安全保存及管理器读取；52 个真实 MPV 选轨用例、播放器快捷键、位置续播和字体加载通过。
5. 实际 uosc 排列、音轨存在条件、上下集、倍速菜单、鼠标操作、全屏、独立循环与随机、EOF 和重启；部署的上游 main.lua 摘要核对一致。
6. 自动建立同目录自然排序队列，音视频过滤、不扫子目录；自动连播关闭后后续 `loadfile` 只加载单文件，重新开启不重建当前队列，下次打开才建立队列。显式列表、字幕目录与语种后缀、非最终文件连播、最终文件退出/保留窗口通过。
7. 新增 beta.3 原生语言场景：英文安装 → 管理器切中文不写文件 → 普通画质保存保持英文 → 完整重新应用中文、保留高画质 → 新 MPV 读取中文菜单。核对三个文件摘要及安装语言记录；所有自定义标题、子菜单和画质提示使用简洁单语言文本。
8. 通过真实菜单键盘操作执行 HDR On → Off → Auto，核对 `target-trc`、`target-prim`、`target-colorspace-hint` 的完整往返；Windows 值分别为 `pq/bt.2020/true`、`bt.1886/bt.709/false`、`auto/auto/auto`。自动连播菜单修改选项且不改当前队列，结束行为菜单修改 `keep-open` 且不立即退出。菜单检查使用音频及 null 输出，仅证明命令和属性行为。

播放器检查探针只追加在测试根的上游副本中，用于读取实际菜单项和控件；生产部署的上游文件未修改。原生测试日志与 result.json 留在 `.local/e2e-artifacts/`，最终各场景路径由回归日志打印，不入库。

## 依据、失败尝试与限制

目标 MPV 包自带 `doc/manual.pdf` 已核对 `--autocreate-playlist`、`--keep-open`、`--target-colorspace-hint` 和 `--target-trc`，摘录留在 `.local/beta3/mpv-manual-excerpts.txt`。手册确认自动队列在打开本地文件时建立，`keep-open=yes` 在无下一文件且未循环时保留窗口；HDR 元数据仍依赖平台、合成器、驱动和显示器。固定 uosc 的公开语言/菜单依据见 [方案](plan.md#固定上游能力依据)。

首次沙箱 Rust 测试被 Windows 文件替换及回环端口策略阻止，离线浏览器检查被启动策略阻止；按相同隔离范围在沙箱外复跑通过，没有用占位结果代替检查。编译器报告增量缓存硬链接退回复制及 Windows 链接信息，未出现 Clippy 代码诊断。

适配回归脚本时修正了旧版假设：空闲播放器没有媒体时不应等待播放时间；HDR 默认值已移至可手改区，冲突应修改管理块；预设修订应为 3；删除 GUI 控件后无草稿不应点击保存；中文确认应使用中文按钮。新增菜单探针又修正了异步检查竞态、ESC 关闭尚未完成就打开下一菜单的竞态，以及 MPV 将 `autocreate-playlist=no` 返回为布尔 false 的断言。最终对应场景复跑通过，先前失败日志保留于 `.local/beta3/`，不将失败尝试计为通过。

本地实施阶段未执行 Linux/Arch 编译、KDE Wayland 原生 GUI/菜单/实播及 Pacman 包验收；Linux 测试入口已适配新设置与 HDR profiles，但 Windows 构建不编译这些条件模块。真实 HDR 显示器输出、多屏 DPI、辅助技术、所有自有 OSD 故障分支、真实 WinGet/UAC、Polkit/Pacman、跨版本更新与自动重启未验。用户现已授权本版发布，双端 CI、正式签名和公开资产核验结果将在下文单独补录，不把本地测试等同发布完成。

## 用户实测程序

用户随后要求构建测试程序待实测。已重新运行 `pnpm build` 和上述 Tauri debug 构建，交付目录为 `.local/beta3-test/`；`run.cmd` 启动管理器，`play.cmd` 在隔离配置中播放附带的两段 20 秒音视频，也接受拖入媒体路径。应先运行管理器并安装配置，再打开播放器。

目录包含实际 EXE、固定增强资产、已验证核心副本、来源许可材料和本地 WinGet 夹具；`build.json` 记录版本、摘要及大小。启动脚本只在该目录的 `session/data/` 中保存应用数据与 MPV 配置，首用初始化核心；真实包管理器不执行。启动请使用脚本，直接双击 EXE 不会设置隔离环境。交付前 session 尚未创建，供用户从安装前状态开始。

交付副本通过 `native two pages` 冒烟测试（1 passed），验证原生启动、语言/主题、日志和仓库入口；启动脚本语法和资产摘要检查通过。日志为 `.local/beta3/manual-build-frontend.log`、`manual-build-native.log`、`manual-package-smoke.log`。用户本轮反馈测试通过，未观察到问题；未提供逐设备测试明细，不据此关闭真实 HDR/KDE 等未验项。

## 主界面增强明细与布局校正

用户要求将组件明细从高级项移到主界面，并提醒按相关规范设计。现将 uosc、thumbfast、字幕字体直接列在“播放增强”组内，各行显示名称、用途、已安装版本和独立状态；删除重复折叠项及失效翻译键。未安装、已就绪、需要修复分别由现有组件版本/健康报告展示，不新增后端状态、单独安装或设置开关；thumbfast 提交号仅显示七位，完整来源保留在高级项。

复核微软 [App settings](https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings)、[Content layout and spacing](https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics)、[Typography](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography)，及 Mantine Paper/Text 官方公开 API 与本地 9.7.3 类型。按现有项目规范将分组标题置于卡片上方，使用 14px/600、标题下距 8px、组间 24px；行主文案 14px/400、辅助信息 12px/400，状态沿用右对齐。组件分隔线与设置行一致，首行前不画额外横线；两页与底栏统一常规 24px、窄窗 16px 水平边距，保留 64px 最小行高及内容增长换行。具体尺寸为项目选择，未增加依赖、通用列表封装或新样式系统。

本批前端构建、2 项前端契约测试、更新后的离线完整矩阵及 `native two pages` 原生冒烟（1 passed）通过。预览额外核对未安装、部分缺损、单项损坏和全部就绪的三项独立状态，组件在高级项关闭时可见；双语/明暗、480/560/720、200% 文字、键盘与回焦、底栏和横向溢出检查通过。人工查看主界面与放大截图；未将模拟预览当作 Linux 或辅助技术验收。初次 JSX 构建发现缺失闭合片段，修正后构建及矩阵复跑通过。日志为 `.local/beta3/component-ui-build.log`、`component-ui-preview.log`、`component-ui-native-build.log`、`component-ui-native-smoke.log`。

`.local/beta3-test/mpvkit.exe` 已替换为最终布局的构建并通过交付副本原生冒烟，`build.json` 同步摘要；原启动入口和测试数据保留。此次只改 UI 与文档，未重跑已通过的播放器/Rust 全套。

## 发布前复核

用户确认测试通过并授权本版发布。版本/tag 契约、发布材料 gate、19 项前端/发布模拟契约测试通过。发布前发现两端依赖材料仍记录 beta.2 输入摘要，按现有 collector 重新生成后通过 Windows/Linux `--check`：仅更新版本标题及 Cargo/package 输入摘要，包数、许可原文和依赖均未变化，无缺失材料。日志 `.local/beta3/pre-release-contracts.log`；后续 CI、PR、正式签名及匿名核验记录待实际完成后补充。

PR #18 首轮 Arch 默认测试为 63 passed、1 failed、5 ignored：语言测试用任意双下划线判断模板残留，将嵌入 helper 路径的 GitHub runner `/__w/` 误判为占位符。改为逐一检查文本表中的确切 token 及 `__MPVKIT_HELPER__`，不修改生成逻辑或放宽业务验证；本地定向复验及后续 CI 结果分别记录。原日志 `.local/beta3/pr-arch-failure.log`。

## 双平台 CI 与合并

实现提交 `b2c8592`、测试修正 `cc2b814` 已推送 dev，经 [PR #18](https://github.com/Leovikii/mpvkit/pull/18) 合并为 `8ab5afd61fb2277294b282c03d34c7465a541c51`。修正后的 [Checks 41458874188](https://github.com/Leovikii/mpvkit/actions/runs/41458874188) 全部成功：前端、Windows/Arch Rust、最终 CI 汇总均通过。Windows 为 56 passed/5 ignored，Arch 为 64 passed/5 ignored；Arch 包元数据专测 1 passed，隔离 Pacman 安装/重装/资源/卸载通过，不等同 KDE 或真实系统授权测试。两端日志分别保存在 `.local/beta3/pr-windows-success.log`、`pr-arch-success.log`。

本地断言定向复验为 1 passed，见 `.local/beta3/runner-path-test.log`。tag `v0.2.0-beta.3` 固定在合并源码 `8ab5afd`；[Release 41459748720](https://github.com/Leovikii/mpvkit/actions/runs/41459748720) 已由 main 推送触发，后续文档补记不移动该 tag。

发布首轮 Arch 正式构建、签名及隔离包验收成功；Windows 默认测试 55 passed、1 failed、5 ignored，失败发生在 junction 安全测试调用 PowerShell 创建隔离夹具时超过 10 秒，尚未执行目标路径拒绝断言。错误复用了 `probe_timeout` 文案，不是 MPV 播放失败。原日志 `.local/beta3/release-windows-failure.log`；同一源码本地定向复验 1 passed，夹具及断言在 0.53 秒内完成，见 `junction-recheck.log`。保留 tag 与成功的 Arch 任务，仅通过 Actions 重跑失败任务；不修改产品或放宽安全检查，重试结果见后续发布记录。

第一次重试仍在同一 PowerShell 夹具启动处超时（55 passed、1 failed、5 ignored），见 `release-windows-retry-failure.log`；原 PR 同项成功但全套耗时 15.35 秒，发布失败两轮为 16.19/16.45 秒。为避免新 runner 的 PowerShell 冷启动占用夹具 10 秒上限，发布工作流在测试前显式启动一次 Windows PowerShell（独立 1 分钟限制、失败仍阻断），保留原测试、断言及超时。此为发布流程修正，版本源码/tag 不变，通过 PR 检查后按既有 tag 的 release 重试入口重新构建签名，不沿用失败测试结果。
