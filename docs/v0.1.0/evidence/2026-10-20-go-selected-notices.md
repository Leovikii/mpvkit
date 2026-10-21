# 固定选中输入的两段Go来源声明补充

日期：2026-10-20。任务M5/L、F04。继续审阅已取得的固定输入，补充实际存在且此前未保留的AES参考实现及amd64指数函数来源/免责声明；没有实现新的加密或数学代码，不增加Go产品依赖。

## 发现与范围

固定离线重建的selected-files.json（SHA-256 e9a0f289d20dfcf7ef71d184432a676d2bde6ae731c884875a59d1414b81c28b）共有952选中输入：923个standard条目（922匹配官方Go源码、1个SDK生成版本文件）、22个模块条目、7个uosc条目。本轮逐文件重新核对该集合的大小/摘要，再筛查标准头以外声明。输入含p256_asm_table.bin二进制数据表，不能把它当UTF-8文本；初筛遇到该表的解码错误后，按原始字节核对并区分数据与文本，未修改上游输入。

这不是完整逐文件版权或全部链接适用结论；生成文件、模块许可层级和原始helper对应构建仍有剩余审阅，不能把952次摘要相同称为版权审阅完成。筛选含普通permission等代码文本的候选也没有被误记为许可。

确认追加两段原文：

| 来源 | 完整源文件字节 | 原始字节范围 | 保留内容 |
| --- | ---: | --- | --- |
| go/src/crypto/aes/block.go | 6574 | 160:1683 | Rijndael参考实现版本、Rijmen/Bosselaers/Barreto作者、公有领域陈述及完整免责声明 |
| go/src/math/exp_amd64.s | 4344 | 183:816 | Shibata论文、原实现来源、历史README的公有领域陈述及Go简化版本说明 |

精确完整源文件摘要、范围及原文摘要见[本轮机器证据](2026-10-20-go-selected-notices.json)和更新的[选段清单](2026-10-20-go-notice-excerpts.json)。指数汇编保留的是固定Go源码对历史来源的陈述，不以今天的SLEEF版本授权或远程页面替换它，也不据此断言整个helper为公有领域。

## 变更与验证

- 复用现有collect-go-notices标准库入口，清单从33份独立声明+11选段增加为33+13，共46来源条目。新合集90469字节、SHA-256 1cc8c91b9bb436cfa3e69059aadc405218fa9b0a11f55815365dfb042586af3b；原88051字节前缀的SHA-256仍为a0d773d39a11b6827c701ecb1933228b345fd133588b4037dd6b3d6f92c23649，只追加2418字节，未重写既有原文。
- 更新source-materials材料摘要/选段数及随部署THIRD-PARTY.txt说明。只有三个产品编译输入变化：声明合集、source-materials.json、components.rs中的数量说明。归档固定哈希、下载、业务流程、前端、Lua、工具链和锁文件不变。
- 声明check通过；既有2项测试/4类拒绝用例通过（4.843秒），覆盖输出同长度篡改、范围越界、完整源文件摘要不同和重复来源。没有为两个数据条目建立重复测试；现有收集器逐条核对新范围及官方整包摘要。
- cargo fmt --check、clippy --locked --all-targets -- -D warnings通过；普通后端56通过、0失败、6显式入口默认忽略（4.39秒）。单独真实归档入口1通过、0失败/0忽略（33.92秒）：63文件/132788727字节，真实校验→解析→部署→卸载，所有许可字节及外部播放器保护断言通过。
- pnpm tauri build --debug --no-bundle和pnpm tauri build --bundles nsis通过，均包含前端类型/生产构建；八资源源/暂存字节与NSIS精确File/Delete目标通过。日志go-selected-notices-{clippy,backend,real-archives,debug-build,release-build}.log，原文go-selected-notices-check.json及go-selected-notices-tests.log；资源记录go-selected-notices-staged.json。
- [此前62项完整业务回归](2026-10-20-current-regression.md)与三类分次原生交互保留实际范围。本轮仅声明与数量说明，不重跑另外五个显式入口、图形矩阵或原生交互，不称全部62项在新材料后重新运行。

## 当前构建产物

| 产物 | 字节 | SHA-256 | 签名 |
| --- | ---: | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14775808 | 8895f6170d9bcd52c7d89f8f77e0037b17f05677006b8a9818d520f4962b7895 | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3863444 | 2affc50f9f2ad8f1da15ab970600630851e4cd29d3cfa03770a4463052db94e4 | NotSigned |

75输入快照.local/v010-tests/go-selected-notices-build-inputs.json；产物release-artifacts-go-selected-notices.json与go-selected-notices-signatures.json。前同代码MIT阶段的同路径产物成为历史，应用依赖321包/567材料数量不变。

本轮补齐两段实际遗漏的来源原文，完整Go逐文件/原始对应构建、纹理字体精确授权/最终生成工程、MPV聚合缓存对应源码及完整适用材料仍未齐备。W/P按用户决定保留，所有releaseReady=false；整版未完成，未安装、提交、推送或发布。

本轮收束复核：108份自有文本/240本地链接与Git diff通过；75产品输入、两个产物、七份验证日志、15份Mantine原文及原始LICENSE摘要全部匹配。声明check和应用许可check（321包/567材料、无缺失原文）通过；首次应用check因当前终端PATH缺cargo未执行，补入仓库工具及固定Rust路径后通过，没有修改系统PATH或依赖。完整L/W/P门槛保持未完成。

后续[SunSoft原文补充](2026-10-20-go-sunsoft-notice.md)仅改变声明与数量说明，生成新产物；本报告中的旧产物/输入不变声明保留为当时阶段。最新56普通/真实归档及构建已验证，未冒称62项/原生在新材料后重跑。
