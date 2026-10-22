# v0.1.0 验证与发布

> 历史文档：当前行为与执行约束见[开发基线](../current.md)及 [AGENTS.md](../../AGENTS.md)。旧布局、版本迁移、安装包和流水线任务不再执行；仅按需追溯。

状态：最近完整业务回归62项及此前三类分次原生通过；后续Go声明补充已跑56普通项与单独真实归档，另外5显式入口未重复。范围见[本轮证据](evidence/2026-10-20-go-sunsoft-notice.md)及[完整回归](evidence/2026-10-20-current-regression.md)；[F01–F11台账](acceptance.md)中的设备/系统/完整来源验收仍未完成。更新日期：2026-10-20。

## 验证原则

测试针对数据丢失、错误配置、错误平台假设和实际播放行为。纯文档变更检查链接、格式、Git diff 和内容一致性，不运行或虚构应用测试。

M1 建立代码后，在 README 中记录真实命令。Rust 至少具备格式检查、静态检查与相关单元/集成测试；前端具备类型检查和生产构建。纯业务逻辑可直接测，文件操作使用临时目录，不依赖用户实际 MPV 环境。

命令行配置检查和 Lua 静态检查不能证明图形输出、HDR、脚本接口或缩略图可用。

## 自动化和故障测试

| 组 | 场景 | 预期结果 |
| --- | --- | --- |
| A | 未获取 GPU/HDR/电源信息 | 返回未知和理由，保守推荐且允许手选 |
| A | 不同能力/版本下生成配置 | 不生成已知不支持项；未支持部分明确说明 |
| A | 重新检测和切换推荐 | 字段级用户覆盖保留；单项恢复仅清除该覆盖 |
| A | 中文、空格及特殊字符媒体路径 | argv 传递正确，不变成 shell 指令 |
| A | 发现候选含远程/不可用位置、重复位置和空 PATH | 不执行候选、不递归；跳过后继续，位置标签合并，保留手动选择；超出 PATH 上限如实提示 |
| D | 哈希错误、损坏包、下载中断 | 旧安装不变，错误可理解，暂存可清理 |
| D | 归档路径穿越、绝对路径、链接越界 | 拒绝写出目标目录，保护外部文件 |
| D | 目标目录含 junction/reparse point | 写入/删除不得越过预期管理边界 |
| D | 重复安装相同版本 | 无重复配置、无累积 shader、结果一致 |
| D | 预览后文件被修改 | 执行被拒绝并重新预览，不覆盖新修改 |
| D | 备份/替换/校验阶段失败 | 恢复旧状态或明确保留待恢复记录 |
| D | 应用中途异常退出 | 下次启动识别未完成操作并可恢复 |
| D | 取消、磁盘不足、权限拒绝、文件占用 | 不报告假成功，不强杀播放器，不丢备份 |
| D | 两个管理进程同时修改目标 | 串行或拒绝第二个操作，不交叉写入 |
| D | 外部 MPV 与用户改过的受管理文件 | 更新/卸载不误删，给出保留或恢复选择 |
| D | 无效或更新版本的状态格式 | 不覆盖原文件，提示兼容问题 |
| R | A→B→A 档位/增强切换 | 实际参数可回到原方案，无遗留和重复链 |
| R | 播放列表从动画切到真人/下一文件 | 本次临时选择正确重置 |
| R | 全局记忆后重启 MPV | 正确加载已保存选择 |
| R | 多实例与 GUI 同时更新偏好 | 无损坏、无无关字段丢失，失败明确反馈 |
| R | 保存时崩溃、组件被移除 | 状态可读取；缺失增强回退且说明原因 |

D 组在临时目录与模拟下载源运行；恢复关键路径使用故障注入。HTTP、文件系统或注册表抽象只在确实需要测试边界时引入，不构造巨大的 mock 框架。

## M1 显式实机验收入口

默认 cargo test/CI 只运行模拟程序。真实入口位于 src-tauri/src/launch/real_tests.rs，需要显式设置本地测试根目录和可信 mpv.exe；根目录需已存在。用户要求自动测试时，agent 可默认使用仓库 .local/v010-tests 或新建临时子目录及已固定并校验的候选，不再重复等待路径；不得据此操作真实配置或注册表。它只在根目录内新建唯一子目录，不复用已有配置，失败保留记录，不下载或运行安装脚本。

