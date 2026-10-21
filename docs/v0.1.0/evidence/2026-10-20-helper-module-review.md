# uosc辅助程序选中模块与生成输入核对

日期：2026-10-20。任务M5/L、F04。接续[两段Go来源声明补充](2026-10-20-go-selected-notices.md)，核对固定重建中22个模块文件、7个uosc文件及1个SDK生成文件。没有改变产品代码、资源、锁文件或技术栈，也没有新增维护脚本。

## 模块声明层级

重新校验三个模块归档SHA-256、完整h1、22个选中文件大小/摘要，以及四项现有原文镜像；结果见[机器证据](2026-10-20-helper-module-review.json)。三个归档中的独立声明只有根LICENSE和x/sys根PATENTS，没有发现嵌套的同名声明文件；这个事实不等同全文版权适用审阅。

| 模块 | 选中文件 | 实际头部与已保留材料 |
| --- | ---: | --- |
| atotto/clipboard v0.1.4 | 2 | 两文件均有2013 @atotto版权头并指向根LICENSE；BSD-3-Clause原文保留Ato Araki署名 |
| pkg/browser 5ac0b6a4141c | 2 | 两文件没有独立版权头；归档根BSD-2-Clause保留Dave Cheney署名，不从缺少头部推断无授权 |
| x/sys v0.28.0 | 18 | 15文件有Go Authors/BSD版权头，3文件有生成标记；根BSD-3-Clause及PATENTS与现有原文字节一致 |

7个uosc源码均从package声明开始，无单独许可头。固定仓库根LICENSE.LGPL为LGPL 2.1正文，且与现有保留原文一致；仍不从正文模板推断helper精确的only/or-later授权表达或全部二进制材料已齐。

## 实际重生成一项模块输入

固定x/sys的windows/mksyscall.go指定五个输入：eventlog.go、service.go、syscall_windows.go、security_windows.go、setupapi_windows.go。本轮先核对生成器、五输入和原zsyscall_windows.go均与固定模块ZIP逐字节一致，再用已校验官方Go1.23.1便携SDK执行生成器。

参数契约为：go run <固定mkwinsyscall.go> -output <新隔离目录/zsyscall_windows.go> <按上述次序的五输入>。调用使用独立argv；GOROOT指向既有隔离SDK，GOTOOLCHAIN=local、GOENV=off、GOWORK=off、GOPROXY=off、GOSUMDB=off、GO111MODULE=off、CGO_ENABLED=0、GOOS=windows、GOARCH=amd64、GOAMD64=v1。没有联网获取模块或运行上游安装脚本。

实际退出0，stdout/stderr均空。生成文件206316字节，SHA-256 3182516f373d9992934ed52fee0efd37f9b721e0f73640f03ad541d09c790c50，与固定模块中实际选中输入逐字节一致。输出仅在.local/v010-tests/uosc-module-generator-fdqfpo7p；报告保留完整argv、生成器与输入摘要、SDK及go.exe身份、输出摘要，不覆盖原文件。

其余两张表的生成器已在固定模块ZIP内核对：mkerrors.bash读取本机Windows Kits最新的winerror.h与ntstatus.h，mkknownfolderids.bash读取最新KnownFolders.h。配方未固定当时SDK版本，当前没有据此重生成或将当前SDK当成原始输入。这是生成输入来源的剩余审阅边界，不影响本轮已核对的模块文件字节。

## SDK生成文件与证据边界

61字节runtime/internal/sys/zversion.go与固定SDK逐字节一致。官方Go源码中的cmd/dist/buildruntime.go和buildgo.go也与SDK内同文件一致；mkzversion的writeHeader加package sys换行与该61字节文件完全相符。没有重新执行整个go tool dist，不把字面量对应称为完整SDK重建。

本轮是来源与生成输入实验，产品输入不变；沿用[最近62项完整业务回归](2026-10-20-current-regression.md)、后续材料变更的56普通项/真实归档及此前三类分次原生结果，没有重复应用构建或前后端/图形测试。75产品输入、两个构建产物、15份Mantine原文及原始LICENSE的最终摘要复核另记于收束结果。

Go标准库其余逐文件/原始helper对应构建、两张SDK派生表的输入、uosc精确适用、纹理字体与MPV聚合材料仍未关闭；用户保留的系统/实体设备W/P验收仍未完成。releaseReady=false，整版目标保持未完成，未安装、提交、推送或发布。

本轮收束：110份自有文本/246本地链接与Git diff通过；75产品输入、两个产物、七份既有验证日志、15份Mantine原文和原始LICENSE摘要匹配。生成实验的实际报告/空stdout与stderr/输出摘要再次核对一致；产品及完整发布标志未变。

后续[SunSoft原文补充](2026-10-20-go-sunsoft-notice.md)仅改变声明与数量说明，生成新产物；本报告中的旧产物/输入不变声明保留为当时阶段。最新56普通/真实归档及构建已验证，未冒称62项/原生在新材料后重跑。
