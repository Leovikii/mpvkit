# v0.1.3 实施验证

> 历史记录：此文记录移除迁移/兼容分支之前的实现与测试。用户已取消迁移范围；当前状态和产物以[本轮验证](2026-10-22-no-compat.md)为准。

日期：2026-10-22；Windows x64，本机隔离验证。用户批准后实施；没有提交、推送、发布或变更真实默认播放器。GitHub Actions 文件与实施前快照完全一致，没有新增或执行 CI/CD。

## 实现范围

- Windows 便携布局：根目录 mpv.exe、原构建 d3dcompiler_43.dll 与 portable_config。fonts.conf 随固定构建保留，字体在配置目录加载。
- 主配置直接编辑；GUI 只替换标记范围，非 GUI 默认参数、键位、脚本选项原样保留。手动修改状态持续显示，整文件恢复推荐配置有具体文本预览及备份。
- 原生命名 profile 与 apply-profile；菜单保存和 GUI 共用 .mpvkit/settings.json 事务。新布局无 user.conf、context/profile JSON 或独立 playback/setup JSON。
- 全局 mpvkit.json 合并应用偏好与目标选择，保留旧记录；共用锁防止互相覆盖。环境缓存和稳定助手收入 .mpvkit。
- 旧环境迁移与恢复显式预览，检查占用、登记、文件变动与复杂自定义。外部 MPV 不移动。旧 user.conf/元数据保留为非活动原件；旧续播记录保留原路径。

## 本地检查

| 检查 | 结果与范围 |
| --- | --- |
| Rust 常规测试 | 69 通过，0 失败，6 个显式真实入口默认忽略；27.52 秒 |
| 格式、Clippy | cargo fmt --check、all-targets -D warnings |
| 前端 | TypeScript、Vite 构建与双语 2 项通过 |
| 依赖许可 | 321 包 / 567 份材料，check 无缺失；没有新增依赖 |
| NSIS 目录函数 | 1 项测试内 5 种路径情形通过：新装、显式自定义、显式旧默认、已有默认和已有自定义；只运行隔离函数，不写注册表、不安装应用 |
| 原生前后端旅程 | 4 项分次全部通过；主流程 283.86 秒，最终构建补测默认目录、扩展迁移与视频 3 项共 351.00 秒 |
| 保存中断恢复 | 真实 MPV 保存被提前退出中断后，经应用恢复事务通过；归属文件一致、引导完成状态保留 |
| 构建 | Debug、Release、NSIS 本地构建通过；产物未安装、未签名、未发布 |

常规后端覆盖路径边界、覆盖保护、预览过期、跨进程竞争、中断恢复、手改/重置、统一偏好及显式迁移。新用例还核对：user.conf 在迁移预览后改变会拒绝执行；路径型选项拒绝自动迁移；非活动旧记录不影响新布局；恢复主配置时不清除仍被保留的键位手改提示。

原生覆盖：新安装与隔离登记、主配置手改保留/冲突取消/明确替换、损坏标记的重置及恢复、直接启动/受管理启动一致、GUI 关闭后的原生档位保存、默认路径与高级选项、旧 config 布局迁移回退、旧平铺外部 MPV 迁移回退、外部程序摘要保持、最小键位与菜单优先级、位置续播且不恢复音量/速度、BGRA 缩略图、PNG 含字幕/JPEG 不含字幕、文楷加载与 SRT 渲染。截图已人工查看，配置差异弹窗可滚动，状态页明确提示手动修改。

主流程通过后只刷新了本地测试命令对应的依赖清单输入摘要，并重新构建 Debug/Release/NSIS；功能代码保持一致。最终 Debug 通过默认目录、扩展迁移和真实播放补测，依赖清单复核通过。原始主流程与最终构建摘要分别保存在[机器可读记录](2026-10-22-implementation.json)，不冒称所有用例均在最终二进制上整批重跑。

