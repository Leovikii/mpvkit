# v0.1.1 架构摘要（历史）

2026-10-22 精简；旧布局和迁移设计正文已删除，不作为兼容需求。

保留的职责：React 展示/草稿 → 薄 commands → 业务模块 → 平台能力。结构化 GUI 设置与应用外观偏好分离，稳定 helper 随受管理环境保存，预览/摘要/锁/恢复保护实际文件和登记。组件成功、登记成功和默认应用是不同事实。

当前配置/路径/数据记录见 [v0.1.3 实际结构](../v0.1.3/storage-layout.md)，持续约束见[开发基线](../current.md)。原先 setup.json、playback.json、上下文 JSON、user.conf 及版本迁移已废弃，禁止复制旧实现。

Mica 使用原生 DWM、首帧前初始化及系统策略回退，保留证据：[调查](evidence/2026-10-21-mica-investigation.md)、[修复](evidence/2026-10-21-mica-fix.md)。平台登记与安装编排的下一步拆分见 [v0.1.4 方案](../v0.1.4/plan.md)；不预建空 Linux 实现。
