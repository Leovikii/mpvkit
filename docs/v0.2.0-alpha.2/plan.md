# v0.2.0-alpha.2 交接

更新：2026-10-27。第二个 alpha 预览版，已公开；0.2.0 稳定版尚未发布。用户授权本批实现、测试、dev→main PR、同步分支和公开历史 alpha.1。

## 已实现

- 固定官方最新稳定 `tauri-plugin-single-instance =2.5.4`，先于 updater 注册；重复启动恢复/显示/聚焦主窗口，各版本共用一个实例。Linux 缺会话总线时明确失败，避免失去互斥；`--save-playback` 在 GUI 初始化前独立执行。新增用途、锁文件及两端许可材料齐备，既有工具链未升级。
- KDE 双重分类来自 desktop 同时声明 AudioVideo/Settings；新包仅 AudioVideo，升级包后生效，不直接改主机系统入口。
- main 无版本 tag 时自动创建、构建和公开；REST 从创建起 `draft=false`，完整资产上传后才推广渠道。已有 tag 跳过，不移动 tag/覆盖资产。历史 alpha.1 草稿已公开，新版本不采用草稿。
- 删除 `releaseReady` 人工状态及旧阶段的发布阻断；许可审阅事实仍保留。版本/tag、测试、缺失许可原文、签名、SHA-256、资产和渠道契约检查继续执行。
- README 英文说明当前 pacman 安装；AUR 暂不接入。相关开发入口压缩约 44%，历史证据和第三方原文保留。

## 验证与公开结果

| 范围 | 实际结果 |
| --- | --- |
| Linux 本地 | Rust 47 passed / 4 ignored，fmt/clippy all-targets -D warnings、TypeScript/Vite、Node 25/25、desktop-file-validate、actionlint 通过（未安装 shellcheck） |
| 单实例原生 | debug 与实际 Arch 包内 release 程序均通过：两轮各 6 个并发重复启动退出 0，无新增窗口，原 GUI 保留；退出后可重启；helper 不被拦截；缺总线 GUI 拒绝而 helper 仍处理 |
| 实际 Arch 包 | 0.2.0alpha2-1，desktop 仅 AudioVideo；namcap 只有未 strip、间接共享库与 xdg-utils 静态识别警告，无错误 |
| 材料一致性 | Linux 393 packages / 690 materials，Windows 331 / 587；missingMaterials 均为空，两端 --check 通过 |
| 远端检查 | [PR #5 CI](https://github.com/Leovikii/mpvkit/actions/runs/40520587280)、[发布修复 CI](https://github.com/Leovikii/mpvkit/actions/runs/40525260860) 双平台全通过 |
| 发布 | [成功任务](https://github.com/Leovikii/mpvkit/actions/runs/40525904761) 复用[首次完整构建](https://github.com/Leovikii/mpvkit/actions/runs/40521873592)的 publish-assets，不重编译；[alpha.2](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-alpha.2) 与 [alpha.1](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-alpha.1) 均 draft=false / prerelease=true，各 10 个资产 |
| 公开更新 | [channels.json](https://github.com/Leovikii/mpvkit/releases/download/updates/channels.json) 和版本元数据匿名 HTTP 200；preview=alpha.2、stable=null、archPackage=null，alpha.1 比较可发现新预览版 |
| 实际应用请求 | 新 XDG 根与私有总线，直接访问公开渠道：alpha.2 收预览返回 current、不收预览返回 noRelease，均无错误；未调用安装器 |
| 公开资产 | 全部 9 项 SHA-256 及正式公钥 Minisign/版本注释独立验证通过 |

版本 tag 固定于 e2727bb7ebea0c6f7d4db7f5f224b2693a0ae6b4，不随发布脚本修复移动。[PR #5](https://github.com/Leovikii/mpvkit/pull/5) 和 [PR #6](https://github.com/Leovikii/mpvkit/pull/6) 已合并并同步 dev；[后续 main 版本检测](https://github.com/Leovikii/mpvkit/actions/runs/40525892932) 确认已有 tag 时正常跳过构建。

本地日志与公开资产 `.local/linux-v020-alpha2/`；单实例结果 `/tmp/mpvkit-single-instance-i462dn87/result.json`、实际包 `/tmp/mpvkit-single-instance-v6ud57k5/result.json`；公开原生请求 `/tmp/mpvkit-public-update-hLumAd/result.json`。Windows EXE SHA-256 `121f98a6980a64eff3f85eb362f9ddb02c6b8994ae988c7646d9ee4948d1445c`；Arch 包 `6964c175f437ce0fca8c08735e576c9227688b4b5ed03c7228fbbde756ad00ab`。

## 初次失败与修复

alpha.1 已成功转公开，但首次 updates 初始化返回 403；alpha.2 首次创建版本 Release 返回 422，明确为 target_commitish 无效。该 API 只接受分支/SHA，已统一使用 main；版本 tag 预先存在且不变。新增可选 build_run，从完整 Actions 产物重试发布，仅授予 actions:read，无 PAT。修复后实际公开与渠道初始化成功。

Windows CI 曾在 PowerShell 创建测试 junction 时超过 10 秒；错误来自共用进程函数的 probe_timeout，并非 MPV 查询或路径安全断言失败。只重跑失败任务后通过，安全测试未删改。独立验签先需解码 Tauri 的 base64 公钥；WebKit 取证需等待异步值落定；均为本地验证器修正，未据此修改产品。

## 未验证边界

本轮未重跑 Windows 原生窗口激活、真实 NSIS 跨版本安装、完整播放矩阵、polkit/物理输入/HDR；基础结果仅见 [alpha.1 交接](../v0.2.0-alpha.1/plan.md) 和[历史证据](../v0.2.0-alpha.1/evidence.md)。Wayland 聚焦受合成器策略约束，跨用户/桌面会话不互锁；WebKit 子进程与播放期 helper 不算重复 GUI。完整许可审阅仍未完成，记录不变。
