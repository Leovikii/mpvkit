# 固定 uosc Windows helper 离线重建

日期：2026-10-20。本轮只新增开发验证工具与文档，不改产品代码、依赖锁、运行时组件或应用资源。MpvKit 后端继续使用 Rust，Go 仅用于第三方 uosc 的 ziggy；便携工具链留在仓库隔离目录，不安装系统级 Go。

## 固定输入与执行边界

[SDK 清单](2026-10-20-uosc-go-toolchain.json)固定官方 Go 1.23.1 Windows amd64 ZIP，81905152 字节，SHA-256 32dedf277c86610e380e1765593edb66876f00223df71690bd6be68ee17675c0，与保存的官方发布 JSON 对应。其余输入沿用[五源码归档清单](2026-10-20-uosc-source-archives.json)和 source-materials 中已固定的 ziggy-windows.exe。先核对七个文件的大小/摘要，三个模块重新核对 h1、许可镜像，uosc 全部94个 Git blob/大小/模式及 go.sum 与内嵌模块重新对应。

[重建入口](../../../scripts/rebuild-uosc-helper.py)只使用 Python 标准库（3.11或以后），仅支持 Windows。归档用标准 zipfile/tarfile 解析，检查原始路径、条目类型与展开限制；不使用 extractall，不允许路径越界或覆盖已有文件。每次在 .local/v010-tests 下新建自己的目录，SDK/源码/模块/缓存/临时文件/结果均在该目录；没有执行上游 tools/build 脚本，也没有运行上游或重建后的 ziggy。仅运行已核对官方 SDK 的 go.exe 编译与只读检查 PE 构建信息。

GOROOT/GOPATH/GOMODCACHE/GOCACHE/GOTMPDIR/TMP/TEMP 均设置为本次隔离路径；清除继承的 GO*/CGO_* 配置，GOTOOLCHAIN=local、GOENV=off、GOWORK=off、GOPROXY=off、GOSUMDB=off、CGO_ENABLED=0、GOOS=windows、GOARCH=amd64、GOAMD64=v1。PATH仅包含本次SDK及Windows系统目录。不更改全局 PATH 或 go env；关闭Go模块/工具链在线获取，不声称建立了操作系统级断网沙箱。子进程使用参数数组、180秒时限和 CREATE_NO_WINDOW。

## 可复验命令

按上述清单准备五源码归档、SDK ZIP与固定 helper 到同一个本地目录，再运行：

~~~powershell
python -X utf8 scripts/rebuild-uosc-helper.test.py
python -X utf8 scripts/rebuild-uosc-helper.py .local/source-research
~~~

固定上游 tools/build 和 tools/build.ps1 中的 ziggy 分支均使用 ./src/ziggy/ziggy.go、GOARCH=amd64、GOOS=windows、-ldflags "-s -w"。本入口显式加 -mod=readonly，使用原始 go.mod/go.sum；仅准备三个实际使用模块的离线缓存，没有把 go.mod 的 x/exp、k8s.io/apimachinery 未使用声明冒称为实际链接依赖。此次编译参数为：

~~~text
go build -mod=readonly -ldflags "-s -w" -o <新隔离目录>/out/ziggy-rebuilt.exe ./src/ziggy/ziggy.go
~~~

## 结果与证据强度

最终成功目录 .local/v010-tests/uosc-rebuild-2bvd19tx，五个命令均退出0；[机器报告](2026-10-20-uosc-rebuild.json)保留命令、局部环境、包数、源码对应统计及二进制摘要。该目录中的 list-deps.stdout 为完整 go list -deps -json 输出，两个 buildinfo.stdout 为 go version -m 结果；selected-files.json 保存每个选中源码的种类/相对路径/大小/摘要，182483字节，SHA-256 e9a0f289d20dfcf7ef71d184432a676d2bde6ae731c884875a59d1414b81c28b，可由入口重新生成。原始日志留本机，不在开发文档保存用户名路径。

