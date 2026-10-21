# 2026-10-20 / M1.4 本地 MPV 发现验证

## 本轮范围与环境

未提交工作区，补齐 M1.4，并复用已有 M1 查询/启动链路。新增 detect.rs、discover_mpv 命令与概览页候选列表，未新增依赖或升级工具链。原有 [启动证据](2026-10-20-m1.md) 的 11 项结果属于前一批变更；本轮共 17 项。

Windows 11 企业版 LTSC x64 build 26200；Node 24.19.0、pnpm 11.25.0、Rust/cargo 1.98.1、MSVC Build Tools 2022、WebView2 155.0.4283.62。播放器仍为专用模拟 mpv.exe，无真实媒体、驱动、HDR 或功耗结果。

## 行为与边界

- Known Folder API 取 LocalAppData、ProgramFiles、Profile 路径，使用 KF_FLAG_DONT_VERIFY，不让该 API 验证重定向目录。检查四个固定候选：LocalAppData/MpvKit/runtime/mpv.exe、LocalAppData/Programs/mpv/mpv.exe、ProgramFiles/mpv/mpv.exe、Profile/scoop/apps/mpv/current/mpv.exe。
- 另检查本应用程序同级 mpv.exe，以及 PATH 前 64 项下的 mpv.exe；不枚举盘符、目录或子目录，空 PATH 不作为工作目录。
- 访问候选文件元数据前执行本地路径/reparse 校验；缺文件是正常未发现，不可用/不安全位置记录原因并继续。
- ASCII 大小写重复路径合并位置标签。标签仅描述在哪个配置位置发现，不证明发布者或管理归属；包括 runtime 位置在内的候选都不自动取得所有权。
- 查找不执行程序，不改变当前输入或已生成预览。用户点击“使用此路径”才更新输入并使旧预览失效；点击“检查并预览”才查询版本/参数。

## 自动化结果

运行以下命令成功：

~~~powershell
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
pnpm tauri build --debug --no-bundle
~~~

17 项 Rust 测试全部通过，0 失败、0 忽略。新增 6 项检查：

| 检查 | 实际结果 |
| --- | --- |
| 只读、不执行、不递归 | 无效可执行文件内容仍被列为候选；内容与目录项数不变；不会从上级目录查出嵌套 mpv.exe |
| 路径重复 | 同一路径的 ASCII 大小写变体只返回一次，合并位置标签 |
| 错误后继续 | UNC 与相对路径被跳过，后续合法本地文件仍返回 |
| 未发现与非文件 | 不存在的路径正常忽略，目录候选显示 candidate_unavailable |
| PATH 上限 | 70 项仅生成前 64 个位置，并标记截断，顺序不变 |
| 引号、中文和空项 | 带引号中文空格路径保持完整，空项不搜索工作目录 |

首次检查中发现 KF_FLAG_DONT_VERIFY 的绑定类型为 i32，而 API 接收 u32；显式转换后 clippy 与测试通过。临时编辑脚本的字符串转义错误已修正；未忽略检查失败。

## 实际桌面流程

构建后的 mpvkit.exe 使用内置页面，以隐藏窗口运行；临时启用仅本地 WebView2 调试端口 9229，使用运行时自带 Playwright 连接本次测试进程。只对测试进程设置 PATH，不修改系统/用户 PATH。

在测试 PATH 前部放入：模拟程序目录、其 ASCII 大小写变体、不可达 UNC 文本、本地测试 junction，再接原 PATH。执行：

1. 先手动填写一个路径，再点击“查找本地 MPV”。返回唯一模拟程序候选，合并 PATH 第 1/2 项位置标签；手动输入保持原样。
2. 展开跳过项：UNC 与实际本地 junction（指向本仓库的模拟目录）均返回 local_path_required，未追随链接。
3. 选择该候选，再点击“检查并预览”。实际 Tauri IPC 成功返回 mpv 0.40.0-fixture 的版本/参数检查结果及启动预览。
4. 重新查找保留原预览；再次选择路径使旧预览消失。窄至 640 CSS px 时无横向溢出。
5. 未确认启动，不创建真实 MPV 配置或应用会话。测试结束后仅结束本次程序；调试端口关闭。

已目视检查 .local/discovery-desktop.png，路径换行、候选按钮、跳过原因和手动表单可读。一次性脚本/截图位于忽略目录，不新增浏览器测试依赖。

## 仍未验证及后续入口

- Scoop current 若为 junction 会被跳过；自定义 Scoop 根、非标准安装位置、PATH 第 65 项之后等由手动入口处理。当前不跟随 reparse point，不声称覆盖所有安装方式。
- 仅按 ASCII 大小写合并路径，不合并短文件名/硬链接/全部 Unicode 大小写别名；不会据此进行删除或更新。
- 位置数量有界；未对任意本地设备驱动的同步文件 I/O 提供硬性墙钟期限。网络和链接路径先拒绝，不把阻塞远程访问包装成后台任务。
- 此次实际 junction 验证只证明只读发现的拒绝路径，不证明部署写入具有句柄级竞态保护；M2 的互斥、哈希、重解析点写入边界与恢复仍未实现。
- 未证明真实 MPV 可播放、加载隔离配置或在 GUI 关闭后继续播放；M1.5/1.6 保持进行中。M0 的固定来源研究另见 [来源报告](2026-10-20-m0-sources.md)。
- 未更改默认播放器、注册表、磁盘映射或真实 MPV 配置，未提交推送发布；所有现有未提交改动保留。

## 最终交付检查

最终文案明确“版本未检查；位置标签不证明可信来源”，随后重新执行 pnpm tauri build --debug --no-bundle，包含类型检查的前端生产构建与 Windows exe 重建均成功。最终 cargo fmt --check、git diff --check 无错误；脚本检查 35 份 UTF-8 文本的 LF/空白/JSON/Markdown 围栏及 31 个本地链接通过。LICENSE、.gitattributes、工具链和双锁文件未修改，现有文件仍保留原有未跟踪/未提交状态。
