# 固定Go立方根实现的SunSoft声明补充

日期：2026-10-20。任务M5/L、F04。接续[模块与生成输入核对](2026-10-20-helper-module-review.md)，核对固定选中Go输入的声明，并追加一段署名不同的完整原文。没有新增Go产品代码、维护脚本、依赖或工具链。

## 来源与实际差异

重新校验五份固定归档和952项已保存输入：922个标准库输入、22个模块输入、7个uosc输入与归档相同，另1个为此前核对的SDK生成文件。923个standard条目的大小/摘要复核通过；按明确的文本规则分为842个单年Go Authors头、76个前250字符内含生成标记、1个二进制表及4个其他头。后者是MD5汇编、memmove汇编、含年份范围的math/mod.go和无单独头部的path_windows.go。这些分类是定位声明的辅助证据，不是完整版权或链接适用结论。

13个数学函数含Sun版权段落，其中12个与已保留math/log.go中的SunPro原文完全相同；cbrt.go署名为SunSoft，不能以相似的SunPro段落替代。另4个Cephes版权/授权段落与已保留math/atan.go逐字节一致。crypto/internal/nistec/fiat/README中的完整版权/授权，去除注释前缀及缩进后与已保留edwards25519/scalar.go段落相同；这是授权文本对应，不是两个完整文件字节相同。因此不重复追加这16段数学声明或已保留的fiat授权。

新保留go/src/math/cbrt.go的310:684原始字节，共374字节，包含SunSoft署名、Sun版权和完整保留条款。完整源文件2366字节，SHA-256 de08f8225477bd143bd5318c5f46b7e26bdb008db8e4d174c6130e751e66e335；原文摘要773323f4b12573d59e6a6cbd9a967b4613f39fc2bb699b477041c808f1503bc1。[清单](2026-10-20-go-notice-excerpts.json)和[机器证据](2026-10-20-go-sunsoft-notice.json)记录范围、重复段落对应与本地报告摘要。

声明合集现在为33份独立声明+14选段，共47条、90968字节，SHA-256 c653af627706aef02a76be7b08cfb26e3d1b7f354da84541c52e170f0789ee74。旧90469字节前缀完整保留，只追加499字节（含来源标签）。同步材料摘要/数量说明；75个产品输入中只有声明合集、source-materials.json和components.rs数量说明变化，业务/前端/Lua/锁文件不变。

## 验证与失败记录

- 原文check与既有2项/4类拒绝测试通过，4.763秒；首次测试调用漏传归档参数，退出1且没有运行测试，补入固定归档后通过。初次Cephes严格比较包括未收录的版本标题，调整到完整版权/授权段落；Sun声明严格比较实际发现SunSoft差异，并据此补齐原文。
- 首轮普通后端在沙箱内33通过、23失败、6忽略，错误包括本地回环连接10013和文件替换权限拒绝；保留go-sunsoft-notices-backend-sandbox.log。相同测试沙箱外通过，未改断言。规范LF前56普通和真实归档亦通过；因嵌入清单CRLF多197字节，规范到LF后重新验证最终材料，前后记录分别保留，未混用部署字节数。
- 最终cargo fmt --check、clippy --locked --all-targets -- -D warnings通过；56普通后端通过、0失败、6显式入口默认忽略，4.24秒。单独真实归档1通过、0失败/0忽略，32.11秒：63文件/132789226字节，校验→解析→部署→卸载；许可原文字节、归属保护及卸载断言通过。
- 前端类型/生产、Tauri调试和Release/NSIS构建通过；八资源源/暂存字节及NSIS精确File/Delete目标核对通过。应用许可check为321包/567材料、missingMaterials为空。构建存在MSVC创建库的linker_messages提示，不冒称日志完全无提示；最终clippy无诊断。
- 先前[62项完整业务回归](2026-10-20-current-regression.md)和三类分次原生记录保留原范围；本轮只改声明/数量，不重复另外五个真实入口、图形矩阵或原生交互，不称新材料后全部62项重跑。

## 当前构建产物

| 产物 | 字节 | SHA-256 | 签名 |
| --- | ---: | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14775808 | e66dbd47ea6cca4edb570cf64b2b290f257bd170036f50f6cf843c28489355f0 | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3863996 | 0edede2ee2dd4b4ea5c7e24c9afea99e603e16f3d1834253ad40f89a69e25579 | NotSigned |

本地日志前缀.local/v010-tests/go-sunsoft-notices-；75输入快照go-sunsoft-notice-build-inputs.json，产物记录release-artifacts-go-sunsoft-notice.json。前两段声明阶段同路径产物成为历史，未签名、安装或发布。

本轮只关闭SunSoft原文遗漏及上述重复声明对应范围；生成来源/完整逐文件适用、原始helper对应构建、uosc精确授权与纹理字体、MPV实际聚合源码及W/P仍未验收。releaseReady=false，整版目标未完成。

本轮收束：112份自有文本/260本地链接及Git diff通过；75产品输入、两个产物、七份最终日志、四份前次运行日志、来源筛查报告/资源记录、15份Mantine原文与原始LICENSE摘要均匹配。最终原文check和321包/567材料应用check通过；本轮为实际原文补充及验证进展，完整L/W/P仍未完成。
