# 2026-10-20 / M1 隔离验收入口

## 范围与结果

本轮落实 M1.5/1.6 的可重复验收代码，不改变播放器启动参数、GUI 或部署范围。当前所有变更仍未提交。真实播放器测试尚未执行：M0.4 要求用户明确指定测试目录，本轮尚未收到目录及真实播放器运行选择，因此不运行已下载的 MPV 候选。

本轮实际结果为 **18 项 Rust 测试通过，0 失败，1 项实机测试默认忽略**。模拟程序只用于验证验收入口自身，不能证明 PCM 解码、配置加载或真实 GUI 关闭后的播放。

## 代码与保护

- 新增 src-tauri/src/launch/real_tests.rs，实机入口为 launch::real_tests::real_mpv_contract；默认 cargo test 和 CI 不会运行它。需要显式设置 MPVKIT_REAL_MPV 与 MPVKIT_REAL_TEST_ROOT，并指定 --ignored。
- 复用生产 prepare、execute、launch_args 和有界进程输出收集；不在应用中暴露测试 command。子进程环境单独设置 MPV_HOME，不改变测试宿主或全局环境。
- 先检查本地绝对路径、网络/reparse 边界和输入文件；测试根目录必须已存在。在其中独占创建唯一子目录，复制选定 mpv.exe 和存在时的 d3dcompiler_43.dll，不复制原配置、不执行安装脚本。其他构建需要的附加 DLL 不自动猜测复制，缺失时测试失败。
- 生成 30 秒、8 kHz、单声道、16 位 PCM 静音 WAV（中文/空格/&/$ 文件名），不使用私人媒体。
- 在本次目录内建立 portable_config、MPV_HOME 和显式配置目录，各有不同 volume 哨兵。配置与自动脚本均有正向控制，再对生产 no-config/load-scripts=no 契约检查未加载状态。短探针使用 null 音视频输出，显式指定测试 cache/state 目录，5 秒超时。
- 通过另一个短生命周期 Rust 测试进程调用生产启动；该启动者退出后检查 MPV 是否继续存在至少 2 秒。Windows 打开进程句柄并核对可执行程序路径，只有核对成功的本次进程才可结束；正常结束还检查退出等待结果。
- 真实入口拒绝带 fixture 的模拟版本。记录保留在本次唯一目录：完整版本和选项输出、配置/脚本正向控制、隔离输出、启动者结果和通过记录。记录有 schemaVersion 和 verificationMode，模拟回归不写成真实解码或隔离通过。
- 新的普通回归测试使用已有模拟播放器，实际覆盖子进程再进入、正向控制、生产启动、父进程退出、句柄路径核对与清理。不增加 npm/Rust 包或升级版本；windows-sys 仅增加 Threading feature，用于测试进程句柄操作。

## 已执行验证

环境沿用固定 Node 24.19.0、pnpm 11.25.0、Rust 1.98.1 与 Windows x64 开发环境。执行入口：

~~~powershell
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
pnpm tauri build --debug --no-bundle
~~~

前端类型/生产构建、Rust 格式/静态检查、18 项普通测试和完整 Tauri 调试构建通过。实机测试默认忽略是预期行为，不计入通过数。文本格式、Markdown 本地链接及 git diff --check 通过。普通沙箱仍受不可达远程卷影响；补丁工具未返回后已终止，改用经批准的本地命令通道，没有探测远程盘或修改映射。

另外只读查询固定 MPV 提交的官方 options.rst，核对 no-config、config-dir、load-scripts、term-playing-msg、length 与 audio-display 的语义。见 [来源决策](../sources.md#m1-002--显式实机验收入口)。这属于文档核对，不是实际构建能力验证。

## 仍未验证与后续入口

- 实机入口本身尚未在真实 MPV 上运行；仅已编译并用模拟程序检验执行流程。实际输出格式、脚本控制、音频设备可用性仍可能使实机验收失败。
- null 输出探针只计划验证合成音频解码和配置契约。生命周期阶段使用生产原生输出；需要可用的实际音频设备，失败如实保留。启动者是测试进程，不能代替真实 Tauri 窗口关闭验收。
- 未验证图形视频输出、硬解、SDR/HDR、两类目标设备、uosc/thumbfast、缩略图、功耗或系统集成。
- 失败时保留本次测试目录。已取得并验证的播放器句柄会尝试清理；若启动后尚未取得句柄即失败，播放器最多播放合成 WAV 的 30 秒。目录不递归自动删除。
- MPV/uosc 完整分发许可材料仍未补齐，尚无正式 catalog.json；M2 未开始。

下一任务：收到用户明确指定的测试目录与可信播放器选择后，按 [实机入口](../verification.md#m1-显式实机验收入口) 运行并检查保留记录，再补真实 GUI 关闭和视频播放证据；补齐来源许可后进入 M2。
