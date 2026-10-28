# Mantine 官方 LLM 文档快照

2026-10-20 从[官方索引](https://mantine.dev/llms.txt)下载，入口为[llms.txt](llms.txt)。同时保存本次迁移使用的组件、主题和Vite集成页面；原始字节、来源URL和SHA-256见[snapshot.json](snapshot.json)。官方站点页面不是版本化URL，快照与实际锁定的core/hooks 9.7.3类型声明及构建共同核对。

第三方原文保持下载字节，适用仓库MIT文本见[LICENSE.txt](LICENSE.txt)。本文和snapshot.json为MpvKit编写；不要用格式化器改写快照。前端必须遵守这些官方文档中适用的API、样式与可访问性规范，具体接手与验证规则见[前端规范](../../v0.1.0/frontend.md)。安装包不包含开发文档；应用依赖许可另由resources中的原文合集分发。

2026-10-22：补充 [Tabs 官方正文](core-tabs.md)，已核对本地 9.7.3 的受控值、Panel、键盘激活与保留挂载声明；摘要与日期见 snapshot.json。

2026-10-22：为 v0.1.5 待审预览补充 [AppShell](core-app-shell.md)、[NavLink](core-nav-link.md)、[Accordion](core-accordion.md)、[Group](core-group.md)、[Stack](core-stack.md)、[Tooltip](core-tooltip.md)。核对 9.7.3 的布局尺寸、button 多态导航、折叠 heading order、间距及提示触发 API；原文哈希见 snapshot.json。产品尚未实施该重构。

2026-10-22：补充 [Transition](core-transition.md) 与 [useReducedMotion](hooks-use-reduced-motion.md)。核对 9.7.3 的 fade-up、duration/exitDuration、respectReducedMotion 及 Collapse/Transition 实现；官方 Transition 主要用于固定/绝对定位的呈现，不把它当通用布局动画引擎。

2026-10-28：补充 Loader 正文，核对固定 9.7.3 的 size/color 及 Button leftSection，用标准布局显示行内加载指示。
