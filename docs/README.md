# 文档入口

当前公开预发布为 **v0.2.0-beta.3**，双端 CI、正式签名及匿名公开下载核验通过，见[本版交接](v0.2.0-beta.3/plan.md)与[发布证据](v0.2.0-beta.3/evidence.md#公开发布核验2026-11-02)。

管理器主播放设置只保留默认画质，主界面直接显示增强组件及状态；HDR/自动连播/结束行为采用默认值并收入 uosc 菜单。安装或重新应用生成管理器对应语言的简洁单语言文本。安装确认前不下载、音轨显示及倍速菜单修正也已包含，整套预设修订为 3。原始历史结果留在 [beta.2](v0.2.0-beta.2/plan.md)，本批验证与未测边界见 [evidence](v0.2.0-beta.3/evidence.md)。

用户介绍与安装见[主 README](../README.md)，配置取舍见[预设设计](presets.md)；开发接手从 [AGENTS.md](../AGENTS.md) 开始。历史版本按需追溯，不继承旧版测试通过结论。

| 当前入口 | 用途 |
| --- | --- |
| [v0.2.0-beta.3 需求与方案](v0.2.0-beta.3/plan.md) | 本版发布：精简播放设置、扩展菜单、安装对应语言文本；验证与未测边界 |
| [v0.2.0-beta.2 需求与方案](v0.2.0-beta.2/plan.md) | 历史实施、单一预设修订/重新应用、原生 uosc、UI 优化、授权与验证边界 |
| [v0.2.0-beta.1 方案与交接](v0.2.0-beta.1/plan.md) | 当前两页/统一安装基线及发布后 MPV 窗口修复；验证、入口与待验边界 |
| [v0.2.0-alpha.5 实施与交接](v0.2.0-alpha.5/plan.md) | 历史公开版：三页、统一窗口材质、安装配置分离、本地测试程序与验证边界 |
| [v0.2.0-alpha.4 实施与交接](v0.2.0-alpha.4/plan.md) / [调查证据](v0.2.0-alpha.4/evidence.md) | 历史修复、根因与双端测试矩阵 |
| [v0.2.0-alpha.3 历史交接](v0.2.0-alpha.3/plan.md) | 标准更新、两端进度、Arch 安装、工作流及验证 |
| [当前行为与目录基线](current.md) | beta.3 已实现行为、原生路径、模块职责与安全边界；本版验证见 beta.3 交接 |
| [v0.1.9 交接](v0.1.9/plan.md) / [详细证据](v0.1.9/evidence.md) | 已完成、用户实测、未测边界、下一任务与产物 |
| [本地测试入口](testing.md) | 固定工具、默认检查、Linux/Windows 原生、当前预览、来源校验 |
| [CI/CD 与分发](ci/README.md) | PR 检查、main 自动建 tag / 构建 / 公开发布与标准 manifest、失败重试及签名配置 |
| [界面规范](design/ui-guidelines.md) / [前端与 Mantine](v0.1.0/frontend.md) / [官方快照](vendor/mantine/README.md) | beta.1 两页目标、品牌、保留动效、交互与组件 API 约束 |
| [组件管理与清理](v0.1.8/maintenance.md) / [uosc 与窗口依据](v0.1.8/uosc-window-review.md) | 既有专项依据；beta.1 完整卸载覆盖旧按归属保留内容的卸载范围，其余保护继续有效 |
| [来源政策](v0.1.0/sources.md) / [验收台账](v0.1.0/acceptance.md) | 固定组件、许可证、真实设备和发布审阅缺口 |
| [离线预览入口](v0.1.6/preview/README.md) | 路径沿用 v0.1.6，已同步 beta.3 画质单项、重新应用与安装流程；不能替代原生验证 |

## 仅按需追溯

- [v0.2.0-alpha.2](v0.2.0-alpha.2/plan.md)：单实例、Linux 分类与直接公开发布；旧 Release/tag 已撤下。

- [v0.2.0-alpha.1](v0.2.0-alpha.1/plan.md)：目录精简、应用更新、首轮双平台包及[原始证据](v0.2.0-alpha.1/evidence.md)。

- [v0.1.8](v0.1.8/plan.md)：组件管理、uosc/窗口和两端原生验证；专属媒体登记已在 v0.1.9 删除。
- [v0.1.7](v0.1.7/plan.md)：Linux 接入与[原生目录检测](v0.1.7/linux-config-routing.md)。
- [v0.1.6](v0.1.6/plan.md)：预编辑、两页导航及真实文件时间；[v0.1.5](v0.1.5/plan.md)：Fluent/Mantine 与[播放默认值](v0.1.5/playback-defaults.md)。旧三页预览源码已删除，旧入口说明跳转到当前预览。
- [v0.1.4](v0.1.4/plan.md)、[v0.1.3](v0.1.3/plan.md)、[v0.1.2](v0.1.2/plan.md)、[v0.1.1](v0.1.1/plan.md)、[v0.1.0](v0.1.0/plan.md)：更早交付和原始 evidence。旧路径/迁移/备份/NSIS 安装器命令不再适用，详细旧源码也可从 Git 历史追溯。

保留 evidence、官方原文、来源/许可材料和未关闭验收；.local 中供本机复验使用的研究资产、工具链和证据继续保留，仅清理本轮用完的一次性桌面辅助脚本及状态文件。事实以源码和对应证据为准，历史授权不自动扩展到新任务。
