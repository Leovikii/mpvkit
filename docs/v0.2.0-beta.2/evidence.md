# v0.2.0-beta.2 验证证据

日期：2026-11-02。接续 dev `475070f`，用户已授权测试通过后推送 dev、PR 合并到 main 发布，并同步 dev；Arch 由 Actions CI 验证。源码版本 `0.2.0-beta.2`，固定依赖不升级，整套预设 `PRESET_REVISION=1`。范围及用户授权见[交接](plan.md)。

## 环境与隔离

Windows x64，本机固定 Node 24.19.0、pnpm 11.25.0、Mantine core/hooks 9.7.3、仓库 Rust 工具链；Edge 离线预览及真实 WebView2/MPV 回归。执行前加载 `.local/beta1-windows/env.ps1`，复用固定资产和测试工具，目录名 beta1 不代表继承旧测试结论。

- `MPVKIT_TEST_ASSETS=.local/source-research`，固定组件按下载契约检查版本/大小/SHA-256。
- `MPVKIT_CORE_SOURCE=.local/winget-research/extracted/{app}`，只复制固定 MPV 到新私有根；样本为 v0.41.0-244-gaf9c81fa1，不以包标签证明精确稳定版。
- `MPVKIT_FFMPEG=.local/v010-tests/ffmpeg/ffmpeg.exe`，合成黑/白视频，SHA-256 `7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1`。
- `MPVKIT_TEST_EXECUTABLE=src-tauri/target/debug/mpvkit.exe`，统一测试根环境；WinGet 使用本地固定 fixture，不执行真实安装器、登记、默认应用修改或宿主配置写入。
- 图形测试串行运行，日志和截图留 `.local/beta2/`、`.local/e2e-artifacts/`；测试注入只改私有副本，产品 uosc 不含探针。

## 构建与静态检查

| 检查 | 本批结果与日志 |
| --- | --- |
| `pnpm build` | 通过；`.local/beta2/frontend-build.log`；TypeScript 和 Vite 通过，现有包体 >500k 提示仍在 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | 通过 |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | 通过；`.local/beta2/clippy-final.log`；工具环境的硬链接缓存回退复制提示保留 |
| `node --test scripts/frontend.test.mjs scripts/release.test.mjs` | 19 通过；`.local/beta2/js-tests.log`，含版本/更新契约及本版 notes |
| 两端 `collect-app-notices.mjs --check` / `--linux --check` | 通过；Windows 331 包/587 材料，Linux 410/717；无缺失许可材料，日志 notices-check / notices-linux-check；仅随版本更新清单输入摘要与标题，不改许可原文 |
| Tauri debug EXE | `pnpm tauri build --debug --no-bundle --config .local/beta1-windows/build.conf.json` 通过；`.local/beta2/native-build.log`。该配置只跳过已完成的前端构建；未打包/签名/发布 |

排版修正阶段 debug EXE：`src-tauri/target/debug/mpvkit.exe`，SHA-256 `11368d9119b55078b810cba63dad49ee0d1eeff164c8a01400a7f4068a8af207`。最终构建纳入 catalog 未修改声明、滚动区域名称/region 语义、中英按钮完整行盒、Windows 字体栈、弹窗原生高度计算与“请先保存配置”的简短文案。

## Rust 与固定组件

`cargo test --manifest-path src-tauri/Cargo.toml`：**53 通过，0 失败，5 个需外部资产/图形环境的默认忽略项**，最终日志 `.local/beta2/rust-tests-final.log`。覆盖新增整套标识的元数据提交、局部更新保留标识、中断恢复、不变文件过期保护，以及重新应用的选择保留、手改还原、未知内容/续播保护与幂等；catalog/source-materials 未修改声明同步受检。

显式提供 `MPVKIT_COMPONENT_ASSETS=$env:MPVKIT_TEST_ASSETS` 后执行 `cargo test --manifest-path src-tauri/Cargo.toml fixed_archives_extract_and_deploy -- --ignored --nocapture`：**1 通过**，见 `.local/beta2/fixed-archives-native.log`；59 文件、33,245,426 字节完成固定归档校验、解析、部署与卸载。uosc main.lua 为 43,478 字节、SHA-256 `bf97dcde4f1b8bd74a5e1c672a436a5584b5db2dc9413ab5585745afd9e1dd8d`，与固定上游完全一致；两端 catalog/source-materials 均记录未修改。

