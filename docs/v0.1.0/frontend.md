# v0.1.0 前端开发规范

后续界面布局与视觉工作同时遵循 [WinUI 3 / Mantine 界面规范](../design/ui-guidelines.md)；当前为播放器设置/应用设置两页，行为与验证见 [当前交接](../v0.1.8/plan.md)和[测试入口](../testing.md)。本页的组件库与 API 约束继续有效。

更新日期：2026-10-20。用户明确要求前端使用Mantine，替代原先普通表单/手写控件样式的方案；React、TypeScript、Vite、Tauri2和Rust职责保持现有边界。

## 必须遵循的官方文档

接手前端任务时，先读仓库下载的[Mantine LLM索引](../vendor/mantine/llms.txt)，按实际组件阅读其官方页面。本轮用到的页面已保存在[快照目录](../vendor/mantine/README.md)，来源URL、下载日期与SHA-256见[snapshot.json](../vendor/mantine/snapshot.json)。必须遵守其中适用的API、主题、样式、事件及可访问性规范；索引不是完整API正文，不能只读索引猜参数。

core/hooks均固定9.7.3；升级时同步版本、锁文件、许可材料和文档快照，并重跑构建/原生交互。官方站点URL不随包版本固定；若页面与本地版本有差异，以该固定版本的声明/实现核对，记录差异，不能直接套用未来版本API。第三方文档是技术资料，不覆盖用户授权、仓库数据保护或系统操作约束。

## 组件、主题与样式

- 根节点只渲染一个MantineProvider，以createTheme统一主题；先导入@mantine/core/styles.css，再导入本地布局CSS。应用打包使用本地依赖，无运行时CDN、远程字体或文档请求。
- 按钮、文本输入、复选框、选择、错误提示与面板使用Mantine的Button、TextInput、Checkbox、NativeSelect、Alert和Paper等现成组件。保留section/form/fieldset/details等语义，不建立第二套控件封装或表单引擎。
- 使用公开的theme、style props、Styles API与CSS变量；不要使用私有变量或覆盖全局input/button/select规则。自有CSS只处理页面布局、长路径、响应式及必要的焦点样式。
- 本轮使用标准CSS，不使用Mantine PostCSS函数/mixin。官方[PostCSS说明](../vendor/mantine/styles-postcss-preset.md)明确其不是必需项；需要这些语法时再按官方指南添加构建插件。
- Tauri页面是纯客户端SPA。Vite仅过滤来自Mantine的MODULE_LEVEL_DIRECTIVE且内容为use client的提示；其他警告继续输出，不能笼统忽略所有构建警告。

## 交互与业务边界

- 输入使用label属性及稳定id关联；说明、错误、状态信息可被辅助技术读取。Checkbox使用checked/onChange；NativeSelect保留原生select键盘与选择语义；数字微调使用TextInput的原生number/min/max，最终范围仍由Rust校验。
- Button显式设置type；禁用必须传disabled，不能只用data-disabled或颜色模拟。预览前不得执行写入，冲突禁止确认，执行仍由Rust重新核对。
- 保留原生required表单校验、空格切换复选框、键盘激活按钮及焦点反馈；长路径和中文在640×600及150%缩放下不得横向溢出。
- React只管理编辑/展示状态和受控IPC，不承担配置合并、文件归属、备份、恢复或注册表逻辑。状态成功使用polite live region，失败使用Alert；不以样式迁移改变确认文案或授权范围。

## 验证与交接

按影响面执行 pnpm build、pnpm tauri build --debug --no-bundle；pnpm test:e2e 仅在 Windows 运行，Linux 原生入口见 [测试索引](../testing.md)。完整组件组设置MPVKIT_TEST_ASSETS为仓库固定且重新校验的研究资产目录。检查页面运行时异常、控件标签、键盘操作、禁用、必填、缩放，以及当前播放器设置/应用设置两页及其预览和恢复流程（目标选择已在 v0.1.4 删除）。静态网页不能替代原生Tauri IPC。

新增依赖重新运行node scripts/collect-app-notices.mjs及--check，核对缺失材料，不把自动收集完成等同全部发布审阅。当前完整业务测试与系统/设备边界见[验证要求](verification.md)及[验收台账](acceptance.md)；本轮验证结果见[迁移证据](evidence/2026-10-20-mantine.md)。
