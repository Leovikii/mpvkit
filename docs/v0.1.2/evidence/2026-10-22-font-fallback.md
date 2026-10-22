# 字幕回退字体选择与隔离验证

日期：2026-10-22。用户指定霞鹜文楷项目并授权选择变体；选择已确定，产品接入待 v0.1.2 实施。用户同时认可之前的同步/倍速方案，profile 文件仅在实际条件切换需要时使用，不作为固定交付文件。

## 选择

采用 **霞鹜文楷屏幕阅读版 / LXGW WenKai Screen** 的完整非等宽版本，不选 Mono、Lite 或地区特化 GB/TC 版本。官方[屏幕版说明](https://github.com/lxgw/LxgwWenKai-Screen/blob/34f2c5602eb51d7455a0032c56ea9b0029edab53/Readme.md)指出：基于原版 Medium 的笔画粗细，将字重标为 Regular 并调整度量，面向 PC/移动屏幕阅读。较粗笔画适合作为视频背景下字幕候选；非等宽拉丁字符适合中英文混排，完整版保留较广字符覆盖。该选择不是已完成复杂视频背景/全部语言的视觉验收。

| 项目 | 固定值 |
| --- | --- |
| 上游 | https://github.com/lxgw/LxgwWenKai-Screen |
| 发布版本 | v1.530，2026-10-18 |
| 源码提交 | 34f2c5602eb51d7455a0032c56ea9b0029edab53 |
| 文件 | LXGWWenKaiScreen.ttf |
| 内部 family / style | LXGW WenKai Screen / Regular |
| 字节 | 26095590，约 24.9 MiB |
| SHA-256 | abd13364163e3cd84372b333f2fdcee9efb848634668f1c0ce5788dcd059ee61 |
| 发布下载 | https://github.com/lxgw/LxgwWenKai-Screen/releases/download/v1.530/LXGWWenKaiScreen.ttf |
| OFL.txt 字节 / SHA-256 | 6027 / ba38832f9d7e0ad601a1ab90d58f251ba957532885ef80dca050a611962fd8b6 |

已实际下载到忽略目录 .local/v012-font-research/fonts，逐字节 SHA-256 与 GitHub 发布资产 digest 一致；解析 TTF name 表确认 family 与 PostScript 名 LXGWWenKaiScreen，没有改字形、内部名称或制作子集。

## 接入方式

字体作为“播放增强”下的字幕回退资源自动准备，不新增引导选择或字体管理页面；下载、归属、预览、安装/更新/恢复和哈希检查复用现有组件机制。只下载一个 TTF，不下载全字体包或所有字重。不把字体嵌入管理器 EXE，不安装系统字体。

受管理配置目录的 fonts/LXGWWenKaiScreen.ttf 配合 sub-font=LXGW WenKai Screen。现代布局位于 config/fonts；旧平铺布局遵循已有配置目录映射，不能写到错误根。继续使用默认字体提供器及 embeddedfonts=yes，不设置 sub-ass-override=force/strip，不关闭系统字体。不设置 osd-font，也不将应用 UI 字体改为文楷。

- 无字体指定的 SRT 等使用文楷屏幕版。
- ASS 中可找到的字体保留；包含媒体内嵌字体时保持原查找机制，不主动替换。
- ASS 指定字体无法匹配或指定字体缺字时，使用可用的回退字形。文楷也缺少的字继续交由系统/libass 回退，不承诺覆盖所有语言/符号。
- 位图字幕（PGS/VobSub）不受字体设置影响。回退也不能保证缺失字体时原有排版尺寸完全一致。

sub-font 的用户手册主要描述无指定字体场景，但固定 MPV 的 sub/ass_mp.c 同时把它作为 libass 默认 family 传给 ass_set_fonts。实际 ASS 回退必须以目标构建日志为证据，不能只从文档措辞推断。无需为了本需求额外复制 subfont.ttf 或创建字体替换规则。

旧版已有配置只经用户确认的组件/配置更新接入，不在启动时静默改写；未知已有字体文件与手写覆盖按现有保护保留。字体丢失应报告并允许修复，不把仍可用的 MPV 谎报为不存在。新组件是否计入新装就绪须在组件迁移测试中明确，不能让所有旧环境因新增字体重进安装向导。

## 许可与来源

发布字体采用 SIL OFL 1.1。保留原始 OFL.txt、版权、保留字体名称及来源记录，在关于/第三方声明中列出；字体仍遵循 OFL，不改变 MpvKit 的 GPL 许可。官方说明另提及 Klee、Roboto 等来源，正式分发前核对字体内声明及适用 NOTICE，不能仅把一次许可阅读等同全部材料验收。保留上游原文件，不重命名字体内部 family、不改字形、不制作子集。

## 本机隔离验证

固定播放器：mpv v0.41.0-1224-g6a262d11f，日志 libass API=0x1705000，DirectWrite with GDI。使用仓库已有合成媒体，独立目录、--no-config、--load-scripts=no、--ao=null、--vo=gpu-next、--gpu-api=d3d11、最小化小窗口渲染两帧。明确指定隔离字体目录与字幕文件，不加载用户配置、不安装字体、不写注册表。

三项实际字体选择检查通过，机器记录见[结果](2026-10-22-font-fallback.json)：

1. ASS 字体名 MpvKitMissingFont_7e301 不存在：日志 Using default font family，选择 LXGWWenKaiScreen。
2. ASS 指定 Arial：英文选择 ArialMT；中文缺字回退到 LXGWWenKaiScreen，未强制替换整段字体。
3. 无样式 SRT：选择 LXGWWenKaiScreen。

验证范围仅为真实渲染路径字体选择，未完成内嵌字体、粗斜体、特殊字形、复杂 ASS 排版、HDR/不同背景可读性及下载部署恢复验收。原始日志、合成字幕、下载字体与许可原文保存在 .local/v012-font-research。此段记录实施前的研究阶段。后续已接入产品 catalog、配置生成与原始许可文件，并完成真实下载资产的解包/部署/卸载、归属及旧清单兼容检查；实际受管理 SRT 渲染和截图也已验证，见[实施证据](2026-10-22-implementation.md)。复杂 ASS、内嵌字体排版等未验收项继续保留。