在选择隔离路径后运行下列命令；示例路径需要替换。环境变量仅用于当前终端，示例会恢复原值。单项验收使用 --exact --ignored。只有下文全部显式环境已配置且授权运行全部隔离测试时，才使用 --include-ignored；缺少条件不应绕过入口守卫。

~~~powershell
$taskPreviousMpv = $env:MPVKIT_REAL_MPV
$taskPreviousRoot = $env:MPVKIT_REAL_TEST_ROOT
try {
    $env:MPVKIT_REAL_MPV = 'C:\可信播放器\mpv.exe'
    $env:MPVKIT_REAL_TEST_ROOT = 'C:\已明确指定的测试目录'
    cargo test --manifest-path src-tauri/Cargo.toml --locked launch::real_tests::real_mpv_contract -- --exact --ignored --nocapture
} finally {
    $env:MPVKIT_REAL_MPV = $taskPreviousMpv
    $env:MPVKIT_REAL_TEST_ROOT = $taskPreviousRoot
}
~~~

入口复制选定 exe 及存在时的 d3dcompiler_43.dll，生成中文路径的 30 秒静音 PCM WAV，设置 portable_config/MPV_HOME/显式配置的不同 volume 哨兵和自动脚本正向控制。短探针复用生产 argv，使用 null 输出；以子进程 MPV_HOME 覆盖缓存/状态目录；再由短生命周期测试启动者调用生产 prepare/execute，检查其退出后播放器存续至少 2 秒，并核对进程路径后结束本次进程。只支持该最小运行文件集，其他构建缺少 DLL 时如实失败。

保留记录包括完整 --version/--list-options、探针输出、启动结果和 schemaVersion=1 的 passed.json。只有全部断言通过才生成通过记录。普通回归的 verificationMode=fixture 不得用作实机证据；即使 verificationMode=real-player，也只覆盖合成音频及启动者退出，不能代替真实 Tauri GUI 关闭、图形视频、HDR 或功耗验收。操作步骤和当前结果见 [本轮证据](evidence/2026-10-20-m1-validation.md)。

## 原生前后端自动交互

先运行 pnpm tauri build --debug --no-bundle，再运行 pnpm test:e2e。固定 Playwright 仅连接本次 WebView2；默认用自有模拟程序，需要 rustc。设置 MPVKIT_E2E_MPV 为已校验可信 mpv.exe 可运行真实播放器模式。两种模式均新建临时数据目录，调试构建仅在 MPVKIT_TEST_MODE=1 且显式 MPVKIT_TEST_ROOT 时使用测试根；发布构建不提供此覆盖。

覆盖维护预览/取消/过期/冲突/移除/恢复、真实 IPC、发现与启动、GUI 正常关闭后进程存续；保存本地截图和通过记录。结束时核对进程映像并只结束本次播放器，不默认上传日志。当前结果见 [证据](evidence/2026-10-20-m2-core.md)。

## Windows 集成验证（W）

在虚拟机、专用测试账户或用户明确指定的环境操作真实系统集成：

- 用户级安装，不依赖管理员启动。
- 注册“打开方式”，默认应用按钮打开正确系统入口。
- 从资源管理器打开中文/空格路径文件，确认加载受管理配置。
- 更新 MPV 后，关联和快捷方式仍指向正确入口。
- 另一应用修改关联注册值后，卸载 MpvKit 不覆盖对方修改。
- GUI 关闭后正常播放，uosc、快捷键及全局保存继续工作。
- 自身安装包卸载与 MPV 环境卸载分别验证，保留用户资料。
- 多显示器、高 DPI 和键盘操作可用。
- 缺少所需 WebView2 时有可执行的安装引导或安装包处理，并明确联网要求。

不能以注册表写入成功代替默认应用已由用户设置成功。

## 播放矩阵（P）

至少覆盖两个目标设备类别；可以用用户提供的设备，但不能假定当前开发机代表二者。

