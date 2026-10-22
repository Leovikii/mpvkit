# 播放默认值：画质、HDR 与轨道语言

2026-10-22 用户审阅决定；已获本版实施授权；本文件记录当前实现决策，验证结果见本版交接。

## 画质与硬件检测（最新审阅决定）

删除 MpvKit 为播放配置提供的全部硬件检测与显示器选择逻辑。三档继续使用固定参数，初次生成默认“均衡”，由用户手动选择；不根据 GPU、分辨率、刷新率、电池或节能状态推荐、切换或重设档位。不新增按硬件分级、按分辨率着色器或后台监测机制。

首屏直接显示画质选择框和对应的一句建议，与 HDR 同组；不再增加摘要或放进高级选项。不再显示“已检测”“适合本机”“重新检测”或设备未知提示。中英文文案：

| 档位 | 中文提示 | 英文提示 |
| --- | --- | --- |
| 节能 | 降低渲染负担，适合低性能设备。 | Lower rendering load for less powerful devices. |
| 均衡（默认） | 推荐日常使用，兼顾画质与性能。 | Recommended for everyday use. |
| 高画质 | 推荐独立显卡使用，卡顿时请降低档位。 | A dedicated GPU is recommended; lower quality if playback stutters. |

这些是使用建议，不是硬件准入条件或流畅度保证。所有档位均可手动选择；保留 MPV 自身按实际输出尺寸缩放、自动硬解和 HDR Auto 的运行时行为。

实施时完整删除调用链，不能只隐藏界面：

- 前端：删除 Device 类型、检测状态/effect/按钮、检测失败文案、自动推荐及显示器/手动分辨率控件；删除 Settings/Playback 中 displayId、width、height、manualProfile 及对应默认值。
- 后端：删除 src-tauri/src/device.rs、platform/device.rs、detect_device 命令及模块/IPC 注册；删除设备报告、显卡/屏幕枚举、HDR 能力探测、供电推荐及其专用测试。
- 配置：删除 display_id、width、height、manual_profile 和对应验证；删除 validate_output 的显示器/HDR 探测校验以及 config/portable.rs、deploy.rs 中调用，不保留空函数。保留画质/HDR 枚举合法性验证、目标 MPV 参数能力检查及文件安全验证。
- 清理相关文案、诊断字段、测试夹具与仅为这些检测使用的 Windows API feature；共享依赖须先核对调用方，不能整项盲删。只维护当前设置格式，不做历史开发版字段兼容或迁移。

本决定针对播放器配置的硬件检测。MpvKit 窗口仍保留原生 Mica 能力/策略回退、主题、高对比、DPI 和响应式布局等必要平台行为；组件安装/版本/健康检测也继续保留。

## HDR

默认 Auto。首屏与画质同组提供一个三态选择框：自动（推荐）、强制开启 HDR、关闭 HDR（SDR）；不使用无法表示“自动”的单个二态开关，不增加第二个“启用自定义”状态。

Auto 继续使用 `target-colorspace-hint=auto`、`target-trc=auto`、`target-prim=auto`。界面不为 Auto/Off 显示附加说明。手动开启不再探测或校验显示器 HDR 能力与系统 HDR 状态；选择“强制开启 HDR”时就近提示“请先在系统中开启 HDR，并使用支持 HDR 的显示器。”（英文：Enable HDR in system settings and use an HDR-capable display.）不切换系统开关，不承诺环境适配成功。Auto 的判断由 MPV 在播放时完成，不由 MpvKit 检测；不能声称所有驱动、缺失信息和多屏切换均已验证。

## 语言默认值与界面

音轨和字幕均为 **汉语 → 日语 → 英语 → 西班牙语**，与应用语言无关。不提供 GUI 编辑、排序或高级开关。保留打开配置文件夹入口；用户可直接修改 `mpv.conf` 的 `alang` / `slang`，也可在 MPV 播放菜单中临时选轨。

拟生成相同的两项初始值：

```ini
alang=zh-Hans-CN,zh-Hans-SG,zh-Hant-TW,zh-Hant-HK,zh-Hant-MO,zh-Hans,zh-Hant,zh-CN,zh-SG,zh-TW,zh-HK,zh-MO,zh,cmn-Hans-CN,cmn-Hant-TW,cmn-Hans,cmn-Hant,cmn-CN,cmn-TW,cmn,yue-Hant-HK,yue-Hant,yue-HK,yue,chs,cht,ja-JP,ja,en-US,en-GB,en-AU,en-CA,en,es-ES,es-MX,es-419,es-AR,es
slang=zh-Hans-CN,zh-Hans-SG,zh-Hant-TW,zh-Hant-HK,zh-Hant-MO,zh-Hans,zh-Hant,zh-CN,zh-SG,zh-TW,zh-HK,zh-MO,zh,cmn-Hans-CN,cmn-Hant-TW,cmn-Hans,cmn-Hant,cmn-CN,cmn-TW,cmn,yue-Hant-HK,yue-Hant,yue-HK,yue,chs,cht,ja-JP,ja,en-US,en-GB,en-AU,en-CA,en,es-ES,es-MX,es-419,es-AR,es
```

