# M5：react-remove-scroll-bar 相同代码的 MIT 原文映射

日期：2026-10-20。仅更新第三方来源核对工具与许可材料，不升级依赖或改变产品业务。原始项目LICENSE和既有第三方原文不改；新增原文按上游字节保存。

## 原缺口与新增对应证据

Mantine迁移锁定react-remove-scroll-bar2.3.8，发布包与README声明MIT但未附正文；npm gitHead b3b1287aad81def2e2ae707274b74531b61ddbaf的GitHub入口此前404。上游公开提交清单仍为2.3.7，不能直接称作2.3.8同版本源码。本轮保留此事实，用实际文件对应建立补充原文来源，不依靠相近版本号推断。

1. 从npm官方注册表取得2.3.7/2.3.8发布包，分别核对各自元数据的SHA-512完整性。2.3.7的gitHead为29e9fcd1eecf7d3b77a767941c4a57fe461fc1e4。
2. 两发布包各27文件，其中26个文件（24个JS/类型声明、constants/package.json和README）逐字节相同。根package.json仅版本2.3.7→2.3.8、react-style-singleton依赖范围^2.2.1→^2.2.2不同；其余字段包括MIT声明一致。依赖自身许可由已有收集清单单独处理。
3. 下载2.3.7 gitHead和8ca9ba5ea52de03308fe8ced94f7b159a44d28ff固定源码归档；前者30文件、后者31文件，所有原30文件逐字节相同，仅增加LICENSE。2.3.7发布包的根package.json/README也与其固定源码逐字节相同。
4. GitHub比较入口另显示只有7301c160许可添加提交与8ca9ba5e合并提交、唯一变化LICENSE；原始响应保存在.local/source-research/scroll-bar-mapping/compare-from-2.3.7.json。正式复核依靠两固定源码归档完整字节比较，不依赖易变HEAD。

新增MIT原文1093字节，SHA-256 a79aae0c0f21990d9d963bb3c5a79cdcea9a46f8523ba55c58d7fe776b6ebc84，版权为2025 Anton Korzunov。补充文件resources/app-license-supplements/react-remove-scroll-bar-MIT.txt与固定源码/LICENSE逐字节相同。该授权对应同一项目的相同代码；不宣称2.3.8自身gitHead已恢复、取得了2.3.8新源码提交或完成全部分发义务审阅。

输入URL/大小/摘要、26个相同发布文件与30个相同源码文件的逐项摘要见[机器记录](2026-10-20-scroll-bar-license.json)。六份固定输入位于.local/source-research/scroll-bar-mapping，工具不下载、解包落盘或执行内容。

## 实现与验证

- 新标准库入口scripts/verify-scroll-bar-license.py复核全部固定输入、两个npm SHA-512、完整文件集、许可新增前后源码、清单精确差异和补充原文；最终输出必须与机器记录相符。命令为 python -X utf8 scripts/verify-scroll-bar-license.py .local/source-research/scroll-bar-mapping，已通过；日志.local/v010-tests/scroll-bar-license-check.json。
- python -X utf8 scripts/verify-scroll-bar-license.test.py：5项测试通过，包含正确映射、代码/意外清单/额外文件拒绝、随许可改变源码拒绝、同长度输入篡改拒绝，以及归档越界/链接/重复条目拒绝。最终日志scroll-bar-license-tests.log，0.049秒；两个Python文件UTF-8/LF、语法和缩进通过。
- 现有收集器无需新分支：补充清单新增该精确包ID，重新collect及--check通过，321包/567份材料，missingMaterials为空；新增原文与摘要已出现在生成合集/清单。不将材料数量等同全部链接或许可兼容性证明。
- pnpm tauri build --bundles nsis通过，包含前端类型/生产构建与Rust Release/NSIS；日志scroll-bar-license-release.log。八资源源/暂存逐字节相同，NSIS精确File源和目标、Delete目标通过，记录scroll-bar-license-staged.json；未执行安装器。
- 相比上一轮74输入，仅3个许可清单/合集资源改变，新增1份补充原文，75输入快照scroll-bar-license-build-inputs.json。业务源、前端、Lua、锁文件和工具链未变，沿用56项普通后端及三类分次原生通过；不重复慢网、播放矩阵或原生交互。此前6个显式入口未全重跑的边界保持，见[查询前保护证据](2026-10-20-runtime-probe-integrity.md)。

## 本阶段构建产物（历史）

| 产物 | 字节 | SHA-256 | 签名 |
| --- | ---: | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14771712 | ccd5fee77728500fe9cc71759d84fa1a548fd5946411caa78fad87b5917b35fb | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3865441 | bc382c0ee7c20832266efec4197babeefa9b03abca035eb3481ddaccaa142822 | NotSigned |

身份/签名记录.local/v010-tests/release-artifacts-scroll-bar-license.json、scroll-bar-license-signatures.json。上一轮同路径产物成为历史；补充原文已随合集/清单进入现有八项资源映射，不新增安装/卸载路径。

本轮关闭“已锁定应用包缺失许可原文”条目；完整对应源码/适用审阅、MPV聚合材料、uosc纹理字体与helper对应构建审阅仍未完成。W/P隔离Windows、目标设备、实体HDR/功耗按用户决定保留，releaseReady仍false。未安装、提交、推送或发布，整版目标未完成。

最终复核：实际安装的27文件与经SHA-512核对的2.3.8归档逐字节相同；104份自有文本/213本地链接及Git diff通过。许可--check再次确认321包/567材料、缺失项为空；75产品输入、2产物、15份Mantine原文和原始LICENSE摘要保持一致，catalog全部releaseReady仍false。

后续已追加两段实际选中Go源码的来源原文，声明46条/90469字节，并重建同路径产物；当前摘要及补充后普通/真实归档范围见[后续证据](2026-10-20-go-selected-notices.md)。本报告的当时数量、完整回归/交互结果与身份作为历史保留，不改写为新材料后的全量重跑。
