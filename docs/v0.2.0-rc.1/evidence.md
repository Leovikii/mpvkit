# v0.2.0-rc.1 实施与验证证据

日期：2026-11-05。基于 dev 的 `afadae2` 实施 **0.2.0-rc.1 / 整套预设修订 4**，本机自动化及测试程序构建已完成。用户随后反馈测试通过，并授权推送更新、合并 main 发布 rc.1，随后同步 dev 到 main。发布前 main 为 `167efb2`，beta.3 标签固定 `8ab5afd`；不移动已发布 tag。历史版本通过结论不计入本版。

## 本轮实现

- 画质、HDR、自动连播、播完后通过同一受控 helper 更新一项，复用原 settings、配置管理区、代际/修订、摘要、互斥与失败恢复；无新依赖或偏好数据库。
- 即选即应用并保存，删除单独保存与切集恢复。Lua FIFO 串行保存，原生应用失败回滚，保存失败保持运行值并提示，可重选重试。外部属性观察只刷新菜单，不保存。
- 菜单仅一棵数据树，原生 active 读取实际属性，公开 open-menu/update-menu 保留导航状态。常用项置顶、短文案；增加信息按钮，列表至少两项时显示两侧切集。固定上游 uosc 未修改。
- Linux 窗口策略原样拆入仅 Linux 部署的 mpvkit-window.lua；自有脚本进入健康检查及修复，未知同名脚本不覆盖。
- 管理器只显示画质；共用业务入口在预览/保存/修复/重新应用时保留磁盘上的最新三项播放器专属选择，避免旧 GUI 草稿覆盖新偏好。画质使用当前页面选择。
- 普通 Windows 安装成功显示完成页，默认勾选启动，完成前不启动；updater/passive/silent 保持原有路径。检查 RunAsUser 返回值，失败反馈。

详细契约及后续入口见[计划](plan.md)。README 公开下载仍指向已发布 beta.3。

## 环境与命令

本机 Windows x64，固定 Rust 1.98.1、Node 24.19.0、pnpm 11.25.0、Tauri CLI 2.12.3，未升级依赖。Windows 原生套件使用固定 WinGet MPV 样本（包标签 0.41.0，实际构建 v0.41.0-244-gaf9c81fa1）及已校验 ffmpeg/组件资产。`. .local/beta1-windows/env.ps1` 只准备本机工具与资产路径；新主机按[测试入口](../testing.md)重新准备。

所有日志、探针和媒体留在忽略目录 `.local` 或测试自行创建的临时根，不提交或分发。没有执行真实 WinGet、Pacman、正式 NSIS、默认播放器登记或真实配置写入。

