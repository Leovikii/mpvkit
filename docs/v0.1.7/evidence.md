# v0.1.7 本机验证记录

2026-10-25。源码基线为远端 v0.1.6 `13939c1`，当前 v0.1.7。仅本机 CachyOS（Arch 系）x86_64、KDE Wayland/Vulkan。用户已授权系统 MPV 配置目录检测与安装实现，并随后授权提交、推送本批更新至 GitHub。没有 CI/CD、安装包或开发版迁移；所有部署验证均使用新隔离目录，不改变真实默认应用与配置。GitHub 推送不等于正式发布。

## 当前实现与检测

Linux 系统核心固定 `/usr/bin/mpv`。读取本进程非空 MPV_HOME，或绝对 XDG_CONFIG_HOME/mpv、HOME/.config/mpv；标准目录不存在时识别 MPV 原生 HOME/.mpv。后者是系统 MPV 的加载规则，不是对 MpvKit 旧格式的兼容。不扫描磁盘，不读取此前开发版私有安装目录。

相对 MPV_HOME/XDG_CONFIG_HOME、符号链接、非目录和根目录被拒绝。标准目录与 ~/.mpv 同时存在实际配置时报告歧义，不混合安装。四个新隔离 HOME 的真实 `mpv --show-profile` 检查已证实默认、XDG 覆盖、MPV_HOME 优先和原生 ~/.mpv 的检测结果确实被系统 MPV 默认加载，未传 config-dir。

实测 MPV 对相对 XDG_CONFIG_HOME 不可靠地回退 ~/.config，因此没有按通用 XDG 规则猜测目标，而是阻止相对覆盖路径。另一个实验确认同时存在两个用户目录时 MPV 会合并读取，检测据此阻断。拒绝路径、空值、优先级、重复目录、已有普通文件与符号链接都有单元断言。

配置和增强直接部署到检测根，归属、设置、锁、备份、恢复和稳定 helper 放在同根 .mpvkit 中，继续使用既有单目录事务。应用外观偏好仍在数据根。Linux Lua 直接以配置目录定位保存记录/helper；Windows 继续使用 portable_config 父目录。Linux helper 启动不再把 MPV_HOME 改成日志目录。Linux 默认不额外登记打开方式；可选登记继续单独受保护。

本机只读环境：HOME=/home/viki，MPV_HOME/XDG_CONFIG_HOME 未设置，且无 ~/.mpv，检测目标为 `/home/viki/.config/mpv`。本轮未向该真实目标安装文件；最后检查仍没有真实 mpv.conf。此前开发版实际私有目录保留，不迁移或删除。

## 环境与产物

- Rust 1.98.1（48a229cea）、Node 24.19.0、pnpm 11.25.0；未升级工具链和依赖。
- 系统 MPV 包 `1:0.41.0-6.1`、GTK `1:3.24.52-2.1`、WebKitGTK `2.54.2-1`。参数和路径语义依据为该 MPV 包的 `/usr/share/man/man1/mpv.1.gz`。
- `pnpm tauri build --debug --no-bundle` 通过；bundle.active=false，无安装包。
- 当前 ELF：`src-tauri/target/debug/mpvkit`，270640480 字节，SHA-256 `d4d1b8d8632e5f790666ec2c93ec35c03aadf8d1ae15045601af0cb2dcc0859d`。x86-64 动态链接调试程序，仍依赖本机 GTK/WebKitGTK 等库。
- 系统 MPV SHA-256 `390d76705f0af028775a140b4b51a61bd47fc77e8e39039354e14186ace65076`，真实集成前后核对不变。手册压缩文件 SHA-256 `a0469788efade27fbd0923ae5879f8e15013ea4f06ef7355d12b4c1933a8ab86`。

## 本地检查

