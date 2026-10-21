# Go源码额外声明保留与纹理字体历史

日期：2026-10-20。任务M5/L、F04；未提交变更。当前Windows x64开发机，Python3.12.14只用于可选来源核对，不是应用运行时依赖。

## 实现和已确认范围

顶层Go BSD文本不能代替全部逐文件声明。已从官方固定Go1.23.1源码归档保留33份独立LICENSE/PATENTS等原文，以及11个经审阅源文件中的完整版权、许可与免责声明段落，生成[组件声明合集](../../../resources/licenses/uosc-go-NOTICES.txt)。合集88051字节、SHA-256 a0d773d39a11b6827c701ecb1933228b345fd133588b4037dd6b3d6f92c23649。

额外段落包括Sun数学函数声明、Cephes/Moshier原始说明、Lucent/Vita Nuova的amd64 memmove许可、fiat-crypto许可、AMD64 MD5作者公有领域声明、ELF BSD声明、IJG/MPEG JPEG声明、Google profile及traceviewer原始声明。IJG的要求确认文字也已保留。所有选段保留注释符、原版权、条件和免责声明，没有转换为自写许可证。

[审阅范围清单](2026-10-20-go-notice-excerpts.json)固定完整源文件大小/SHA-256与原始字节范围。33份独立声明覆盖整个Go归档，包括编译工具链vendor和可能未链接内容，属于保守保留；11个选段也不是完整逐文件扫描或实际链接图。因此没有把44来源条目数量当成全部依赖或完整适用审阅结论。

新增文件、其摘要/范围说明及来源记录接入现有组件许可事务。完整自有组件安装现在为63文件；声明与其他管理文件同样受预览、归属、备份、恢复和卸载保护。MpvKit仍为Rust后端，没有新增Go运行时或应用依赖，原有12份组件许可原文未改。

## 可重复入口

[collect-go-notices.py](../../../scripts/collect-go-notices.py)使用Python标准库，复用已有固定源码校验。先核对官方归档整体大小/SHA-256，再逐选定完整源文件核对类型/大小/SHA-256和选段边界；只读tarfile，不展开/执行归档内容。默认核对输出字节及source-materials.json中的摘要，--write只生成唯一声明目标；调整审阅范围后须同步审阅材料清单摘要。

~~~powershell
python -X utf8 scripts/collect-go-notices.py .local/source-research/go1.23.1.src.tar.gz
python -X utf8 scripts/collect-go-notices.test.py .local/source-research/go1.23.1.src.tar.gz
~~~

[拒绝风险测试](../../../scripts/collect-go-notices.test.py)为2项测试/4类用例：同长度声明篡改、越界选段、完整源文件摘要不同、重复来源。测试仅在自己新建的隔离目录验证输出，不改仓库真实声明；默认check不会把错误内容重写为正确内容后假通过。实际2项/4类均通过，4.483秒；临时初验也有4类全部拒绝记录。

## 纹理字体历史结果

[固定历史事实](2026-10-20-texture-history.json)记录：作者在[设计讨论](https://github.com/tomasklaen/uosc/issues/206#issuecomment-1248139886)明确表示尝试创建自己的字体，c909409提交引入字体、19份SVG与JSON工程；cabe84d修正a.svg及字体对齐，字体blob变为e89f1d8cb87262b02db089894e92a52e71f8bf8c，与当前固定字体一致。后续fonts→dist/fonts→src/fonts重命名保持此blob；中间656ddcf提交的38228字节又独立核对SHA-256 ccc0660f284dfceb5ab31eb363ccb2355df30fcdf628e781ee374b7d4172ada5一致。

[PR222](https://github.com/tomasklaen/uosc/pull/222)明确说明JSON工程未改，而TTF和a.svg对齐被修正。source-materials.json现在记录sourceHistoryVerified=true和generationReproduced=false；尚未重建，不将原始JSON直接称为当前字体的可重建证明。作者来源陈述也不被当成独立精确授权声明，纹理字体licenseMappingComplete继续为false；不套用Google图标的Apache许可。

## 验证与产物

- fmt/clippy通过；普通Rust55通过、0失败、6显式入口默认忽略（4.34秒）；单独真实归档1通过、0失败/0忽略（42.73秒），63文件/132786309字节，经真实校验/解析/临时部署/卸载；新增声明及全部许可原文的部署/卸载字节断言通过。
- 声明check及2项/4类拒绝测试通过；应用依赖许可--check仍为321包/566材料，react-remove-scroll-bar2.3.8缺口保持。原图标字体/五源码归档证据沿用，未把额外版权保留当成完整兼容性审阅。
- 调试与Release/NSIS构建内的前端类型/生产构建通过；八个应用资源源/暂存字节、NSIS精确File/Delete及来源路径通过。组件新增声明嵌入业务二进制，不是第九个应用bundle资源。链接器只报告创建lib/exp的中文信息，clippy -D warnings通过。
- 日志.local/v010-tests/go-notices-{clippy,backend,real-archives,debug-build,release-build}.log；声明go-notices-check.log，拒绝测试go-notices-rejection-tests.log及初验go-notices-negative-tests.json。资源记录go-notices-staged.json，签名go-notices-signatures.json。
- 最终95份自有文本/166本地链接、git diff、两个新增Python脚本语法/缩进/UTF-8/LF通过；15份Mantine原文、9个编译输入与2个当前产物摘要复核一致。最终声明check/拒绝测试重验及五源码归档与机器记录一致性通过，日志go-notices-final-*.log。
- 未重跑另外五个真实入口、九组播放矩阵或整套原生交互；业务/UI逻辑未改，交互安装计数断言为动态数量下界，无固定62文件假设。沿用此前61项完整后端和Mantine交互证据，不声称本轮全套重跑。

| 产物 | 字节 | SHA-256 | 签名 |
| --- | ---: | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14821376 | a94ba105104da4e668ca11a1dce79b9567c9cf76f8dfbd741a60b8a25fbc507f | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3869095 | e774aa85f767d5174c22386e682ca793d01cdbd202156e30a11361bfbfa257d2 | NotSigned |

身份记录.local/v010-tests/release-artifacts-go-notices.json。此前图标映射阶段产物为历史；新产物未签名、未运行安装器或发布。

## 限制与下一任务

Go完整逐文件/链接适用与helper对应重建、纹理字体精确授权和可重建源工程、MPV聚合构建材料、react-remove-scroll-bar2.3.8原文，以及用户确认保留的W/P系统/实体设备验收仍未完成。所有releaseReady继续false。下一步依据固定源码与helper实际构建信息核对Go链接适用/对应构建，继续其余L材料；不通过增加声明数量替代完整证据。

后续已追加两段实际选中Go源码的来源原文，声明46条/90469字节，并重建同路径产物；当前摘要及补充后普通/真实归档范围见[后续证据](2026-10-20-go-selected-notices.md)。本报告的当时数量、完整回归/交互结果与身份作为历史保留，不改写为新材料后的全量重跑。