| 检查 | 本轮结果 | 本机日志（仓库根相对路径） |
| --- | --- | --- |
| `pnpm build` | TypeScript/Vite 通过；Vite 提示单 chunk 超过 500 kB | `.local/rc1-build.log` |
| `pnpm test:frontend` | 2 通过 | `.local/rc1-frontend.log` |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | 通过 | 终端结果 |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | 59 通过，5 显式 ignored | `.local/rc1-unit-final3.log` |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings` | 通过 | `.local/rc1-clippy-final4.log` |
| `node --test scripts/release.test.mjs` | 17 通过；契约检查不等于实际发布 | `.local/rc1-release-final.log` |
| `node scripts/collect-app-notices.mjs --check` | Windows 331 packages / 587 materials，无缺失 | `.local/rc1-notices-final.log` |
| 同上加 `--linux` | Linux 410 packages / 717 materials，无缺失 | `.local/rc1-notices-linux-final.log` |
| 目标 MPV 执行 playback-policy.lua | FIFO、四项输入、失败回滚、保存重试、观察不写盘通过 | `.local/rc1-policy-details.log` |
| 目标 MPV 执行 window-policy.lua | 安静期/实时复核/取消/一次性修正/音频先行通过 | `.local/rc1-window-policy-details.log` |
| `node docs/v0.1.6/preview/build.mjs` 与 `check.mjs` | 当前两页预览及窄窗/中英/主题/200% 文字通过 | `.local/rc1-preview-build.log`、`.local/rc1-preview.log` |
| Tauri Windows debug 单 EXE 构建 | 通过；MSVC 输出创建 import library 的链接消息 | `.local/rc1-native-build-final2.log` |
| `node scripts/test-windows-installer.mjs` | 安全 NSIS 页面探针 7 场景通过 | `.local/rc1-installer3.log` |

Rust 默认忽略项涉及固定资产/网络/真实图形，不能把 ignored 记为通过。真实组件部署与 52 个媒体轨道案例另由下述 Windows 套件执行。

Lua 检查使用目标 MPV 的 `--no-config --load-scripts=no --vo=null --ao=null --idle=yes --script=<policy.lua>`，读取真实产品模板；不读取用户配置。窗口策略的确定性时钟覆盖不能证明 KDE 合成器时序。

## Windows 原生回归

完整命令：`node --test --test-concurrency=1 scripts/native-v016.test.mjs`。使用真实 WebView2、隔离组件安装、目标 MPV 与实际 helper；包管理器为本地 fixture，不运行真实系统安装。

首轮 `.local/rc1-native-all.log`：13 项中 11 通过、2 失败。已通过项包括更新检查/下载界面、WinGet 终端夹具、窗口尺寸、自然队列/字幕、完整配置事务（含 52 个真实 MPV 轨道案例）、单实例、两页界面、预设重新应用、完整安装/卸载、确认前不下载以及 rc.1 菜单专项。

两处失败经定位后修正测试：Esc 后等待原生菜单关闭再打开，避免淡出时 toggle 把新菜单关闭、导航键落到音量；窄窗/高缩放遵守上游收缩，不强制音轨/信息按钮始终可见，改验更多菜单仍可达。跨 Lua 消息后增加实际菜单状态等待。没有修改上游布局或添加外观 Lua。

- `.local/rc1-native-recheck.log` 中真实按键/菜单/位置续播及 rc.1 菜单均复验通过；其中布局仍有异步读取失败，随后继续修正。
- `.local/rc1-controls-final3.log`：布局/播放模式/stats 单项最终通过。取证目录 `.local/e2e-artifacts/mpvkit-v018-edr929`，覆盖 1200/900/640、原生按钮、单/多音轨、亮背景、倍速、随机/独立循环及 EOF、信息按钮/键盘、全屏、运行期列表 0/1/2 项和增删、进程重启。
- `.local/rc1-manager-final.log`：最后四项定向复验全部通过（配置安装/保存及 52 轨道案例、预设重新应用、统一安装/卸载、rc.1 菜单）。菜单专项保留播放器操作前的 GUI 草稿，再验证保存/修复/重新应用没有覆盖新偏好；取证 `.local/e2e-artifacts/mpvkit-rc1-menu-IDcGet`。此次使用包含旧草稿修复的最终 debug EXE。

合计 13 个原生项目均有本轮通过记录，采用首轮加修正后定向复验；没有宣称最后完整 13 项单次重跑通过。最后只改后端默认值保留和对应测试，未重跑无关图形项目。

rc.1 菜单专项覆盖四项保存/实际激活、同值不写、连续选择、保存失败后重试、外部属性变化不写盘、父子菜单/焦点/滚动、切集与重启、安装语言规则、管理器保存/修复/重新应用保留选择。取证向私有 uosc 副本追加检查代码前核对固定上游摘要；仅更新测试 fixture 的归属摘要，产品不包含插桩。

## Windows 安装器

安全探针从真实模板提取初始化、完成页、成功收尾，编译原生 NSIS fixture 和只写测试日志的 launcher，使用固定 nsis_tauri_utils 的真实 RunAsUser。仅操作精确 probe PID/映像，未写注册表或产品目录。

`.local/rc1-installer-1793865317157/result.json` 记录 7 场景：中文默认勾选且完成前不启动、英文取消勾选不启动、`/P`、`/UPDATE /P /R /ARGS`、`/S`、`/S /R`、失败。更新参数仅启动 fixture，不证明真实 updater 完成跨版本重启。普通流程完成后只记录一次 launch；静默无隐式启动。

完整真实模板已用 Tauri 编译为未签名 debug NSIS（`--bundles nsis`，本机配置 `bundle.createUpdaterArtifacts=false`），未运行。未使用测试密钥，不能用于正式分发。最终代码的编译日志 `.local/rc1-package-final.log`，产物 `src-tauri/target/debug/bundle/nsis/MpvKit_0.2.0-rc.1_x64-setup.exe`；单 EXE 为 `src-tauri/target/debug/mpvkit.exe`。仅供本机隔离验证，均不进入 Git。

| 本机产物 | 字节数 | SHA-256 |
| --- | --- | --- |
| 与 NSIS 同批构建的 mpvkit.exe | 25521664 | `39907d470c4076066062ff53b64dbcbeaffdc36e68d3d6286d3514c465d3db26` |
| debug NSIS | 5291310 | `596d4b16509971071597bbdda5774c0387efd4923829a145671b20bba634dae4` |

### 用户手动测试程序（2026-11-05）

用户另行要求构建测试程序后，重新运行 `pnpm build` 及 Tauri `build --debug --no-bundle`，均通过。日志为 `.local/rc1-test-frontend.log` 与 `.local/rc1-test-build.log`。独立 EXE 已复制到 `.local/rc1-test/MpvKit_0.2.0-rc.1_x64.exe`，25521664 字节，SHA-256 为 `0386dd81501518267ed37ee5553a456cdbf6937de96ec57890e2a0c6b46e5549`。未自动启动或执行安装，未改代码；该未签名 debug 程序供用户手动测试。此时 target/debug/mpvkit.exe 对应本次单 EXE 构建，前表保留 NSIS 同批构建时的历史摘要。

## 首轮问题与修正

- MPV 将 profile 中 `99.995` 回读为 `99.995003`，初版严格字符串比较误判应用失败。现 profile 数值使用相对 `1e-6` 容差，倍速六档仍精确匹配；实际 MPV 菜单复验通过。
- uosc 的公开菜单类型使用 native 属性读取，初版字符串 getter 无法正确触发更新；已修复，并验证打开期间实时激活与导航状态。
- 发现管理器旧草稿可能覆盖播放器刚保存的隐藏偏好，修到共用业务入口；Rust 覆盖旧草稿保留三项，原生专项保留玩家操作前的 GUI 草稿再保存/修复/重新应用。
- 版本变化后原依赖材料输入摘要过期，已重新生成双端 notices/inventory 并通过 `--check`，依赖清单未升级。
- 一次 debug NSIS 编译与原生测试同时使用 EXE 导致文件占用；串行重试成功。之后图形测试与构建串行执行。
- 最后补测时 Clippy 提示测试字段应在 Default 初始化器中赋值，已改并通过，不屏蔽告警。

文档本地链接/锚点、全部 29 份 Mantine 原文快照大小/摘要、Node 测试入口语法及 Git diff 空白检查通过；Linux 窗口策略与拆分前原文比较一致。固定组件清单与上游源文件未改。

## 未测与后续入口

1. 未在 Arch/CachyOS 构建本版 ELF/原生包，未运行 KDE Wayland 实播、polkit 或包验收；Linux 测试调用方已同步，但 Windows 编译不验证 cfg(linux) 代码。
2. 未执行真实 NSIS 首装/同版重装/跨版本升级、取消/失败、运行中程序处理、卸载以及真实 updater 单次重启；须隔离系统/VM。页面探针与模板编译不能代替这些结果。
3. 未验证真实 HDR 输出/显示器、多屏混合 DPI、物理拖动竞态、辅助技术或低速合成器。原生属性和画面截图不证明 HDR、画质或能效优劣。
4. CI、正式签名、六项 Release 资产/匿名下载进入本轮已授权发布流程；具体结果另记下节，不把本机 debug 构建算作正式发布。后续新版本仍需其对应授权。

接手先检查工作区及[计划](plan.md#后续-agent-接手)，保留用户改动；不重审已确认设计、不继承历史验收。

## 用户实测与发布授权（2026-11-05）

用户对上述 Windows 测试程序反馈“测试通过”，并要求推送更新、合并 main 发布，随后同步 dev。未提供逐项设备结果，因此保留真实 HDR、KDE、系统安装/升级等原有未测边界。按既有 PR CI → main 自动发布 → 正式资产/匿名下载核验 → 分支同步执行，结果待补。