| 检查 | 结果 |
| --- | --- |
| TypeScript + Vite / 单 ELF 构建 | 通过，869 模块 |
| 前端语言优先级与双语完整性 | 2 项通过 |
| cargo fmt --check | 通过 |
| cargo test --locked --offline | 45 项通过，3 项显式忽略 |
| cargo clippy --locked --offline --all-targets -- -D warnings | 通过 |
| 两平台许可 collector --check / --linux --check | Windows 321 包/567 份材料、Linux 358 包/620 份材料，无缺失；不等于完整许可审阅 |
| 显式 Wayland 集成 | 通过，使用添加播放信息快捷键前的同版 ELF，见下文 |
| 原生窗口启动存活 | 通过，隔离窗口运行至 8 秒 timeout，退出码 124 |

默认忽略项为在线固定 HTTPS 下载、真实 KDE 会话与真实 Wayland 集成。Wayland 本轮显式通过；KDE 系统偏好在本版此前已显式验证，相关代码未变。下载使用逐字节核验的固定本地资产，本轮未重新验收在线链路。

共享部署检查已使用平台实际相对路径，覆盖未知文件、覆盖保护、备份、失败恢复、过期计划、手改、互斥、脚本冲突与设置优先级。远端 v0.1.6 的只读草稿预览、完整组件健康、fingerprint 和保留文件摘要保护仍在 Linux 运行。新增检测逻辑没有历史格式读取、迁移或路径映射分支。

## 普通系统 MPV 的真实 Wayland 集成

最终根 `/tmp/mpvkit-v017-native-config-02/config/mpv`，命令与固定资产准备见 [计划](plan.md)。测试使用构建 ELF、固定 uosc ZIP/thumbfast Lua/文楷、合成 15 秒 1280×720 H.264 视频（SHA-256 `c6b03028e3f7a15bea3f048dff3bee68df831762fbdbd7ae11e1291e0a3980f1`）。

本次 `arch_wayland_install_playback_and_uninstall` 23.07 秒通过，断言包括：

1. 无安装记录时仍显示系统 MPV 实际状态/版本。首次预览只读生成 Linux 文本；组件缺失时不能应用。安装后移走 ziggy-linux 也会阻止保存，恢复后按 fingerprint 正常保存。
2. 默认关闭额外登记，配置就绪后没有 desktop entry。直接按系统 MPV desktop entry 的模式运行 `/usr/bin/mpv --player-operation-mode=pseudo-gui`；仅给测试子进程设置隔离 HOME/XDG 目录，没有 config-dir 或 MPV_HOME 覆盖，没有经由 MpvKit launcher。实际加载配置和 uosc，输出 gpu-next/waylandvk，本次 hwdec-current=vulkan，1280×720。
3. 实际 UP 键音量 50→60；Lua 状态与 uosc 动态键位存在。切换高画质后 scale=ewa_lanczossharp；播放中的全局保存成功，验证原生配置根下 helper/归属定位有效，且不要求额外登记。
4. 实际生成 PNG 截图和 202800 字节 thumbfast BGRA 缩略图。输出快照见 [机器记录](playback-evidence.json)。没有字幕样本，不声称已验收实际字幕字体呈现或画质优劣。
5. 跳至 4 秒退出后生成 watch_later。随后显式启用可选登记，desktop-file-validate 通过；再通过真实稳定 helper 启动，续播位置至少 4 秒、高画质保留、音量为默认 100。普通系统入口与受管理入口使用相同文件。
6. 三档画质 × 三种 HDR 的九组配置均按草稿预览/健康/fingerprint 流程保存，由本机 Wayland MPV 各渲染两帧成功。
7. 卸载后配置字节、watch_later 与预置的未知 notes.txt 保留；helper 和隔离登记移除；系统 MPV 安装状态、版本、文件摘要不变。

驱动输出一次 `radv is not a conformant Vulkan implementation, testing use only`，测试仍成功；不据此声称实体 HDR、画质或正式驱动认证。首次普通启动的“无 config-dir”契约和保存/续播在本次实际执行，不以先前私有目录结果代替。