| 设备/输出 | 内容 | 关注点 |
| --- | --- | --- |
| 4K 台式机 | 1080p 动画、真人电影/视频 | 放大、颗粒/线条、掉帧、字幕、预览 |
| 4K 台式机 | 4K SDR | 原尺寸播放、硬解、无无效超分 |
| 已确认 HDR 输出链 | 4K HDR | 亮度映射、色彩、字幕、输出状态 |
| SDR 输出链 | 4K HDR | HDR→SDR 映射，不能仅凭“画面出现”判断正确 |
| 1440p 核显本 | 1080p 动画、真人 | 均衡与省电档、缩略图负担 |
| 1440p 核显本 | 4K SDR/HDR | 下采样、解码路径、掉帧及映射 |
| 多显示器/窗口 | 同一片源 | DPI、窗口缩放、换屏、输出能力变化 |

硬件解码记录实际生效方式，不能只记录配置请求。推荐算法不按显示器分辨率盲目放大字幕或加载增强。未知 HDR 信息不能被报告为已成功 HDR 输出。

外部 shader 如收录，额外验证放大/原尺寸/缩小三种关系、支持后端、HDR 限制、开启/关闭对比和缺失文件回退。

## 性能和功耗

固定设备、片源、输出尺寸、亮度、刷新率、音量、HDR 状态、电源方案和解码路径，预热后重复测试。记录测试时长、版本、掉帧、渲染时间及测量方法。

比较前先确认设备对测试片源的基础播放能力；不能把不支持该编码的硬件归咎于档位，也不能把源文件自身的时间戳异常隐藏掉。

功耗需要整机功耗或可靠电池放电数据，GPU 占用率/渲染耗时不能单独证明省电。无可靠设备时标记“功耗未验证”，不宣传节能百分比。画质要包含正常观看距离的动态片段，静态截图仅作补充。

## 来源、维护与发布检查（L）

- [ ] F01–F11 完成对应验证并有报告。
- [ ] Windows 支持版本、架构、WebView2 及联网要求明确。
- [ ] MPV、uosc、thumbfast 固定版本；完整资产、依赖和许可可追溯。
- [ ] 配置没有已知废弃/未知选项，日志无未解释的关键错误。
- [ ] 失败恢复、更新、卸载和用户文件保护经过验证。
- [ ] 组件清单更新页面准确表达其检查范围，不假称联网查到最新。
- [ ] 用户附加配置与手工修改不因更新丢失。
- [ ] 诊断导出处理私密信息，日志有容量或保留上限，无默认上传。
- [ ] 安装包在干净环境运行；代码签名是否存在如实说明。
- [ ] 第三方声明、适用源码/构建材料和修改记录齐备。
- [x] [开发候选版本说明](release-notes.md)列出未支持的平台、未验证能力和已知问题；不是已发布声明。
- [ ] 构建/格式/静态检查与相关测试通过，准确记录跳过项。
- [ ] 发布内容无下载缓存、个人媒体、凭据、用户配置或不明二进制。
- [ ] README 与实施计划反映真实状态；发布行为已获相应授权。

## 证据记录

报告放入本目录的 evidence/，按日期和任务命名；只有真实执行后才创建报告，不创建空报告或假测试结果。每份报告至少记录：

- 任务/需求 ID、提交或未提交变更说明、日期。
- 系统、设备、驱动、播放器及组件版本。
- 测试媒体的编码、分辨率、帧率、HDR 类型；不提交私人媒体。
- 操作步骤、预期、实际结果、相关日志摘要。
- 失败与跳过项、限制、下一步。

文档内链接和 Markdown 格式校验可用临时脚本执行，不需要为本次文档增加长期依赖。

## M2 固定组件自动验收

