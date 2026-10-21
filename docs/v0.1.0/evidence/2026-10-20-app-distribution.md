# 应用分发许可原文与原生库核对

日期：2026-10-20。任务：M5/L。未提交、推送、发布或修改项目LICENSE/依赖锁文件。当前应用业务逻辑不变，本轮新增安装包资源映射与材料验证。

## 新证据

固定webview2-com-sys 0.39.1的x64 WebView2Loader.dll文件版本为1.0.3800.47；其src/lib.rs在MSVC目标明确静态链接WebView2LoaderStatic。依官方NuGet固定版本登记取得SDK归档，大小8973762字节，SHA-512 Base64：

~~~text
EASM6IwWa38ppWP825SH1xusVyN3fNCpi1xaDnHP80RVGgvCdBCwyw+Egqns3sdFSkWtDSp66Zj8NH6tFVmIiQ==
~~~

验证归档后，仅从明确条目读取LICENSE.txt、NOTICE.txt及build/native/x64/WebView2LoaderStatic.lib。crate静态库与SDK对应库的大小/摘要相同：10500472字节，SHA-256 89c6b872783b8f6c3cedbff618adb42082d455c615453ae10cfc753f1e8f25d8。不是单凭同版本DLL名称推断静态库归属。

SDK的1487字节LICENSE为Microsoft BSD条件文本，3894字节NOTICE原样保留；没有将crate仓库MIT文本当作整个Microsoft SDK的授权。NuGet原始登记和catalog保存在.local/source-research/webview2-sdk-registration.json / webview2-sdk-catalog.json，归档保留于同一忽略研究目录，未执行内容。

本机固定Rust1.98.1工具链自带COPYRIGHT-library.html，1512520字节，SHA-256 68129500b616d5838629e68f55ff3aed5e096dacf60ce9eb41bbe599a563afa6；原始HTML字节保存于resources/licenses/rust-library-COPYRIGHT.txt。它覆盖标准库版权汇编，不把整套编译工具链误当作应用随附内容。未从浮动master取替代文件，记录保留rustc完整版本/提交信息。

上述原文和库身份记录在[分发材料清单](../../../resources/app-distribution-materials.json)，均经大小/SHA-256与UTF-8检查。修改SDK NOTICE的实验被脚本拒绝，实验后已恢复原始字节。

## 安装包映射

Tauri bundle.resources使用七个明确源文件到目标文件的映射：

| 随应用程序安装的文件 | 来源 |
| --- | --- |
| LICENSE.txt | 仓库原始LICENSE，不改文本，不代替O09授权选择 |
| third-party/APP-DEPENDENCY-NOTICES.txt | 301个锁定应用依赖的546份许可/NOTICE原文 |
| third-party/app-dependency-inventory.json | 包版本、声明、许可原文路径与摘要 |
| third-party/app-distribution-materials.json | 原生库/标准库证据及已知限制 |
| third-party/RUST-COPYRIGHT-library.html | 固定Rust标准库版权汇编，原始字节 |
| third-party/WEBVIEW2-SDK-LICENSE.txt | 已匹配静态库的固定SDK许可原文 |
| third-party/WEBVIEW2-SDK-NOTICE.txt | 同一SDK的NOTICE原文 |

没有把全部上游工具链、源码缓存或二进制归档打入安装包。组件部署的12份许可仍走独立的组件事务；本轮三个应用原文走安装器自己的明确文件清单，不能混淆两种卸载边界。

复用[收集脚本](../../../scripts/collect-app-notices.mjs)的--check验证应用依赖合集及三个新增原文/仓库LICENSE摘要，不引入依赖。原文收集不是完整许可兼容性或对应源码交付证明；所有releaseReady保持false。

## 验证状态

node scripts/collect-app-notices.mjs --check及脚本语法/修改拒绝通过。前端类型/生产构建与Release/NSIS构建通过。生成NSIS脚本的七个写入/移除目标均核对，源文件与target/release暂存文件逐项SHA-256一致，记录.local/v010-tests/app-distribution-staged.json。初次临时检查误将/oname按文件basename匹配，按实际完整相对路径修正后通过，没有修改生成器来迎合检查。本轮61项完整后端回归全部通过，0失败/0忽略，961.27秒，见[收束核对](2026-10-20-final-audit.md)；不能将分发文件映射审阅当作真实安装/卸载验收。

## 余项

仍需完整审阅原生库逐文件许可与对应源码义务、Windows/NSIS运行时分发边界；MPV聚合源码、uosc字体/Go完整材料和项目精确授权选择尚未齐备。原文已经配置随包保留，并不表示可以越过发布门槛。隔离系统安装/卸载/WebView2与设备/实体HDR/功耗验收仍未完成。

## 后续项目授权更新

用户随后明确选择GPL-3.0-or-later，新增第八个资源PROJECT-LICENSE.md，原七个映射保留；清单记录项目SPDX及声明摘要。前文O09未决状态已由此关闭，第三方材料缺口继续保留；最新构建及八资源核对见[授权证据](2026-10-20-project-license.md)。