普通沙箱首轮 Rust 事务/固定归档测试被 Windows 原子替换权限阻断；Edge/回环网络也受沙箱限制。相同隔离命令在获准的沙箱外执行后通过，原失败日志保留 rust-tests.log、fixed-archives.log、preview-check.log，不将环境失败隐藏为产品通过。

## UI 与播放器

离线预览：`node docs/v0.1.6/preview/build.mjs` 后 `node docs/v0.1.6/preview/check.mjs` 通过，见 `.local/beta2/preview-native.log`。验证实际 App 的两页、草稿、统一任务/失败、预设提示无自动弹窗、取消保留、重新应用保留当前选择；480/560/720、双语、深浅主题、200% 文字、减少动态效果与高对比。SVG 管理箭头实际垂直居中；ScrollArea 无横向溢出、可键盘 End 到达末尾。截图 `.local/ui-v016/beta2-scrollbar.png`，其余矩阵文件名沿用 beta1。

Windows 原生整套首轮：11 项中 10 通过，uosc 背景双击失败，见 `.local/beta2/native-full.log`。通过项包含新预设标识/取消/过期/重新应用、应用更新夹具、包管理器夹具、窗口尺寸、连播/字幕、52 项实际 MPV 选轨、安装/保存/完整卸载、单实例、两页及真实播放/续播/字体。

统一滚动条后的整套回归 `.local/beta2/native-full-final.log`：10 通过，更新弹窗因旧测试断言要求 pre 自身溢出而失败。新设计让长说明随外层滚动，故将断言移至实际 ScrollArea viewport，补键盘 End 和固定底部检查。最终 EXE 定向运行 `--test-name-pattern="native application updates|native two pages|native preset notice"`：**3 通过、0 失败**，日志 `.local/beta2/native-focused-final.log`，覆盖更新说明纯文本/整体滚动、窄窗 200%/底部按钮、下载和授权状态夹具、两页主题/键盘滚动及完整预设流程。本批 11 项原生场景经整套与定向复验均已通过，不将中途失败写成一次全绿。

uosc 定向复验揭示测试 IPC 指针与原生窗口切换后的 hover 不同步，随后还有输入区异步注册、上游仅在实际尺寸变化时刷新 scale 的测试前提问题。只在私有探针同步指针并等待输入区、实际改变视口触发缩放，不修改产品上游源码。原始失败保留 uosc-native.log、uosc-native-retry.log；最终 `.local/beta2/uosc-native-final.log` **1 通过**，取证目录 `.local/e2e-artifacts/mpvkit-v018-Q7d79Q`。验证左侧三控、三个宽度、原生菜单/倍速选择、独立随机和循环、实际 EOF、stats、控制台、全屏、缩放、重启及无 Lua 错误。

纯白测试使用实际合成 `white.mkv`，恢复产品默认 uosc scale=1 并按真实 display-hidpi-scale 断言缩放，最终 **1 通过**，日志 `.local/beta2/uosc-white-default.log`，目录 `.local/e2e-artifacts/mpvkit-v018-vaRzNE`。保留三档 `white-controls-1200.png`、`white-controls-900.png`、`white-controls-640.png` 及 `white-speed-menu.png`。白背景下原生图标有黑色描边，进度条有深色背景；倍速菜单可辨认。此为代表性画面检查，不代表所有 HDR/显示器或视觉辅助技术通过。

### 中英文标准与字形边界

