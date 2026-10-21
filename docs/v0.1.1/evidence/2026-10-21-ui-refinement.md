# v0.1.1 界面反馈修订验证

日期：2026-10-21。用户授权本轮实现；版本仍为 0.1.1。

## 变更

- 删除“直接使用此文件夹存放配置”，安装页显示有效路径。“高级选项”内的自定义安装目录默认关闭；取消勾选回到平台默认目录。已有自选记录重启后保留。
- 当前位置未改变时直接进入组件检查，不展示变更确认，也不写 target.json；实际换位置继续预览新旧路径、确认并重新核验。
- 后端目标状态提供 defaultRoot；目录布局不再由 UI 指定。新安装采用同一根下的 runtime/config，已有管理清单或同目标历史布局自动保留，不猜测外部文件归属。
- 组件页的覆盖选项/来源详情、播放页的分辨率/画质/HDR/语言及配置目录入口统一放入“高级选项”。默认推荐结果、继续和保存操作常显。
- 移除窗口背景选择，系统能力与透明/供电/焦点策略自动决定 Mica 或同主题纯色。旧 background=solid 记录兼容读取并规范为 auto，不在启动时写盘。
- 产品 SVG/Windows ICO 改用 #B77ACD 亮紫色，保留原 SVG 图形与无框样式；文档原始 SVG 不变。

## 验证

- 前端类型检查、构建、中英资源 2 项通过；Rust fmt、Clippy --all-targets -D warnings 通过；targets 两项和 app_settings 一项通过。
- 调试版真实 WebView2 五组分次通过：主旅程 209.12 秒（mpvkit-v011-W6G0ac）；旧版/外部播放器 58.31 秒（mpvkit-v011-legacy-oPwDua）；外观/失效目标 7.76 秒（mpvkit-v011-appearance-qtzlDg）；键盘/保存失败 2.87 秒（mpvkit-v011-keyboard-mBRCSv）；新增默认/高级目录 4.06 秒。
- 新增目录用例覆盖：默认折叠/未勾选、勾选后编辑、取消自定义恢复默认、同路径无弹窗且不写目标、取消预览无写入、实际切换确认、重启恢复自选路径、切回默认目录。
- 主旅程包含真实 MPV 播放及稳定 helper、配置优先级、登记私有后端与移除保护。旧版旅程保留直接配置布局、旧设置迁移和外部 MPV。未重复不受本轮影响的完整媒体/参数矩阵。
- 已检查中英安装页及浅色设置页截图。16 份 Mantine 官方快照未改；SVG 路径/viewBox 一致；ICO 包含 16/24/32/48/64/256 尺寸。链接与差异空白检查通过。

## 失败记录与测试保护

旧版旅程初次失败是 getByRole 默认排除折叠区域：测试未展开高级选项便定位 Quality。修正用户操作顺序后整组通过，未放宽产品断言。

追加 Release 冒烟时发现平台测试根与私有登记替代仅在 debug_assertions 启用。测试在目录断言失败前把真实应用的界面语言切为 en，未进行下载、MPV 配置或登记操作。已依据用户截图恢复为 system，其他 theme/accent/background 保留原值。此失败不算 Release 业务测试通过。

测试启动现在必须先核对 target_status.defaultRoot 等于隔离根，才允许任何 UI 写入。使用实际 Release EXE 验证守卫按预期提前拒绝，真实 app-settings.json 前后摘要一致。后续业务回归仅使用隔离调试版；不通过启用 Release 测试后门绕过限制。

## 新产物

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| Release / 单 EXE 测试副本 | 22471168 | a6fb3c2e4b6123d2a56ca306a05240ee1ee0db62857c49d52d8f8360055b67db |
| NSIS 安装包 | 4017427 | 590423e0815f8aa0addf8d9db112644b217b179cfb895d36914e515025c2b846 |

测试副本：.local/test-build/MpvKit_0.1.1_x64-test.exe。安装包：src-tauri/target/release/bundle/nsis/MpvKit_0.1.1_x64-setup.exe。当前产物摘要写入 .local/v011-tests/artifacts.json，产品输入摘要为 ui-refinement-sources.json；前一份实施证据的摘要属于修订前历史产物。

未签名、未安装、未公开发布。活动态 Mica 目视、实际系统对话框/Windows 安装关联和目标设备 HDR 仍按[手动清单](../manual-check.md)补验。Linux 仍是后续平台范围，本轮未实现或宣称测试。releaseReady=false。