- `cargo test --manifest-path src-tauri/Cargo.toml --locked` 默认不下载或运行上游组件。
- 设置 MPVKIT_COMPONENT_ASSETS 为已校验本地研究目录，单独运行 `components::tests::fixed_archives_extract_and_deploy -- --exact --ignored --nocapture`，验证真实 ZIP/7z 及临时目录部署/卸载。
- 设置 MPVKIT_TEST_NETWORK=1，单独运行 `components::tests::fixed_https_downloads -- --exact --ignored --nocapture`，使用生产 HTTPS/大小/哈希路径，不执行内容。
- 原生测试设置 MPVKIT_TEST_ASSETS 为固定研究目录时增加获取取消、完整安装预览/取消/执行、卸载和恢复。只有调试构建且 MPVKIT_TEST_MODE=1 才接受此目录覆盖，逐资产仍要求固定哈希；发布构建忽略它。

## 完整本地隔离回归

全部六个显式入口分别验证真实启动契约、固定归档部署、固定HTTPS下载、720p三档图形组件、私有注册表蜂巢和合成媒体矩阵。普通测试不需要这些条件；真实图形测试会短时打开播放器窗口。

先用已校验FFmpeg生成三份合成媒体；输出目录必须不存在，脚本不下载工具、不覆盖已有媒体。固定工具身份见脚本，不能任意替换FFmpeg后跳过哈希。

~~~powershell
node scripts/make-media-fixtures.mjs 'C:\隔离工具\ffmpeg.exe' 'C:\新的隔离媒体目录'
# 原720p图形入口使用单独样本；仅在已有隔离媒体目录中新建，-n禁止覆盖。
& 'C:\隔离工具\ffmpeg.exe' -hide_banner -nostdin -n -f lavfi -i 'testsrc2=size=1280x720:rate=24' -t 10 -an -c:v libx264 -pix_fmt yuv420p -color_primaries bt709 -color_trc bt709 -colorspace bt709 'C:\隔离媒体\synthetic-sdr.mp4'
~~~

生成1080p H.264 8位SDR、4K HEVC 10位SDR、4K HEVC 10位PQ标签样本，各8秒/24fps。矩阵逐样本核对哈希、尺寸、输入色彩标签，三档分别验证gpu-next、真实BGRA缩略图、uosc启动及配置/Lua错误，并保留实际硬解/掉帧/输出尺寸。PQ标签测试图没有经过色彩准确性制作，不能据此验收HDR亮度或色彩；1920×1080播放窗口不等于4K实体输出。

在独立终端配置全部条件后运行（下列路径替换为本地隔离目标）：

~~~powershell
$env:MPVKIT_COMPONENT_ASSETS = 'C:\固定组件研究目录'
$env:MPVKIT_TEST_VIDEO = 'C:\隔离媒体\synthetic-sdr.mp4'
$env:MPVKIT_REAL_TEST_ROOT = 'C:\已存在的隔离测试目录'
$env:MPVKIT_REAL_MPV = 'C:\已校验播放器\mpv.exe'
$env:MPVKIT_MEDIA_MATRIX = 'C:\新的隔离媒体目录\matrix.json'
$env:MPVKIT_TEST_NETWORK = '1'
cargo test --manifest-path src-tauri/Cargo.toml --locked -- --include-ignored --test-threads=1
$env:MPVKIT_TEST_ASSETS = $env:MPVKIT_COMPONENT_ASSETS
pnpm tauri build --debug --no-bundle
pnpm test:e2e
~~~

私有蜂巢入口使用系统offreg.dll创建独立蜂巢文件、仅为该蜂巢设置调用者访问权限，再以RegLoadAppKey进程私有句柄运行生产Win32注册表API；不写HKCU，不修改UserChoice、不挂载全局注册表路径。仅测试构建包含该能力，系统DLL不存在时入口如实失败。覆盖登记、预览过期、外部修改保护、失败恢复及重新加载；资源管理器/默认应用界面仍需隔离Windows验收。

原生交互在显式MPVKIT_TEST_ASSETS存在时运行两组：完整管理流程与指定中文/空格配置目录。覆盖下载取消、事务/设置/关联/诊断/清理、GUI关闭后Lua保存、播放列表临时档位重置、旧代际保存拒绝、两个真实MPV竞争保存和真实--open重启。默认没有固定资产时不运行完整真实组件组，输出中的skip必须保留，不可报告为全部交互验收。

## 固定uosc辅助程序只读核对