原生测试使用固定四组件并复核 SHA-256；Debug 环境在任何界面写入前检查隔离根。系统登记使用隔离后端，未修改开发者真实注册表或 UserChoice。媒体由固定摘要的 FFmpeg 生成，不使用私人媒体。

## 修复与复测要点

- MPV 原生命令成功可以返回 nil,nil，Lua 按错误返回值判断，避免把成功切换误判后还原。
- 布局迁移及恢复执行校验使用源布局，修复回退时被“布局已切换”错误拒绝的问题。
- NSIS 使用原始 Windows 命令行保留 /D，静默模式切换目录后同步输出目录；隔离测试按 NSIS 规定传递未加引号的末尾 /D 参数。
- 原生测试等待保存助手明确结束，不能看到 settings.json 写入就把多文件事务判为完成；实际提前退出产生的中断环境已额外验证恢复。
- 续播验证识别媒体进度记录，并允许 MPV 原生的目录重定向条目；不假定目录中首个文件必含 start。键位测试等待异步全屏状态真正变化，避免在窗口尚未完成切换时误报。

## 复现入口

~~~powershell
pnpm build
pnpm test:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
node scripts/collect-app-notices.mjs --check
# 历史命令：nsis-directory.test.mjs 已在 v0.1.8 接手清理中删除，不再执行
# node --test scripts/nsis-directory.test.mjs
pnpm tauri build --debug --no-bundle
# MPVKIT_TEST_ASSETS 指向固定四组件；MPVKIT_FFMPEG 指向测试内核对摘要的媒体生成器
pnpm test:e2e
pnpm tauri build --bundles nsis
~~~

上述均为手动本地入口，不是 Actions / 自动发布流程。历史 native-v012 脚本保留供旧版复核，当前入口为 native-v013。构建输出的本地化 MSVC 链接器信息会被 Rust 显示为 linker_messages 警告；构建退出码为 0，Clippy 检查通过。

## 保留的边界

- 实体 60 Hz/高刷切屏、HDR、功耗、旧版 Windows、干净系统安装/卸载未验收；没有以合成 SDR 样本代替设备矩阵。
- 字体测试证明配置目录字体加载与普通 SRT 字形渲染，不保证所有 ASS 样式、缺失字体组合与字符都已覆盖。
- 安装器测试覆盖路径选择函数与 NSIS 编译，未执行完整安装/升级/卸载。
- MPV 聚合构建与 uosc 完整来源/许可适用审查延续既有缺口，releaseReady=false。
- 复杂旧配置不会自动迁移。仅支持明确列出的常见单值参数；合入文本位于 GUI 标记之外，仍可能覆盖 GUI 写入值，界面持续提示手改。旧原件/记录不自动删除。
- 保存、迁移或引导确认属于文件事务；系统终止或播放器提前退出中断保存时，需要从维护页恢复。最近一次恢复不等同任意历史版本库。

目录与操作约定见[计划](../plan.md)和[实际目录](../storage-layout.md)。

## 证据与交付状态

- 主流程：.local/e2e-artifacts/mpvkit-v013-wzrx9Z，含配置差异和状态页截图。
- 视频：.local/e2e-artifacts/mpvkit-v013-player-2z4g1e，含按键绑定、播放器日志、PNG/JPEG 和结果记录。日志确认 fontselect 将 LXGW WenKai Screen 解析为 LXGWWenKaiScreen。
- 扩展旧平铺外部环境：隔离目录 mpvkit-v013-migration-VJ9Fcm；详细结果在 .local/v013-native-supplement.log。
- 中断保存恢复：.local/v013-interrupted-save-recovery.json；未更改实际用户安装。
- 本机运行的测试应用与播放器已退出。文档链接、代码格式、Git diff 及第三方文档/工作流快照核对通过。用户已有 v0.1.2 未提交改动保留。

本轮批准的实现与可执行本地验证完成；下一任务为用户复查体验和条件具备后的设备/干净系统/来源验收。未提交、推送或发布。
