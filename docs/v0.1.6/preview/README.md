# 当前离线 UI 预览

目录名沿用 v0.1.6，内容已同步 v0.2.0-alpha.4 两页布局、固定紫色品牌、组件安装/检查更新/修复/卸载、配置预览/保存/重置，以及 WinUI 3 设置行/应用更新弹窗（默认关闭预览版）。准备、独立进度、一次确认和结果留在同一弹窗，普通按钮不承载进度。仅操作内存示例，仅模拟 Tauri IPC，不调用 WinGet/pacman，不写真实配置/默认应用；样例配置不是后端事务实现。

使用已锁定 React/Mantine；图标与品牌色复用产品，input.conf/uosc.conf 直接导入共享资源。样式全部在本目录，不再依赖已删除的 v0.1.5 三页预览。

```sh
node docs/v0.1.6/preview/build.mjs
node docs/v0.1.6/preview/check.mjs
```

输出 .local/ui-v016/preview.html，离线打开；加 --serve 时监听 127.0.0.1:1516，Ctrl+C 停止。检查默认使用 Windows Edge；Linux 用 MPVKIT_BROWSER 指定已安装 Chromium，具体示例见 [测试入口](../../testing.md#当前离线-ui-预览)。

文件 URL 加 `?updates=linux` 查看 Arch 应用内下载、安装预览、授权状态和完成；默认模拟 Windows 下载并安装失败（避免将预览当真实成功）。`updateFailure=download|verify|cancel|install` 演示错误或取消，`updateUnknown=1` 演示未知总量。更新组件直接复用产品 useAppUpdates，只有 IPC 使用官方 mock，无网络或系统写入。

check.mjs 覆盖：首次预编辑、组件/配置解耦与草稿保留、失败/手改确认、文件时间、自动连播/结束行为、组件动作状态、双语/明暗、焦点/Esc、640/900/1200 宽、200% 文字和正常/reduce 动效。输出截图仅留 .local。

界面与文字修改时同步本预览，但不把它的模拟成功等同真实 WebKit IPC、Mica、原生标题栏/图标、包管理器、播放器画面/选轨、DPI 或辅助技术验收。当前实际行为/限制见 [current](../../current.md) 与 [v0.2.0-alpha.4 交接](../../v0.2.0-alpha.4/plan.md)。