在固定ZIP已校验、仅取其明确Windows条目的前提下，运行 node scripts/inspect-uosc-buildinfo.mjs <ziggy-windows.exe>。脚本先核对固定大小/SHA-256，再读取Go内联构建信息，与source-materials.json逐项比较；不执行二进制，不支持任意Go程序。当前结果与分发材料边界见[证据](evidence/2026-10-20-uosc-helper-materials.md)。强化真实归档入口同时验证实际helper摘要、许可原文部署字节及卸载移除。

## 部分写入故障回归

普通后端回归新增3项线程局部故障注入，实际写入前缀后模拟Windows空间/权限错误，核对新文件清理、已有目标/上一成功保护、持续提交/回滚错误后的恢复及首次归属失败重试。使用 cargo test --manifest-path src-tauri/Cargo.toml --locked partial_ 可单独运行；不填满磁盘或改变ACL，不能替代物理硬盘/掉电测试。[本轮证据](evidence/2026-10-20-partial-writes.md)记录范围和限制。

## Mantine前端验证

前端必须遵守[前端规范](frontend.md)和下载的官方LLM文档。迁移使用既有两组原生完整IPC流程，新增空格切换受管理选项、程序路径随之禁用/恢复、媒体required以及前端未处理异常断言；既有640×600/150%无横向溢出断言继续执行。暂存清理预览在调试版会逐文件哈希多个真实大组件副本，等待采用与执行清理一致的90秒上限，不改变冲突/归属断言或业务超时。每轮失败保留页面和日志，实际结果见[迁移证据](evidence/2026-10-20-mantine.md)。

## 固定uosc与Go源码只读核对

可选开发入口需要Python3.11或以后版本，只使用标准库。按[固定归档清单](evidence/2026-10-20-uosc-source-archives.json)准备五份本地源码文件，再运行 python -X utf8 scripts/verify-uosc-sources.py .local/source-research 和 python -X utf8 scripts/verify-uosc-sources.test.py。入口不下载/解包落盘或执行内容；核对官方Go摘要、实际模块h1、完整uosc Git blob/模式和现有许可镜像，缺归档直接失败。Go独立声明文件名枚举不替代逐文件版权/链接适用或二进制可重建证明；[实际结果与边界](evidence/2026-10-20-uosc-source-archives.md)保留发布缺口。

## uosc图标字体原文件比较

在固定uosc ZIP或已验证源码归档中取得明确的uosc_icons.otf，按source-materials.json的固定Google提交准备官方字体与LICENSE，再运行 node scripts/verify-uosc-icons.mjs <uosc图标字体> <Google原字体> <Google原LICENSE>。入口只读核对各文件大小/SHA-256和逐字节相等，不安装字体；[字体证据](evidence/2026-10-20-uosc-icon-font.md)记录正确输入、三个同长度篡改拒绝，以及真实归档部署/卸载入口中的字体摘要断言。此结果只关闭图标字体映射，不适用于uosc_textures.ttf或完整分发审阅。

## 固定Go额外声明核对

可选开发命令为 python -X utf8 scripts/collect-go-notices.py .local/source-research/go1.23.1.src.tar.gz；默认只读，核对官方归档、完整来源文件、已审阅字节范围、输出及材料清单。python -X utf8 scripts/collect-go-notices.test.py .local/source-research/go1.23.1.src.tar.gz 验证2项测试/4类拒绝用例，使用隔离临时目录。--write只生成唯一声明合集，改变审阅范围时还需同步审阅source-materials材料摘要。不能把44来源条目或check通过宣称为完整实际链接/兼容性审阅；[本轮记录](evidence/2026-10-20-go-source-notices.md)保留限制。

## 固定uosc helper离线重建

