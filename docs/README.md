# 文档入口

当前源码为 **v0.2.0-rc.1 / 预设修订 4**，已按[确认方案](v0.2.0-rc.1/plan.md)实施四项共用保存、Lua 拆分、动态菜单、控件与 Windows 完成页。用户实测反馈通过，已授权推送/合并 main 发布及同步 dev，发布流程进行中。验证、未测边界及接手入口见[本版证据](v0.2.0-rc.1/evidence.md)与[交接](v0.2.0-rc.1/plan.md#后续-agent-接手)。公开预发布暂为 beta.3。

已发布 [beta.3](v0.2.0-beta.3/plan.md) 的双端 CI、正式签名、Arch 隔离包验收及六项匿名下载核验通过，见[发布证据](v0.2.0-beta.3/evidence.md#公开发布核验2026-11-02)。标签固定 `8ab5afd`，发布交接后 dev/main 同步至 `167efb2`；KDE、真实 HDR 及系统安装更新等未测边界继续保留。

| 当前入口 | 用途 |
| --- | --- |
| [主 README](../README.md) / [预设设计](presets.md) | 已发布版本的介绍、安装及配置取舍 |
| [rc.1 开发计划](v0.2.0-rc.1/plan.md) | 四项共用保存、Lua 拆分、状态菜单、控件与安装完成页；已实现方案、验证边界及交接 |
| [AGENTS](../AGENTS.md) / [当前基线](current.md) | 授权、安全约束、实际行为、原生目录及源码导航 |
| [测试入口](testing.md) / [CI/CD](ci/README.md) | 固定工具、隔离测试、签名/发布与失败重试 |
| [界面规范](design/ui-guidelines.md) / [前端规范](v0.1.0/frontend.md) / [Mantine 快照](vendor/mantine/README.md) | 当前布局、动效、组件 API 与可访问性 |
| [来源政策](v0.1.0/sources.md) / [验收台账](v0.1.0/acceptance.md) | 固定来源、许可与未关闭设备/系统验收 |
| [离线预览](v0.1.6/preview/README.md) | 路径沿用旧版本，当前演示 rc.1；不能代替原生验证 |

历史交接只按需追溯，不继承授权或测试结论：

- 0.2.0：[beta.3](v0.2.0-beta.3/plan.md)、[beta.2](v0.2.0-beta.2/plan.md)、[beta.1](v0.2.0-beta.1/plan.md)、[alpha.5](v0.2.0-alpha.5/plan.md)、[alpha.4](v0.2.0-alpha.4/plan.md)、[alpha.3](v0.2.0-alpha.3/plan.md)、[alpha.2](v0.2.0-alpha.2/plan.md)、[alpha.1](v0.2.0-alpha.1/plan.md)。beta.2 后续安装顺序/音轨修正已包含在 beta.3；Linux 窗口单项证据仍在 beta.1。
- 早期：[v0.1.9](v0.1.9/plan.md)、[v0.1.8](v0.1.8/plan.md)、[v0.1.7](v0.1.7/plan.md)、[v0.1.6](v0.1.6/plan.md)、[v0.1.5](v0.1.5/plan.md)、[v0.1.4](v0.1.4/plan.md)、[v0.1.3](v0.1.3/plan.md)、[v0.1.2](v0.1.2/plan.md)、[v0.1.1](v0.1.1/plan.md)、[v0.1.0](v0.1.0/plan.md)。旧目录、迁移、三页界面及旧安装任务不再适用。
- 专项：[组件维护](v0.1.8/maintenance.md)、[uosc/窗口调查](v0.1.8/uosc-window-review.md)、[Linux 路径](v0.1.7/linux-config-routing.md)；适用范围以当前基线和新方案为准。

重复叙述在当前文档收敛；发布说明、原始 evidence、来源/许可、官方原文与未测事项保留，失效设计可查 Git 历史。.local 研究资产、工具链和复验证据不分发，不作为新机器必备前提。