列表按语言分组，使用一份内部常量输出两项，不引入别名解析器或 Lua 选轨脚本。详细标签用于匹配常见封装；不是要求 GUI 展示这串代码。

## 已核对的匹配依据

固定播放器提交：`6a262d11f63733d37e3ee6d5fafcb1be1ef83952`。

- [官方选项手册](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/DOCS/man/options.rst)：alang/slang 使用语言标签，标准二字母/三字母等价；slang 不跟随应用界面语言。
- [官方语言匹配实现](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/misc/language.c)：`zh/chi/zho`、`ja/jpn`、`en/eng`、`es/spa` 原生归一，不必逐个重复。匹配忽略大小写。地区和书写体系标签会影响分数；因此不能仅写 `zh,ja,en,es` 就声称任意地区标签都严格按四组排序。枚举上述常见地区/简繁组合，使已覆盖标签优先级稳定；`cmn`（普通话）、`yue`（粤语）并入汉语组。未知附加标签仍可能影响优先级，不宣称无条件兼容。
- [hooke007 示例](https://github.com/hooke007/mpv_PlayKit/blob/4921c6796620a13c2c03266f368d59d354a67b72/portable_config/mpv.conf)及 [dyphire 示例](https://github.com/dyphire/mpv-config/blob/132e1982ef47269118bd125b20aefd87dc646ab1/mpv.conf)：参考常见的 `chs/cht` 与简繁/地区写法，只引用选轨经验，不复制整套配置。`chs/cht` 是媒体生态中的非标准中文别名，只做精确标签匹配；错误标记无法凭配置识别真实语言。
- 不照抄 `sc`：目标 MPV 将其归一为 `srd`，可能把撒丁语误当汉语。`tc`、`ze`、标题中的“简中/双语”等不加猜测规则；需要时由用户手动选轨或编辑文件。语言标签缺失或错误时，不能识别音频内容或理解字幕文本。
- [选轨实现](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/player/loadfile.c)：语言偏好不是唯一规则，外挂轨、强制/默认标记、已有显式选择也会影响结果。保留 MPV 当前原生策略，不为偏好列表引入强制选轨逻辑；未匹配时沿用原生回退（字幕 `subs-fallback=default`）。

## 实现约束

1. 删除前端音轨/字幕列表、排序状态、LanguageList 及失去用途的文案；保留应用界面语言。清除 Settings/Playback 中对应可编辑字段及专用验证，不把它们变成隐藏设置数据库。
2. 初次生成 mpv.conf 时，将两项默认值放在 GUI 管理标记之外、正常全局作用域内。移除 `gui_line` 对 alang/slang 的归属。保存画质/HDR 等 GUI 项目不能覆盖、补回或重排用户的语种设置；用户主动“恢复推荐配置”才重建这些默认值。
3. 保留原有预览、手改保护、备份和恢复；无 user.conf，无旧开发版字段读取/迁移。媒体标签兼容与历史 MpvKit 格式兼容是不同问题，本方案仅处理前者。
4. 更新预览与配置摘要的语言：删除默认语种顺序、手改方法等说明文字；只保留配置文件夹按钮。

## 产品实施时验证

验证初次配置默认均衡、三个档位及双语建议对应、切页/保存保留选择；静态检查确认检测模块/IPC/字段/调用均已删除。HDR 三态可生成和保存，不再因设备检测失败或系统 HDR 未开而阻止保存，仍拒绝非法值。保持事务、覆盖保护及 MPV 参数能力检查；真实 HDR 与性能测试仍是独立验收，不能以短文案代替。

使用隔离的新环境与固定 MPV 构建：针对每个列表标签、标准二/三字母别名、大小写、简繁/地区标签及缺失标签建立最小选轨媒体样例，按汉→日→英→西逐级缺失验证实际选轨；额外验证中文地区标签与裸 `jpn` 同时存在。明确区分正常内封轨、外挂/强制轨及用户手动选轨的预期。

验证 HDR 默认 Auto、手动开/关与恢复 Auto；GUI 无语言控件；手改 alang/slang 后保存画质仍保留；恢复推荐配置明确显示差异。具体已运行结果以本版 evidence.md 为准，不将参数接受测试等同真实 HDR 输出验证。