可选维护入口要求Windows与Python3.11或以后版本。按[SDK清单](evidence/2026-10-20-uosc-go-toolchain.json)和[五源码清单](evidence/2026-10-20-uosc-source-archives.json)准备本地输入，并从已校验uosc发布ZIP取明确的ziggy-windows.exe，运行 python -X utf8 scripts/rebuild-uosc-helper.test.py 和 python -X utf8 scripts/rebuild-uosc-helper.py .local/source-research。入口先核对全部摘要/h1/Git blob，解包到新目录，关闭Go模块/自动工具链在线获取，使用原始go.mod与参数数组执行固定官方编译器；不执行上游脚本或helper，不做系统级Go安装。报告保留实际选中文件对应性、内嵌构建比较、原始二进制差异和离线版本元数据警告；成功退出表示实验及报告完成，只有reproducibleBinary=true才表示本次摘要相同，当前结果为false。4项拒绝风险测试及实际158包/952文件记录见[证据](evidence/2026-10-20-uosc-rebuild.md)。不以重建包图替代原始helper全部链接或完整许可审阅。

## 上游helper运行时元数据只读比较

可选维护入口 scripts/compare-uosc-links.go 使用上一轮固定便携GoSDK的debug/pe与debug/gosym，读取两个固定输入及已校验重建/选中文件报告；只输出JSON，不执行helper。按[复验步骤](evidence/2026-10-20-uosc-link-metadata.md)运行gofmt、go vet、3项具名测试和真实PE比较，三类完整入口拒绝已通过。5902函数元数据和583项规范化路径一致，仅证明表中对应关系；代码/数据节仍不同，不能替代机器指令等价、原始完整源码对应性、实际播放覆盖或完整许可审阅。产品前后端测试沿用此前证据，本轮没有修改应用输入或重跑整套交互。

## 受管理查询前完整性与测试收尾

普通后端新增 launch::tests::managed_runtime_is_verified_before_launch_and_diagnostic_probes，使用fixture实际写入的probes.txt验证完好程序会查询，而exe/DLL变化、DLL缺失、exe无归属时启动及诊断均不执行查询。可单项运行 cargo test --manifest-path src-tauri/Cargo.toml --locked launch::tests::managed_runtime_is_verified_before_launch_and_diagnostic_probes -- --exact；新增后的普通56项通过；后续[当前完整回归](evidence/2026-10-20-current-regression.md)已补齐62项（含全部6个显式入口），0失败/0忽略。

完整原生主流程增加真实部署DLL单字节变更后的诊断/启动拒绝、只读预览和原字节恢复。大型恢复预览确认/指定目录组件预览采用90秒上限，指定目录整组300秒；不改变业务时限、摘要或拒绝断言。测试收尾先结束自己的应用再断开CDP，按已创建的精确测试映像处理无started PID记录的播放器。新增同名不同路径双进程回归；三类原生用例各次实际结果、失败修正及边界见[证据](evidence/2026-10-20-runtime-probe-integrity.md)。

## 同包相同代码的MIT原文映射

按[固定材料清单](evidence/2026-10-20-scroll-bar-license.json)准备六份本地输入，运行 python -X utf8 scripts/verify-scroll-bar-license.py .local/source-research/scroll-bar-mapping 和 python -X utf8 scripts/verify-scroll-bar-license.test.py。入口核对固定摘要、两npm SHA-512、26发布文件/30源码文件相等、唯一许可新增、清单精确差异和补充原文；5项风险测试含代码变化、额外文件、来源变化、同长度篡改与归档边界。只读，不执行上游脚本。通过仅证明此同代码授权映射，不宣称2.3.8自身gitHead可访问或完整来源审阅完成；[实际结果](evidence/2026-10-20-scroll-bar-license.md)说明后端/原生沿用范围。

选中Go输入声明的后续范围已扩为33独立文件+13选段（46条/90469字节），仍使用上述collect-go-notices/check及现有2项/4类拒绝测试；[补充证据](evidence/2026-10-20-go-selected-notices.md)记录AES及指数汇编原文、完整旧前缀保留和真实部署字节断言，不改变完整适用审阅门槛。

当前Go声明范围为33独立声明+14选段，47条/90968字节；新增SunSoft署名，旧90469字节不变。最终56普通及单独真实归档通过，沙箱限制和LF规范前后记录见[本轮证据](evidence/2026-10-20-go-sunsoft-notice.md)；未重跑其余五个显式入口，不扩大此前62项完整回归范围。
