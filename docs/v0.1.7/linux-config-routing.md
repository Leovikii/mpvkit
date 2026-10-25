# Linux 系统 MPV 配置接入研究

2026-10-25。用户要求研究普通系统 MPV 自动使用受管理配置与独立 MPV 两条路线。用户随后明确授权目录检测和文件安装，现已实施原生配置目录方案；当前行为见下文。不修改真实配置或默认应用，不增加历史开发版迁移。

## 建议

本机 Arch/CachyOS x86_64 + KDE Wayland 优先使用系统 `/usr/bin/mpv`，由系统包管理器维护核心；Linux 的配置及脚本直接部署到 MPV 原生配置目录 `$XDG_CONFIG_HOME/mpv`（未设置时 `~/.config/mpv`）。应用外观偏好留在 MpvKit 的用户数据根；归属/摘要、备份/恢复和稳定 helper 随配置根放入隐藏 .mpvkit，继续使用单目录事务。Windows 保持独立程序和 portable_config，不强求两平台物理目录相同。

目标体验：用户继续使用原有 MPV 打开方式、双击或终端 `mpv 文件`，下次启动就读取安装的配置。不能宣称已运行的进程立即重载，也不能覆盖调用方显式 `--no-config`、`--config-dir` 或 MPV_HOME 等设置。其他 libmpv 前端是否读取配置需另行判断，不纳入本机 MPV 范围。

## 路线比较

| 路线 | 普通 MPV 是否自动生效 | 代价与适用性 |
| --- | --- | --- |
| 系统 MPV + 原生配置目录 | 默认配置搜索下可同时读取参数、input.conf、scripts、script-opts、fonts | 推荐；需要逐文件归属与手改保护，不能覆盖用户整目录 |
| 系统 MPV + 私有目录 + MpvKit 启动入口 | 只有该入口或明确传入配置目录时生效 | 原研究方案，仍需要用户更换默认打开方式，不能满足保留原 MPV 入口的体验 |
| MpvKit 分发独立 MPV | 仍需正确配置路径或受控入口 | 增加发行来源、依赖库、更新、源码/许可材料维护；本次路径问题并不要求更换核心 |

本机 `readelf -d /usr/bin/mpv` 显示其动态依赖 FFmpeg 的 libavcodec.so.63 等、libplacebo.so.360、libass、LuaJIT、Wayland/Vulkan 及音频库。复制单个可执行文件不会固定这些库；Arch 滚动更新后仍可能需要重建。要真正独立，需要审查固定构建及运行库的分发方案，当前未选择或核验此类资产。不构建安装包。

只增加 `include` 不能作为完整接入：它读取其他 mpv.conf 的参数，不会把所有脚本、input.conf、script-opts、字体及 `~~/` 相对路径的基准整体切到那个目录。用许多绝对路径逐项补齐反而会形成两套配置。全局设置 MPV_HOME 也会引入会话传播和启动器差异；整个目录符号链接与当前路径安全规则、未知文件保护冲突，均不作为首选。

## 当前实施

检测遵循 MPV_HOME 优先，再取 XDG_CONFIG_HOME/mpv 或 ~/.config/mpv；标准目录不存在时识别原生 ~/.mpv。双目录有配置时拒绝混合部署。实际系统 MPV 的四组只读 profile 检查已验证检测路径与默认加载一致。实测相对 XDG_CONFIG_HOME 并不可靠地回退标准位置，因此检测会拒绝相对 MPV_HOME/XDG_CONFIG_HOME，避免装错目录。界面显示检测路径和来源，Linux 默认关闭额外打开方式登记。

- 新安装写入标准 MPV 目录前，预览每个将创建/修改的具体文件。已有非本应用配置或同名脚本时报告冲突，不智能合并、不替换整目录；用户未明确确认前不覆盖。
- 复用现有摘要、手改保护、互斥、备份和失败恢复；归属记录只涵盖明确管理的文件。未知文件不删除，卸载默认保留配置；系统 MPV 仍不受本应用卸载。
- 已同步配置生成、部署边界、健康检查、预览、Lua helper 和恢复。Linux Lua 使用配置根，Windows 保留父级规则；Linux 子进程不再把 MPV_HOME 改为日志目录。
- 不从当前开发版私有目录迁移实际用户文件；按现行约束，仅维护新设计并从隔离新环境测试。
- 普通系统入口无 config-dir 时的 Wayland 加载、快捷键、播放期保存、续播、截图/缩略图和九组参数已重新验证；现有部署/恢复检查已改用原生相对路径。保持 Linux 参数独立，不引入 Windows D3D11 设置。

## 依据与隔离实验

依据为本机 mpv 0.41.0 随包官方手册 `/usr/share/man/man1/mpv.1.gz` 的 FILES、ENVIRONMENT VARIABLES、CONFIGURATION FILES 和 Script location；可核对 [mpv v0.41.0 手册源码](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/mpv.rst) 与 [选项](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/options.rst)。本机 ELF 动态依赖使用 readelf 只读检查。

实验根 `/tmp/mpvkit-config-routing-research-iuor91cu`，三个全新 HOME/XDG_CONFIG_HOME/XDG_CACHE_HOME/XDG_STATE_HOME，移除测试子进程的 MPV_HOME。生成 0.2 秒静音 WAV，以系统 MPV null 音视频输出播放 0.05 秒，不连接桌面、不写真实用户目录。配置哨兵 volume=37，测试 Lua 顶层打印脚本已加载标记。

| 布局 | 实际音量哨兵 | 脚本标记 | 退出码 |
| --- | --- | --- | --- |
| 只有私有 portable_config，无启动参数 | 100（未读取） | 无 | 0 |
| 原生 XDG mpv 目录放置配置和脚本 | 37 | 有 | 0 |
| 原生 mpv.conf 仅 include 私有主配置 | 37 | 无 | 0 |

上述为方案选择前的实验。后续原生目录与真实 Wayland 完整集成已实施并记录于 [当前证据](evidence.md)。独立 MPV 分发路线仅做本机依赖分析，未下载、构建或运行独立核心。
