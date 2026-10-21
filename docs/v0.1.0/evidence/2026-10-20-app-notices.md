# 应用依赖许可原文收集

日期：2026-10-20。任务：M5/L。未提交工作树；未改动应用运行代码、锁文件、项目 LICENSE 或现有安装包。

## 实际结果

离线执行 cargo metadata --locked --offline --filter-platform x86_64-pc-windows-msvc，收集解析图中的297个外部Rust包（含构建期），并从已安装前端生产依赖追踪4个npm包：@tauri-apps/api、react、react-dom、scheduler。不是按可执行文件反推的最终成分表。

[生成清单](../../../resources/app-dependency-inventory.json)记录301个包的版本、声明许可、来源、546份许可/NOTICE路径及原始字节大小/SHA-256；[原文合集](../../../resources/app-dependency-notices.txt)统一行尾，未代替上游选择多选许可。[收集脚本](../../../scripts/collect-app-notices.mjs)仅使用Node标准库和现有Cargo，不联网、不安装依赖。

首轮有5个包未附带可识别的许可文件，已依据各自发布包.cargo_vcs_info.json和源码声明补齐：

| 包 | 核对依据 | 结果 |
| --- | --- | --- |
| alloc-stdlib 0.3.0 | 0a81fd6928ea3b33c8cd484aa4575d50ffb98012根LICENSE | Dropbox BSD-3-Clause原文 |
| selectors 0.38.0 | 572ecba2d1600e7c3d490586692a209faf703baa发布包lib.rs头部 | MPL-2.0声明；Mozilla官方纯文本 |
| webview2-com 0.39.1 / webview2-com-sys 0.39.1 | edc2caf886175ccaebe86078c9cfe1ae2a187328根LICENSE | Bill Avery MIT原文 |
| webview2-com-macros 0.8.1 | dffa41a8a46d3f5565eefbff2de57d38d399f158根LICENSE | Bill Avery MIT原文，字节相同 |

补充材料的固定来源、SHA-256和依据保存在[补充清单](../../../resources/app-license-supplements.json)，三份原文保留原始字节。selectors上游根LICENSE返回404，因此没有把猜测路径记作来源；其官方MPL URL来自发布源码的明确授权声明。未从其他项目推断该包授权。

## 复验

~~~powershell
node scripts/collect-app-notices.mjs
node scripts/collect-app-notices.mjs --check
node --check scripts/collect-app-notices.mjs
~~~

收集及重复检查通过：301包、546份材料、missingMaterials为空。额外暂改清单的实验确认--check拒绝过期输出；暂改补充许可字节确认摘要校验拒绝，实验后均恢复原件。全部546份材料的包ID/文件/摘要均可在原文合集中定位。

本轮没有修改前后端业务，不重复58项后端及两组原生交互；这些已通过结果和安装包身份仍见[功能证据](2026-10-20-v010-functional.md)。文本/链接与Git diff另经检查。

## 未完成边界

此处只补齐已解析包的许可原文，不是再分发合规证明。源码内逐文件声明、捆绑原生库、Rust标准库/工具链、MSVC/Windows SDK、WebView2/NSIS仍需核对；MPV聚合源码、uosc字体/Go依赖不在此收集范围。本报告初始收集时尚未接入安装包；后续已随包保留原文及标准库/SDK材料，见[后续分发证据](2026-10-20-app-distribution.md)。完整声明与适用源码材料仍未齐备。

所有releaseReady仍为false。O09待维护者选择；目标设备、实体HDR/画质/功耗和隔离Windows安装/卸载/默认应用验收依然未完成。

后续已通过同包相同发布代码与许可新增前后固定源码的逐文件对应补充MIT原文，当前321包/567材料、缺失原文项为空；原2.3.8自身gitHead不可访问的事实未改变。[后续证据](2026-10-20-scroll-bar-license.md)记录方法、风险测试及新产物，不改写本报告历史数量/结果。
