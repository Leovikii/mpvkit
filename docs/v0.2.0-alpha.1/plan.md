# v0.2.0-alpha.1 历史交接

当前开发已转入 [alpha.2](../v0.2.0-alpha.2/plan.md)。本页只保留 alpha.1 结果，详细命令、失败修复和产物摘要见[原始证据](evidence.md)；当前授权与发布规则见 [AGENTS](../../AGENTS.md) 和 [CI/CD](../ci/README.md)，不继承本版实施过程中的旧禁令或草稿流程。

## 已完成

- 目录精简、应用偏好与 MPV 原生配置分离、事务失败恢复与清理；取消历史备份/撤销/迁移。系统 MPV 独立管理，无自定义播放器路径或专属媒体登记。
- Windows 一键用户级 NSIS 和正式 Tauri updater 签名；Arch 原生归档/软件包。应用启动自动检查一次，默认不收预览；Windows 签名更新、Linux 只提示手动更新。
- Linux 必要/扩展测试分开，默认五个画质/HDR 代表组合、两档 uosc 宽度，扩展矩阵按需运行；不缩减安全测试。
- PR 双平台 CI、main 版本检测和建 tag。首次失败因 Arch 环境缺系统 mpv，已补齐；Actions 升为官方 Node 24 版本并固定 SHA。
- 正式公钥和 Environment Secret 配置后签名重试成功。历史草稿补齐 SHA256SUMS，9 项摘要及 updater/Minisign 核验通过。
- README 改为英文用户介绍；AUR 注册关闭，暂从 Releases 下载包用 pacman 手动安装。用户另行授权的一次性旧配置清理已执行，不构成自动迁移。

## 实际验证

| 范围 | alpha.1 结果 |
| --- | --- |
| Windows | 原生完整 9/9，另含 52 项选轨；Rust 44 passed / 4 ignored；更新下载/签名拒绝测试通过 |
| Linux | Rust 47 passed / 4 ignored；必要 Wayland 链 16.53 秒，扩展 17.50 秒；KDE 偏好与原生 WebKit 更新短验收通过 |
| Arch 包 | release 构建、makepkg、vercmp、desktop、readelf、namcap；隔离 pacman 安装/同版重装/卸载保留全部 71 个配置文件，系统 MPV 哈希不变 |
| 共用检查 | fmt/clippy、前端、分发契约、两端许可材料一致性、离线预览与 actionlint 通过；缺失许可原文为空 |
| 签名包 | 正式密钥 Windows 包及独立验签通过；真实 NSIS 安装/跨版本升级未验收 |

## 历史状态与限制

alpha.1 tag `v0.2.0-alpha.1` 固定于 `2f3be388c561f0dfe0357a80f4b8c06e30d7b5a8`，不移动或重建已签名资产。曾被错误创建为草稿，后续公开结果记录在当前 alpha.2 交接。

许可完整审阅未完成，清单保持 `releaseReady=false`；用户已在 alpha.2 明确删除该人工状态的发布阻断，原文缺失及完整性检查保留。历史自动审批拒绝和签名重试过程均保存在 evidence，不再作为当前授权限制。

未以本版关闭：真实 NSIS 安装/升级、WinGet/polkit 系统写入、物理输入/辅助技术/HDR/混合 DPI、AUR 上线。隔离根与页面探针不等于完整系统验收。旧结果不得直接套用于后续源码或安装包。