按[本版排版审查](plan.md#中英文排版与边缘检查)核对微软 Typography/Content layout/Buttons 与 Mantine Button Styles API，使用固定 9.7.3 的实际实现解释问题。只调整公开 label/inner/root 样式，不改组件源文件。

离线 DOM Range 首轮 `.local/beta2/typography-before.log`：中文“检查更新/打开日志文件夹”和英文“Check for updates/Open logs folder”文字范围高 19px，而 label 约 14px，顶端超出 3px、底部 2px；带图标“管理/Manage”的 label 约 17.22px，仍有小幅超出。这是共享自动高度与字体裁剪规则不匹配，非单个英文词或箭头位置问题。修正后完整行高 21px，字形范围在 label 内；中英文普通字号实测上下约 8px 留白。测量日志 typography-after.log 保留修正中间值；最终 Windows 字体栈另由完整矩阵和原生字形测量复验。

最终离线矩阵 `.local/beta2/preview-native.log` 通过：两页中英文、480/560/720、深浅、100%/200%，检查文字在 label 内、四边至少 4px、标签中心、SVG 中心及设置行右边缘；任务弹窗额外检查双语/480/200%/深浅、底部可见、无 Modal.inner 外层原生滚动。4px 是项目阈值，非微软强制内边距。截图 `.local/ui-v016/beta2-dialog-<language>-<theme>-200.png` 及原页面矩阵；几何测量与截图共同评估，不冒充字体墨迹逐像素或正式无障碍认证。

最终 EXE 再跑应用更新、两页与预设专项：**3 通过、0 失败**，`.local/beta2/native-typography-final.log`。真实 WebView 测量中英按钮、主题和 200% 文字；更新弹窗同时断言无外层原生滚动条及底部可见。截图 `.local/e2e-artifacts/mpvkit-beta1-ui-1YLUqm/settings-chinese-200.png`、`settings-scrollbar-200.png` 和 `.local/e2e-artifacts/mpvkit-updates-zMENMS/update-dialog-480-200.png`。本次最终产品变更仅为上述 UI 修正，播放器白底与 Rust 结果不重复计数。

`mpv.exe --no-config --load-scripts=no --vo=null --ao=null --idle=yes --log-file=.local/beta2/window-policy-mpv.log --script=src-tauri/src/linux_tests/window-policy.lua`：实际 Lua 策略的确定时钟测试通过（quiet startup、live revalidation、cancellation、once-only、audio-first）。运行于 Windows，无真实 Wayland，不能计作 Linux 原生复验。

## 交互外观预览（2026-11-02）

用户授权构建并直接启动隔离预览，目录 `.local/beta2/uosc-preview`。复用固定 MPV 和原始 uosc，合成 10 秒白、黑、testsrc2 彩色图（两条静音音轨便于查看音轨按钮）；只在副本中加原生 1/2/3 切片、R quit=42 后由临时 Node 启动器重启，以及暂停控件常显。未加入 Lua、配置监听或产品新功能，未改真实配置。

预览试值 1.2/0.55 明确区别于上游 1.2/0 和产品 1/0；用户可自行编辑 `config/script-opts/uosc.conf`。`.local/beta2/check-preview.mjs` 实测 1/2/3 加载及暂停状态、R 实际换进程重载并返回白底、窗口截图和无 Lua/选项错误，结果 `uosc-preview/check.json` 为 passed；原始 main.lua 摘要仍为 `bf97dcde4f1b8bd74a5e1c672a436a5584b5db2dc9413ab5585745afd9e1dd8d`。截图 `uosc-preview/preview-white.png`。首轮沙箱内 IPC 不可达，使用获准的沙箱外相同隔离入口启动和验证后，另次只读进程核对确认预览仍在运行。

## 未测边界与后续入口

- 当前 Windows 主机未运行 Arch/KDE Wayland 原生 beta.2；Linux 测试已同步原生 uosc 行为，须在目标桌面按 [testing](../testing.md) 的 ignored/native 与 GTK/WebKit 入口复验。前批 Wayland 窗口修复证据仍只属于 beta.1 单项修复。
- 本地未执行真实 WinGet/UAC、NSIS 安装更新重启、Pacman/Polkit 或宿主软件包安装；Arch CI 隔离包验收结果见后文。未验证混合 DPI 多屏、物理拖动竞态、HDR 设备和读屏。夹具或 IPC 输入不等于这些真实交互。
- 未实施旧版迁移/旧桥接恢复，符合用户“没有旧用户”的范围；不建设独立远程预设发布。
- 现有固定组件来源/许可的完整分发审阅缺口与 `releaseReady=false` 保留。本地检查不替代正式签名验收；本批正式 CI 发布与签名结果见后文。
- 本批已获发布授权，按 [CI/CD](../ci/README.md)执行；发布结果、CI 与签名核验另记于后续证据。

## 发布前最终复验（2026-11-02）

用户确认保留上游默认描边 1.2 与 controls 背景 opacity=0；产品删除 text_border 覆盖，预览同时删除 0.55 试值。原生布局参数不变，两端 source-materials 同步实际配置大小与摘要。

- 重新构建 debug EXE 通过：`.local/beta2/release-final-build.log`，最终 SHA-256 `e0ca61a1f3869ccccd252e807f427fc8c2015bec3cc97a3711e8784c81ed7049`。
- Rust 默认测试 53 通过、5 忽略：release-final-rust.log；前端/发布契约 19 通过：release-final-js.log。格式、版本/tag、来源许可 gate、两端依赖材料 check 和本地文档链接/原始快照摘要检查通过。
- 实际 uosc 定向回归 1 通过：release-final-uosc.log，证据目录 `.local/e2e-artifacts/mpvkit-v018-EqJZLt`，包含白底与菜单/播放交互；产品部署仍是上游原文。
- 本批授权提交推送 dev、创建 PR、通过 CI 后合并发布 beta.2，再将 dev 同步到 main；Arch 构建/隔离 Pacman 载荷验收交 Actions，不视为 KDE 桌面实测。

## 公开发布核验（2026-11-02）

- 用户本批明确授权测试通过后推送、PR 合并发布和 dev 同步；Arch 交 Actions CI。本地一次性脚本已清理，日志、截图、媒体、工具链与构建产物均在忽略目录，本批未提交这些产物。
- 源码提交 `fe08c92`；[PR #16](https://github.com/Leovikii/mpvkit/pull/16) 的 [Checks](https://github.com/Leovikii/mpvkit/actions/runs/41380567424) 全部成功，包括前端、Windows/Arch fmt/clippy/test，以及 Arch makepkg、Pacman 元数据和新隔离根安装/重装/卸载载荷检查。
- 合并提交及固定标签 `v0.2.0-beta.2` 为 `5f62599bf5b822998ecd5ab30021698fda493bed`；[Release](https://github.com/Leovikii/mpvkit/actions/runs/41381291683) 双端测试、正式签名、原生包验收、资产汇总和发布成功。发布步骤使用正式公钥通过 Minisign 验证两端安装包。
- [公开 Release](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-beta.2) 从创建起 draft=false、prerelease=true、latest=false；六项资产完整，latest.json 最后上传。匿名下载全部六项，逐一匹配 GitHub API digest、SHA256SUMS 与标准 manifest 契约；本地日志/元数据位于 `.local/beta2/public-release-verification.json`，核验时间 `2026-11-02T03:22:15.750Z`。

| 资产 | 字节 | SHA-256 |
| --- | ---: | --- |
| latest.json | 1749 | `c927f3437b644189de5a8f003b3f549aa50d47faa01b4e06c5b8e57256b1dcfb` |
| mpvkit-bin-0.2.0beta2-1-x86_64.pkg.tar.zst | 6989296 | `e0619a30c78419e0e6e1435d5f1c30073e00afa51fe487e703a765719f49cefe` |
| mpvkit-bin-0.2.0beta2-1-x86_64.pkg.tar.zst.tauri.sig | 436 | `80c24de10023044612f4b6e67239e560354e58ffcf0b245a4bde037634a696b8` |
| MpvKit_0.2.0-beta.2_x64-setup.exe | 4134637 | `5e980f75468582910192b19b1d46e7d091c5bb1e976ff6d78b6b8b1fc7c35b46` |
| MpvKit_0.2.0-beta.2_x64-setup.exe.sig | 452 | `96f1fe129a11e2b95192d2ce75df3410e8c6a2878a6270949aa50baeacdf7e74` |
| SHA256SUMS | 510 | `61f40038607f88f813250e0ef4145d69d0b63b8f45e75dda065292f915f8e71f` |

Arch CI 不替代 KDE Wayland/GTK WebKit 原生操作；真实 WinGet/UAC、NSIS/Pacman/Polkit 安装及跨版本重启、多屏 HDR/辅助技术仍未验收，见上述边界。beta.1 标签保持不变。