- 158个包，其中152个标准库包、3个外部模块包及3个uosc包（含 command-line-arguments 入口）。源码选择基于本次实际构建的依赖图，不等于已证明上游原始二进制的全部链接输入。
- 952个去重输入：922个标准库文件对应官方Go源码，22个模块文件对应已核对h1的模块ZIP，7个uosc文件对应完整固定源码；以上951文件逐字节对应。
- 唯一额外文件为 SDK 自带 go/src/runtime/internal/sys/zversion.go，61字节，SHA-256 f69c03727973664529c2e6fa2d2c53b2d3440c887938d0d9b22254f7984052f2，内容为 dist 生成标记和 package sys。它不在源码TAR中，不把它伪记为源码归档匹配；SDK整体身份已验证。
- 编译器版本和所有内嵌构建信息行相同，包括三个模块版本/h1、DefaultGODEBUG、编译选项和目标设置。报告保留三个 Module.Error 的 module lookup disabled by GOPROXY=off 警告：离线缓存只有原始 .mod/.zip/.ziphash，没有远程版本元数据；go build/go list 均退出0，包级无 Error/DepsErrors，来源/h1/实际选中文件独立核对。不是缺源码后静默跳过，也未制造发布时间元数据。其他模块错误直接拒绝。

| 二进制 | 字节 | SHA-256 |
| --- | ---: | --- |
| 上游固定 ziggy-windows.exe | 6017536 | ab76a5a059ef0aef0f54dc317e31eae25cc5b7e3840f81cbf98b1ab07a96f636 |
| 本次重建 ziggy-rebuilt.exe | 6054912 | 0e74df69ef63e3eb4d5b1b3e689278aa834047f4eccc29d66172685835e4992b |

二进制大小/摘要不同，reproducibleBinary=false。不同隔离路径下的重建摘要也不同；本轮没有定位所有差异来源，不能据此宣称差异仅来自路径、已证明同源或功能等价。未把重建结果替换进产品，也未宣称实际执行/字幕下载功能通过。

## 拒绝与修正记录

4项边界测试最终通过（0失败，0.074秒）：ZIP中的父级/驱动器/反斜杠路径及链接、TAR链接、已有文件覆盖、源码摘要差异/缺失文件、依赖越界、空或错误依赖图。命令行入口无 Module 字段时仍被限制在 uosc/src/ziggy 并纳入校验。

首次正式入口已完成五命令，但在分类 command-line-arguments 时触发 KeyError；目录 uosc-rebuild-zs61g6rp 保留，没有把失败报为完成。首轮测试暴露 Windows ZipInfo 会把反斜杠路径规范化的问题；已补充 orig_filename 与 filename 必须一致的解包前检查，并修正测试构造，最终4项全部通过。修正分类后的中间运行 uosc-rebuild-fbbrn_a_ 成功，最终增加原始路径保护与内嵌编译器检查后以 2bvd19tx 完整重验成功。三个实验目录均保留，不因一次无输出或观察超时重启进程。

## 产品与未完成项

本轮不改变应用编译输入和组件载荷，不重跑已有前端/后端/完整交互/慢网/九组图形矩阵；沿用[Go声明阶段验证与产品产物身份](2026-10-20-go-source-notices.md)，并复核摘要。源码包级/文件级对应性是新增证据，不等于完整版权适用、最终链接函数集合或许可兼容性审阅。

下一步继续核对上游原始helper与对应构建的差异及Go逐文件适用材料；纹理字体精确授权/生成工程、MPV聚合源码和 react-remove-scroll-bar2.3.8 精确原文仍有缺口。W/P系统与实体设备验收按用户决定保留；releaseReady继续false，整版未完成，未签名、安装、提交、推送或发布。

## 本轮最终复核

98份自有文本的UTF-8/LF/末尾换行/空白/JSON/Markdown围栏与178个本地链接通过，git diff --check通过；两个新增Python脚本语法/缩进检查通过。4项边界测试最终复跑通过（0.077秒）；既有Go声明check仍为44条/88051字节，摘要不变。应用依赖许可check为321包/566材料，继续明确react-remove-scroll-bar2.3.8缺口；初次未给当前进程补Cargo PATH而出现ENOENT，补入既有本地工具路径后重跑通过，未更改全局环境。

15份Mantine官方原文、9个既有编译输入、两个当前产品产物的大小/摘要均复核不变；Go声明阶段exe/安装包仍为当前身份，本轮无需重建产品。重建完成后的uosc全94个源码Git blob再次核对一致，包括原始go.mod/go.sum；五命令stderr文件为空，三个Module.Error仅保留在go list输出及机器报告中。完整前端/后端/原生/播放验证仍沿用前轮明确记录，未用维护脚本验证替代产品交互测试。
