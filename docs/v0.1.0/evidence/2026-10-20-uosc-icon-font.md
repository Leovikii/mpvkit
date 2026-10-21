# uosc图标字体精确来源与分发映射

日期：2026-10-20。任务：M5/L、F04；未提交变更。环境为当前Windows 11 x64开发机。没有安装字体、改动默认应用或真实配置。

## 已确认的对应关系

固定uosc 5.13.0发布ZIP及已核对源码中的uosc_icons.otf，Git blob为4c4e0dcfe20684e18da5fd6f4dfbd489ee947c66。实际400360字节，与Google material-design-icons提交[1c575764826fa5752ddc2bdc6b31ef7cc3233371的MaterialIconsRound-Regular.otf](https://raw.githubusercontent.com/google/material-design-icons/1c575764826fa5752ddc2bdc6b31ef7cc3233371/font/MaterialIconsRound-Regular.otf)逐字节一致；共同SHA-256为a853f50913597e67f76facd269ff230c2f3038f9acadec04dd653080785dee25。是未修改的原文件，不能只靠字体名称得出此结论。

固定提交[README](https://raw.githubusercontent.com/google/material-design-icons/1c575764826fa5752ddc2bdc6b31ef7cc3233371/README.md)的License节明确：

> We have made these icons available for you to incorporate into your products under the Apache License Version 2.0.

README原文7248字节、SHA-256 d51beabf61b701c7d5a33f9fb6c9b6e352c4111ba81da0899a2ea94ae22a8142。该提交[LICENSE](https://raw.githubusercontent.com/google/material-design-icons/1c575764826fa5752ddc2bdc6b31ef7cc3233371/LICENSE)与已有material-icons-Apache-2.txt逐字节一致：11357字节、SHA-256 58d1e17ffe5109a7ae296caafcadfdbe6a7d176f0bc4ab01e12a689b0499d8bd。字体内的Copyright 2019 Google LLC. All Rights Reserved.保持原样，并补入组件说明。

## 实现与可复验入口

- source-materials.json记录精确上游提交/URL/版权、字体与许可摘要、modified=false，只有uosc_icons.otf的licenseMappingComplete改为true。
- catalog.json和随现有事务部署的THIRD-PARTY.txt同步说明；没有修改上游字体或Apache许可原文。新元数据和说明仍走既有归属/预览/备份/恢复/卸载保护。
- 真实归档部署入口加强为核对两份字体摘要，继续比较所有实际部署许可原文字节并验证卸载移除。
- 新[只读比较入口](../../../scripts/verify-uosc-icons.mjs)验证两份字体大小/摘要/字节相等、官方许可与已保留原文字节相等；不解析或安装字体，不增加应用依赖。[机器结果](2026-10-20-uosc-icon-font-verification.json)记录具体身份。

~~~powershell
node scripts/verify-uosc-icons.mjs .local/source-research/uosc-icons-fixed.otf .local/source-research/material-icons-1c575764-font-MaterialIconsRound-Regular.otf .local/source-research/material-icons-1c575764-LICENSE
~~~

第一个输入只从已校验固定uosc源码归档的明确条目复制；ZIP中的对应资产又由真实归档入口独立校验。归档身份与完整94个Git blob证据见[源码归档记录](2026-10-20-uosc-source-archives.md)。Google输入按source-materials.json固定URL准备，临时研究文件不进入应用包。

## 实际验证

- 最终元数据上fmt/clippy通过；普通Rust55通过、0失败、6个显式入口默认忽略（4.37秒）。单独真实归档入口1通过、0失败、0忽略（41.62秒），62文件/132696598字节，经校验、解析、部署到临时目录及卸载；两份字体与全部许可断言均通过。
- 字体比较成功；对uosc字体、Google字体及Google LICENSE分别做同长度单字节篡改，3例全部拒绝。脚本语法、五源码归档与此前保存的机器结果一致性通过。
- 前端类型/生产、最终调试程序、Release/NSIS通过；八个资源源/暂存字节与NSIS File/Delete及来源路径核对通过。普通链接器仅报告创建lib/exp的中文信息，clippy -D warnings通过，没有未解释的编译诊断。
- 日志为.local/v010-tests/icon-font-{clippy,backend,real-archives,debug-build,release-build}-final.log；字体比较日志icon-font-verification-final.log，篡改拒绝记录icon-font-negative-tests.json。资源核对为icon-font-staged.json，签名为icon-font-signatures.json。
- 最终应用许可--check通过（321包/566份材料，保留react-remove-scroll-bar2.3.8缺口）；92份自有文本/154本地链接与git diff检查通过，15份Mantine原文、8个编译输入及2个产物摘要未变化。字体机器结果与最终入口输出一致，前轮源码验证器5项拒绝路径继续通过。
- 本轮没有重跑另外五个显式真实入口、完整原生交互或九组图形矩阵；沿用已有61项全量后端及Mantine交互证据，不把本轮55+单独1项写成全套重跑。业务/前端行为没有改动。

研究时先试Google4.0.0字体（242172字节）和固定f7bd4f25字体（400092字节），均与目标不同，没有当成来源通过。为定位相邻版本只读使用FontTools4.60.1纯Python归档（官方PyPI摘要906306ac7afe2156fcf0042173d6ebbb05416af70f6b370967b47f8f00103bbb），没有安装到系统或加入产品；工具身份保留.local/source-research/fonttools-research-tool.json。最终证明只需要原文件字节比较，不依赖该研究工具。

## 本轮产物与限制（历史）

| 产物 | 字节 | SHA-256 | 签名 |
| --- | ---: | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14731264 | ad9fea4d2869231efae9c787926309d38b5767dd530d3b025971a6d5ed29f32d | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3861142 | f28fd8fc5d1ac2d21ad79b6165edd24085a9f0884e7707dd3ef764f53626cb06 | NotSigned |

记录.local/v010-tests/release-artifacts-icon-font.json。两份产物未签名，未运行安装器、未安装或发布；此前Mantine阶段同路径产物身份转为历史。

uosc_textures.ttf仍未完成许可映射；Go逐文件/链接适用与对应重建、MPV聚合材料、react-remove-scroll-bar2.3.8精确许可原文及W/P系统/目标设备验收仍缺。Google图标映射不覆盖纹理字体。所有releaseReady保持false，整版目标未完成。

后续Go额外声明保留已重建同路径产物，本节为历史身份；当前摘要见[Go声明证据](2026-10-20-go-source-notices.md)。