添加播放信息快捷键前的 ELF 原生窗口在 `/tmp/mpvkit-v017-native-window-01` 用 Wayland 启动并存活至 8 秒 timeout。GTK 有 `Can't connect to a11y bus ... Connection refused` 告警，未屏蔽。这仅为启动检查，未完成原生点击或辅助技术验收。

## 此前诊断与仍保留的修复

用户此前确认双击媒体，实际默认关联是 mpv.desktop，其 Exec 没有私有 config-dir。旧私有配置可通过明确 config-dir 读取，但普通 MPV 找不到高画质 profile；该诊断促成本轮原生目录设计。现已移除“必须把 MpvKit 设为默认”的提示，显示检测目录与来源。真实默认关联未修改，旧数据未迁移。

本版此前 KDE 系统偏好复查确认接口可用：plasma-localerc Formats/LANG=zh_CN.UTF-8，portal color-scheme=1，而 GTK 名称 Breeze 不含 Dark。现优先 KDE 语言/portal 明暗，GTK 回退。显式测试在进程内 GTK 偏好反向时仍读到 zh-CN/dark；没有改桌面设置。用户手动 zh-Hans/dark 偏好保留，选择“跟随系统”才启用跟随。系统 MPV 状态独立于增强记录的修复亦保留。

此前 Unix 锁回归修复采用 guard 结束显式 unlock，避免并发 spawn 临时继承描述符延长 flock；确定性测试继续通过。远端 v0.1.6 独立预览仍为 Windows 历史设计，不当作 Linux 原生 GUI 验收。

## 共用播放信息快捷键（2026-10-25）

resources/input.conf 加入 i → stats/display-stats、I（Shift+i）→ stats/display-stats-toggle。Windows 与 Linux 的生成器无条件嵌入同一源文件；v0.1.6 独立预览亦直接导入此文件，因此无需另写平台副本。现有用户 input.conf 保留，不自动追加。

本次 cargo test config:: 的 5 项测试通过（包含手改保留）；TypeScript/Vite、单 ELF 重建通过。隔离根 /tmp/mpvkit-stats-preset-o732_590 中使用系统 MPV，禁用默认内置键位并显式加载预设，通过 Unix IPC 读取实际 input-bindings，核对 i/I 分别为预期命令。测试未改真实配置，不把按键表读取等同实际窗口信息面板视觉验收。

前述真实 Wayland 集成和窗口存活检查使用添加这两行快捷键之前的 ELF（SHA-256 `185f820b72263cd178ff7324bee297dcf93ee252937bcf8bc6098f12182279cf`）；本次不重复九组画质/HDR 验收，当前产物摘要已更新。Windows 原生交互仍待 Windows 实机验证。

## 限制与下一任务

- Windows 原生构建和 Win32/注册表回归本轮未运行；平台代码及材料解析保留，不冒称 Windows 实机通过。
- 真实 GUI 全流程、选择器、键盘/屏幕阅读器、不同 DPI、长时间真实媒体、多轨字幕/音频和字幕实际字体尚未完整验收。
- 实体高刷、HDR 显示效果、功耗与画质比较、干净系统及完整来源许可仍有缺口，releaseReady=false。
- 检测以启动本应用的进程环境与 MPV 原生用户目录规则为准；其他调用方显式 no-config/config-dir、不同 MPV_HOME 或 libmpv 自行禁用配置不受本应用控制。已有播放进程不自动重载。
- 不支持符号链接配置目录、重复用户配置目录并行、ARM、X11、其他发行版/桌面。系统强调色暂固定蓝色。

下一任务是在隔离原生配置目录中完成 GUI 安装/预览/保存/冲突操作验收，再补真实多轨媒体和字幕/HDR 设备证据。不得迁移、删除当前实际旧数据或运行历史安装器任务。
