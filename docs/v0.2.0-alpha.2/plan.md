# v0.2.0-alpha.2 交接

更新：2026-10-27。第二个 alpha 预览版，不升为稳定版。用户授权本批实现、测试、推送 dev、同一 PR 合并 main 并同步 dev，测试直接公开 Release；已有 alpha.1 草稿也需公开。

## 变更

- 官方 `tauri-plugin-single-instance =2.5.4`（已核对最新稳定版），先于 updater 注册。重复启动恢复/显示/聚焦主窗口；不启用 semver 隔离，因此版本间共用一个 GUI。Linux 无会话总线时明确失败，避免插件无互斥回退；`--save-playback` 仍在 GUI 初始化前独立执行。
- KDE 双重分类来自 desktop 同时声明 AudioVideo 和 Settings；改为仅 AudioVideo。新包升级后生效，不直接改主机系统 desktop 文件。
- main 缺 tag 时自动构建并从创建时公开 Release；已有 tag 跳过。资产齐备才推广更新索引；历史 alpha.1 是唯一允许转公开的既有草稿。
- 用户明确删除多余门槛：移除 `releaseReady` 布尔状态阻断及旧阶段限制，保留许可事实和缺失原文/测试/签名/摘要/版本/渠道验证。没有把未完成审阅改成通过。
- README 英文说明实际 pacman 安装方式。压缩当前 agent/开发/发布入口，历史原始证据与许可材料保留；AUR 暂不接入。

## 已验证

| 检查 | 结果 |
| --- | --- |
| Linux Rust | 47 passed / 4 ignored；fmt、clippy all-targets -D warnings 通过 |
| 前端和发布契约 | TypeScript/Vite 构建通过；Node 25/25（含前端、CI 分类、公开创建、不可变资产、材料缺失仍拒绝） |
| Linux 单实例原生 | 私有会话总线，全新根；两轮各六个并发重复启动退出 0，无新增 Wayland 窗口，原 GUI 保留；退出后可重启；helper 不被拦截；缺总线 GUI 拒绝、helper 仍处理 |
| 打包入口/工作流 | desktop-file-validate、实际 Arch staging 分类断言和 actionlint 通过（未安装 shellcheck） |
| 依赖材料 | Linux 393 packages / 690 materials；Windows 331 / 587；missingMaterials 均为空，两端 --check 通过 |

原生命令 `python3 scripts/test-single-instance.py`，结果 `/tmp/mpvkit-single-instance-i462dn87/result.json`；构建/测试日志 `.local/linux-v020-alpha2/`。新增依赖用于跨平台互斥与已有窗口激活，固定锁文件及两端许可原文；既有工具链不升级。

## 待完成与边界

[PR #5](https://github.com/Leovikii/mpvkit/pull/5) 已合并，dev/main 同步于 e2727bb；[双平台 CI](https://github.com/Leovikii/mpvkit/actions/runs/40520587280) 全通过（Arch 5m1s、Windows 6m55s）。自动创建 alpha.2 tag 精确指向该合并提交，[CD](https://github.com/Leovikii/mpvkit/actions/runs/40521873592) 构建中。

历史 alpha.1 已由[复用资产流程](https://github.com/Leovikii/mpvkit/actions/runs/40521938107)公开，draft=false / prerelease=true，10 个资产保留。初始化 updates Release 时返回 403：新元数据 tag 的目标由旧版本 tag 改为 main，避免 GITHUB_TOKEN 隐式标记旧工作流提交；无需 PAT。后续 alpha.2 创建版本 Release 返回 422，确认 target_commitish 只接受分支/SHA；统一改 main。新增 build_run 复用完整 Actions 产物的发布重试，不重编译。Windows CI 一次在 PowerShell 创建测试 junction 时超时；仅重跑失败任务后通过，安全断言未改。最终公开结果待实查。

本轮未重跑 Windows 原生激活、完整播放矩阵或真实安装升级；这些代码未改，基础结果见 [alpha.1](../v0.2.0-alpha.1/plan.md) 和[证据](../v0.2.0-alpha.1/evidence.md)。Wayland 聚焦受合成器策略约束；WebKit 子进程和播放期 helper 不算重复管理器 GUI。跨用户/桌面会话不互锁。许可完整审阅仍未完成，记录不变。真实 NSIS 跨版本安装、polkit/物理输入/HDR 不冒称通过。

实际 Arch release 包已提取核验：版本 0.2.0alpha2-1，desktop 仅 AudioVideo；同一单实例脚本对包内程序复验通过，结果 `/tmp/mpvkit-single-instance-v6ud57k5/result.json`。namcap 只有未 strip、间接共享库与 xdg-utils 静态使用识别警告，无错误。
