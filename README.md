# MpvKit

MpvKit 是面向桌面用户的 MPV 安装与配置管理器，帮助用户安装播放器、选择适合设备的播放设置、管理组件，并在更新或卸载时保留可恢复的配置。

- 软件名称：**MpvKit**。
- 仓库：[Leovikii/mpvkit](https://github.com/Leovikii/mpvkit)。
- 当前实现：**v0.1.1，Windows x64**；四步安装引导、独立管理页面、双语与原生窗口外观。
- 后续平台方向：Linux；v0.1.1 继续聚焦 Windows，Linux 版本号与日期另定。
- 技术栈：Tauri 2、React、TypeScript、Mantine 9.7.3、Rust，必要的播放期逻辑使用 Lua。

## 当前状态

2026-10-21：v0.1.1 已实现“安装位置 → 安装组件 → 播放设置 → 系统集成”的首次引导，完成后进入状态页。播放器与播放增强分组；设置统一管理简中/英文、主题、强调色及关于信息；窗口自动使用 Mica 或纯色回退。图标采用用户提供的 MPV SVG，无边框。前端继续使用 Mantine，后端复用可恢复部署与文件保护。

新功能、验证与已知边界见 [实施计划](docs/v0.1.1/plan.md)、[界面说明](docs/v0.1.1/ui-flow.md)、[架构与迁移](docs/v0.1.1/architecture.md)及[验证要求](docs/v0.1.1/verification.md)。当前为开发测试版，完整公开发布条件仍未满足。下文 v0.1.0 验证记录保留历史范围，不代表新版本全部验收。

截至 2026-10-20，已实现固定组件获取/校验/安全解包、可恢复部署和维护、结构化设置与三档、Windows设备只读检测、受管理启动、uosc/快捷键切换、GUI关闭后的全局保存、打开方式登记与恢复、脱敏诊断和日志清理。首版仍未完成全部验收，用户确认暂时只有当前电脑，保留隔离系统与实体设备验收缺口；上游完整许可/源码材料仍待补齐。

受管理启动与诊断在查询前校验播放器归属及已记录运行文件摘要。最近完整业务回归62通过（0失败/0忽略，含全部6个真实入口，297.77秒），四样本×三档的12份图形探针已核对；[完整证据](docs/v0.1.0/evidence/2026-10-20-current-regression.md)保留范围。后续补充Go来源声明，普通后端56项及单独真实归档通过，未重跑另外5个真实入口或原生交互；[本轮材料与产物](docs/v0.1.0/evidence/2026-10-20-go-sunsoft-notice.md)记录边界。此前Mantine三类原生用例分次通过；应用许可321包/567份材料无缺失原文项。调试/Release/NSIS及八资源核对通过，当时安装包3863996字节、未签名，未安装或发布。完整来源/适用、目标设备/HDR/功耗和干净系统验收仍未完成；详见[实施计划](docs/v0.1.0/plan.md)与[F01–F11台账](docs/v0.1.0/acceptance.md)。

## 本地开发

当前仅支持 Windows x64。准备 Node **24.19.0**、pnpm **11.25.0**、Rust **1.98.1**（仓库的 rust-toolchain.toml 会固定版本）、MSVC C++ 构建工具、Windows SDK 与 WebView2 Runtime。首次安装依赖需联网；包管理器和 Rust 必须能在终端 PATH 中找到。

~~~powershell
pnpm install --frozen-lockfile
pnpm tauri dev
~~~

仅看网页布局可运行 `pnpm dev`；网页模式不能检查文件或启动本机程序。桌面引导在“高级选项”中可使用已有 MPV 并点击“查找本地 MPV”，也可填写完整路径（不加引号）。查找不执行程序、不替换当前输入；点击候选的“使用此路径”后，再检查版本。版本检查会运行所选的 `mpv.exe`，请只选择可信的本地程序。

查找仅检查 Known Folder 对应的 `MpvKit/runtime`、用户 `Programs/mpv`、`Program Files/mpv`、用户 Scoop 默认位置、MpvKit 程序同级位置，以及 PATH 前 64 项中的 `mpv.exe`；不递归、忽略空 PATH 项，拒绝网络路径和 reparse point，错误位置跳过后继续。ASCII 大小写重复路径合并位置标签。位置标签不证明发布来源或归属，所有候选仍按外部播放器处理；版本与能力在用户选择后检查。Scoop 的 `current` 若为 junction，当前会跳过，可手动指定实际版本目录；自定义安装位置仍可手动填写。

检查和构建：

~~~powershell
pnpm build
pnpm test:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
pnpm tauri build --debug --no-bundle
pnpm test:e2e:smoke
~~~

调试构建产物为 `src-tauri/target/debug/mpvkit.exe`，含内置页面，可独立启动；NSIS 用户级安装包使用 `pnpm tauri build --bundles nsis`；本地调试与Release安装包构建通过，真实安装/卸载尚未在干净系统验收。缺少 WebView2 时安装器从 Microsoft 下载引导程序，需联网。Rust 测试编译专用模拟播放器并只操作临时目录，不需要已安装 MPV，也不改真实配置、默认应用或注册表。Windows CI 使用相同入口，远程 CI 运行结果尚待推送后确认。

手动外部启动使用 `--no-config`、显式 `--config-dir` 和 `--load-scripts=no`，采用原生默认值；勾选受管理环境后加载已校验的配置、uosc、thumbfast 与 MpvKit Lua。所选配置目录的可选 user.conf 最后加载，保留用户覆盖。预览不写文件；确认后在 Windows Known Folder API 返回的 LocalAppData 下创建 `MpvKit/cache/sessions/<会话 ID>`，保存有 schemaVersion 的归属记录和最多三行事件日志。日志最多1024份，在维护页预览清理旧日志并保留最近20份；会话归属、缓存、用户文件和备份保留。外部 MPV 始终归用户所有。

真实 MPV 的隔离验收入口已在固定候选上通过；启用条件、命令及证据边界见 [M1 实机验收](docs/v0.1.0/verification.md#m1-显式实机验收入口)。它需要显式设置隔离路径；用户要求自动测试时可默认使用仓库隔离目录。普通 Rust 测试仍默认忽略该入口，原生冒烟测试不需要 MPV；完整旅程使用真实固定组件与可信外部播放器。


完整原生旅程需先准备 resources/catalog.json 中的三份固定资产（后端仍核对版本/哈希），再设置隔离测试输入：

~~~powershell
$env:MPVKIT_TEST_ASSETS = 'D:\fixtures\mpvkit-assets'
$env:MPVKIT_E2E_MPV = 'D:\fixtures\mpv\mpv.exe'
pnpm test:e2e
~~~

资产文件名为 mpv-x86_64-20261020-git-6a262d11f6.7z、uosc-5.13.0.zip、thumbfast-thumbfast.lua。只使用 Debug 构建；测试在任何界面写入前核对隔离根，Release 不支持此隔离入口。CI 运行无资产依赖的外观、键盘、目录选择及进程清理冒烟检查，不代表完整播放/安装验收。旧 v0.1.0 界面测试已由当前旅程取代，历史证据保留。

构建产物、下载缓存、截图和本机证据保存在忽略目录；Git 仅提交源码、锁文件、文档和构建必需资源，不提交 EXE/安装包。维护工具和第三方许可材料仍用于来源复核，不能当作临时文件删除。

### 直接依赖用途

| 依赖 | 用途 |
| --- | --- |
| React / React DOM | 页面与表单状态 |
| @mantine/core / @mantine/hooks 9.7.3 | 按用户要求实现统一控件、主题、布局和可访问性；hooks为core必需的同版本peer依赖 |
| @tauri-apps/api / tauri | 受控业务 IPC 与 Windows 桌面窗口 |
| @tauri-apps/cli / tauri-build | 开发、嵌入页面和构建 Windows 资源 |
| TypeScript / @types/react / @types/react-dom | 前端类型检查 |
| Vite | 本地开发服务及生产页面打包；无需额外 React 插件 |
| reqwest 0.13.5 | 固定 HTTPS 来源、限制重定向和超时的组件获取 |
| sevenz-rust2 0.24.1 | 校验固定哈希后读取 MPV 7z，逐条路径/属性/展开规模检查 |
| zip 8.6.0 | uosc ZIP 解析与 CRC 校验；不用自写归档解析 |
| sha2 0.10.9 | 组件/文件 SHA-256、预览状态和备份完整性（复用已有锁定版本） |
| Playwright 1.62.1（开发） | 连接测试 WebView2，验证原生前端与后端 IPC；不下载额外浏览器 |
| serde / serde_json | 命令数据、结构化错误及会话记录序列化 |
| windows-sys | 本地用户目录、磁盘类型、有界子进程管道读取及测试进程句柄核对 |

可选的第三方来源维护工具使用Python标准库及隔离的Go1.23.1 SDK（仅检查uosc的ziggy），见[重建与运行时元数据证据](docs/v0.1.0/evidence/2026-10-20-uosc-link-metadata.md)。这些工具不参与MpvKit常规构建或运行，不要求用户安装Go；产品后端仍为Rust。

前端开发必须遵守[前端规范](docs/v0.1.0/frontend.md)及已下载的[Mantine官方LLM文档索引](docs/vendor/mantine/llms.txt)，按实际使用组件阅读对应页面。前端与 Rust 解析结果分别固定在 pnpm-lock.yaml 和 src-tauri/Cargo.lock。应用依赖许可收集入口为 node scripts/collect-app-notices.mjs --check，范围与未完成项见[许可证据](docs/v0.1.0/evidence/2026-10-20-app-notices.md)。应用不内嵌上游组件二进制；固定来源、版本/哈希与未完成发布门槛记录在 resources/catalog.json；来源研究下载保存在忽略目录 `.local/source-research`，调试测试可显式读取并重新校验这些候选；生产获取仅允许代码固定的来源；项目授权已由用户确认为 GPL-3.0-or-later，声明与构建元数据已同步。

## v0.1.0 的目标体验

检测环境 → 查看推荐及理由 → 调整设置 → 预览变更 → 安装部署 → 验证 → 更新、恢复或卸载。

首版重点是可靠部署 MPV、uosc 和 thumbfast，提供内置画质档位、播放中切换及全局记忆，完成 Windows 文件关联和可恢复的维护流程。外部着色器经过验证后按需加入。

MpvKit 管理独立的 MPV 播放器，不嵌入 libmpv，不重做播放窗口。管理界面关闭后，播放功能继续工作。

## 开发文档

后续 AI agent 从 [AGENTS.md](AGENTS.md) 开始，再依次阅读以下文档。

| 文档 | 用途 |
| --- | --- |
| [版本范围](docs/v0.1.0/scope.md) | 用户场景、必做功能、延期范围与交付边界 |
| [前端规范与官方LLM文档](docs/v0.1.0/frontend.md) | Mantine组件、样式、可访问性和验证要求；前端开发必须遵守 |
| [架构与数据约定](docs/v0.1.0/architecture.md) | 模块职责、配置所有权、部署恢复、平台边界 |
| [实施计划与交接](docs/v0.1.0/plan.md) | 里程碑、任务依赖、待决事项和当前接手位置 |
| [验证与发布](docs/v0.1.0/verification.md) | 自动化测试、人工播放验证、发布门槛 |
| [开发候选版本说明](docs/v0.1.0/release-notes.md) | 当前能力、环境矩阵、联网要求和未完成验收 |
| [来源与配置决策](docs/v0.1.0/sources.md) | 官方依据、社区项目参考方式、组件来源规范 |

## 开发原则

优先复用 MPV 和组件上游已有能力，只维护 MpvKit 必须提供的少量逻辑。一个应用、一个 Rust crate、少量平台模块；不提前建立插件框架、后台服务或通用规则引擎。

配置语义以目标版本的 MPV 官方文档与实现为依据，组件优先来自各自上游；社区整合项目提供集成经验，推荐效果由目标设备验证。

## 许可证

MpvKit 采用 **GPL-3.0-or-later**（GNU GPL 第3版或任何以后版本）授权，见[项目许可声明](COPYING.md)和[完整许可证正文](LICENSE)。项目声明随安装包分发；第三方组件、依赖、配置片段、字体和着色器分别保留其适用许可证及署名。

## 播放快捷键与维护边界

受管理启动中，Alt+1/2/3 切换省电/均衡/高画质，F6 打开 uosc 档位菜单，Alt+s 保存当前档位为全局默认；保存使用短时 Rust 进程，GUI 可关闭。只更改经实测的缩放/HDR映射参数，解码/输出和字幕/界面调整下次启动生效；无外部 shader。

全局偏好按安装代际保存，新 GUI 设置使旧代际偏好失效；用户附加配置最后加载。受管理文件手工改变会阻止覆盖/启动，应通过备份恢复或将自定义放到 user.conf。系统打开方式通过 `mpvkit.exe --open` 使用同一受管理启动逻辑，不修改 UserChoice。移除环境或自身卸载前先移除打开方式登记；本机测试使用隔离文件后端与原生私有蜂巢，不表示系统默认应用或资源管理器已验证。
