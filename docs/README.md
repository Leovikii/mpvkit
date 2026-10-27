# 开发文档入口

更新：2026-10-27，当前源码 v0.2.0-alpha.1，目录精简及应用更新已实施，Windows 原生回归、Linux 必要测试和 Arch 隔离包验收见本版交接。当前仍是第一个 alpha 预览版，0.2.0 是后续首个正式版目标；用户已授权公开现有 alpha.1，保留预览标记；CD 自动完成公开发布和更新索引，今后不采用草稿发布流程。AUR 暂不接入，Arch 先下载原生包用 pacman 手动安装。实际公开状态和未关闭审阅见当前交接。历史结果不替代本版验收。

用户介绍与安装见[主 README](../README.md)，预设取舍见[预设配置设计](presets.md)。AI agent 从 [AGENTS.md](../AGENTS.md) 开始，本页只作为开发与维护入口。

| 当前入口 | 用途 |
| --- | --- |
| [v0.2.0-alpha.1 当前交接](v0.2.0-alpha.1/plan.md) | 已实现目录/应用更新、NSIS/AUR 配方、CI/分发流程与验收边界、验证和发布门槛 |
| [当前行为与目录](current.md) | 产品现状、原生路径、模块职责与安全边界 |
| [v0.1.9 交接](v0.1.9/plan.md) / [详细证据](v0.1.9/evidence.md) | 已完成、用户实测、未测边界、下一任务与产物 |
| [本地测试入口](testing.md) | 固定工具、默认检查、Linux/Windows 原生、当前预览、来源校验 |
| [CI/CD 与分发](ci/README.md) | PR 检查、main 自动建 tag / 构建 / 公开发布与更新索引、失败重试及签名配置 |
| [界面规范](design/ui-guidelines.md) / [前端与 Mantine](v0.1.0/frontend.md) / [官方快照](vendor/mantine/README.md) | 当前两页、品牌、交互与组件 API 约束 |
| [组件管理与清理](v0.1.8/maintenance.md) / [uosc 与窗口依据](v0.1.8/uosc-window-review.md) | 当前专项决定；实施前调查不当作待审批或未实现 |
| [来源政策](v0.1.0/sources.md) / [验收台账](v0.1.0/acceptance.md) | 固定组件、许可证、真实设备和发布审阅缺口 |
| [当前离线预览](v0.1.6/preview/README.md) | 路径沿用 v0.1.6，持续同步当前两页演示与检查；不能替代原生验证 |

## 仅按需追溯

- [v0.1.8](v0.1.8/plan.md)：组件管理、uosc/窗口和两端原生验证；专属媒体登记已在 v0.1.9 删除。
- [v0.1.7](v0.1.7/plan.md)：Linux 接入与[原生目录检测](v0.1.7/linux-config-routing.md)。
- [v0.1.6](v0.1.6/plan.md)：预编辑、两页导航及真实文件时间；[v0.1.5](v0.1.5/plan.md)：Fluent/Mantine 与[播放默认值](v0.1.5/playback-defaults.md)。旧三页预览源码已删除，旧入口说明跳转到当前预览。
- [v0.1.4](v0.1.4/plan.md)、[v0.1.3](v0.1.3/plan.md)、[v0.1.2](v0.1.2/plan.md)、[v0.1.1](v0.1.1/plan.md)、[v0.1.0](v0.1.0/plan.md)：更早交付和原始 evidence。旧路径/迁移/备份/NSIS 安装器命令不再适用，详细旧源码也可从 Git 历史追溯。

保留 evidence、官方原文、来源/许可材料和未关闭验收；.local 中供本机复验使用的研究资产、工具链和证据继续保留，仅清理本轮用完的一次性桌面辅助脚本及状态文件。事实以源码和对应证据为准，历史授权不自动扩展到新任务。
