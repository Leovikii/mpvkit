# uosc Windows辅助程序构建信息与许可材料

日期：2026-10-20。任务：M0/M5/L。未提交工作树；不改项目LICENSE、工具链或依赖锁文件。未执行下载的构建脚本；辅助程序只做字节读取。

## 固定资产与实际证据

先核对uosc 5.13.0 ZIP的固定SHA-256（4be9da3289285300fa374496c3f1bfd7bb20ac08e890d25bd5a06b28eebe4882），只从ZIP明确条目scripts/uosc/bin/ziggy-windows.exe读取：6017536字节，SHA-256 ab76a5a059ef0aef0f54dc317e31eae25cc5b7e3840f81cbf98b1ab07a96f636。

Go构建信息位于文件偏移5594624，头部指针宽度8、flags=2，内联字符串报告go1.23.1、CGO_ENABLED=0、GOOS=windows、GOARCH=amd64、GOAMD64=v1。三个外部模块：

| 模块 | 内嵌版本 | 固定上游提交 | 已保留原文 |
| --- | --- | --- | --- |
| github.com/atotto/clipboard | v0.1.4 | e4aee1922c628e6966acd4207c408df132dc6fb5 | BSD-3-Clause，Ato Araki |
| github.com/pkg/browser | v0.0.0-20240102092130-5ac0b6a4141c | 5ac0b6a4141c771a0d3a081c36b87c977cf9c7db | BSD-2-Clause，Dave Cheney |
| golang.org/x/sys | v0.28.0 | fe16172d1123f5350a8c5585395465de6866de4c | BSD-3-Clause，Go Authors |

三个h1值均与uosc固定提交d124c2c930d69446448022851373e00ae592390d的go.sum对应行一致；go.sum字节已核对Git blob 6a0384c4e3535cb190d070f92a995bb3a5c949aa。Go 1.23.1的LICENSE和PATENTS亦保留。没有把go.mod中的x/exp和k8s.io/apimachinery自动算作此二进制链接的模块。

[来源清单](../../../resources/source-materials.json)保存12份许可材料的URL/大小/SHA-256、完整内嵌构建字符串、三模块版本/h1及局限。[只读核对脚本](../../../scripts/inspect-uosc-buildinfo.mjs)先核对固定二进制摘要，才按Go 1.23.1内联格式读取并与清单比较。没有声明这是通用Go/PE解析器。

## 实现与验证

现有license_materials路径新增三模块许可和Go LICENSE/PATENTS共5份，随原有组件事务部署到licenses目录，同受预览、归属、备份、恢复和卸载保护。原文不改字节，releaseReady保持false。

~~~powershell
node scripts/inspect-uosc-buildinfo.mjs 'C:\隔离资产\ziggy-windows.exe'
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
# 显式设置MPVKIT_COMPONENT_ASSETS后：
cargo test --manifest-path src-tauri/Cargo.toml --locked components::tests::fixed_archives_extract_and_deploy -- --exact --ignored --nocapture
pnpm tauri build --debug --no-bundle
pnpm tauri build --bundles nsis
~~~

- 固定二进制构建信息核对通过；修改一个字节的副本被摘要核对拒绝，副本随后移除。记录 .local/v010-tests/uosc-helper-buildinfo.json。
- fmt/clippy通过；普通52项通过、0失败，6项真实入口默认忽略（其中真实归档入口随后显式执行）。已有许可摘要测试覆盖全部12份原文及外部播放器归属。
- 强化真实归档入口：实际辅助程序摘要与清单核对，许可证随真实组件一起部署，逐文件比较部署后原始字节，并核对卸载移除。结果：62文件/132695079字节，1通过、0失败/忽略，40.12秒；见 .local/v010-tests/uosc-licenses-real-archive.log。
- 前端类型/生产构建、Tauri调试exe及Release/NSIS构建通过；当前产物未签名、未安装、未发布。新产物摘要见[功能证据末尾](2026-10-20-v010-functional.md)。
- 运行逻辑与前端交互未改变，本轮不重复另外五个真实入口及两组原生交互；此前58项全量/两组交互通过记录保持为历史，不冒称本轮全部重跑。

## 尚未完成

内嵌元数据和h1一致不证明可重建性，也没有核对完整Go模块源码归档的目录哈希；uosc主模块标记(devel)，二进制未给出自身VCS提交。Go标准库内第三方逐文件许可/适用源码、uosc字体完整映射及MPV聚合构建源码材料仍待齐备。现有五份补充原文不能作为整个uosc可发布声明。

最终文档检查：75份UTF-8/LF文本、66个本地链接及Git diff检查通过；应用依赖301包/546材料的--check仍通过。

O09项目精确授权、隔离Windows与两类目标设备/HDR/画质/功耗等验收仍未完成。没有修改真实HKCU、默认应用或开发者播放配置。
