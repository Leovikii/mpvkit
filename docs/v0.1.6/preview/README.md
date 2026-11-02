# 当前离线 UI 预览

目录名沿用 v0.1.6，持续验证实际产品 App 与样式。当前 beta.2 窄窗两页、安装前编辑、配置与增强统一安装、独立保存和完整卸载，使用官方 Tauri mockIPC 和内存场景，不调用包管理器或写真实配置。预览不再维护另一份界面和共享样式。

```sh
node docs/v0.1.6/preview/build.mjs
node docs/v0.1.6/preview/check.mjs
```

输出 .local/ui-v016/preview.html，离线打开；加 --serve 时监听 127.0.0.1:1516，Ctrl+C 停止。检查默认使用 Windows Edge；Linux 用 MPVKIT_BROWSER 指定已安装 Chromium，具体示例见 [测试入口](../../testing.md#当前离线-ui-预览)。

文件 URL 加 `?updates=linux` 查看 Arch 应用内下载、安装预览、授权状态和完成；默认模拟 Windows 下载并安装失败（避免将预览当真实成功）。`updateFailure=download|verify|cancel|install` 演示错误或取消，`updateUnknown=1` 演示未知总量。更新组件直接复用产品 useAppUpdates，只有 IPC 使用官方 mock，无网络或系统写入。

check.mjs 覆盖：两页/草稿、一次确认、缺核心安装失败不部署、核心卸载失败仍清理、仅重试失败阶段、失败保存保留草稿、GitHub 署名及旧界面删除，480/560/720 宽、双语/明暗、200% 文字与正常/reduce 动效，并检查两页控件的实际右边缘及换行后对齐，保留设置页和主页截图。输出截图仅留 .local。

beta.2 补充 preset-update / preset-unapplied 场景，验证提示不自动弹窗、取消保留及重新应用保留选择；矩阵检查 SVG 管理箭头垂直居中和 ScrollArea 纵向滚动，额外验证键盘 End 及截图 beta2-scrollbar.png。

中英文按钮通过 DOM Range 检查文字范围位于 label 内、四边至少 4px 安全距离与垂直居中；覆盖全部页面矩阵，任务弹窗另查双语/480/200%/明暗及外层无原生滚动条。阈值来源和实测根因见[本版排版审查](../../v0.2.0-beta.2/plan.md#中英文排版与边缘检查)。

界面与文字修改时同步本预览，但不把它的模拟成功等同真实 WebKit IPC、Mica、原生标题栏/图标、包管理器、播放器画面/选轨、DPI 或辅助技术验收。当前实际行为/限制见 [current](../../current.md) 与 [beta.2 交接](../../v0.2.0-beta.2/plan.md)。
