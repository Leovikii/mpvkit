# 固定 uosc / Go 源码归档核对

日期：2026-10-20。任务：M5/L 来源与许可材料；本轮为未提交的开发验证工具和证据更新，不修改应用业务、组件部署载荷、锁文件或技术栈。

## 范围与固定身份

MpvKit 仍使用 Tauri 2 + React/TypeScript/Mantine + Rust。Go 来自 uosc 5.13.0 附带的 ziggy-windows.exe；本轮仅读取已选定的源码归档，不安装 Go、不执行辅助程序或上游构建脚本。

辅助程序固定 SHA-256 为 ab76a5a059ef0aef0f54dc317e31eae25cc5b7e3840f81cbf98b1ab07a96f636；其 Go1.23.1、三个模块版本/h1 和局限见[原始内嵌信息证据](2026-10-20-uosc-helper-materials.md)。本轮补齐五份可获取的源码归档核对，未因此宣称二进制可重建。

[固定归档清单](2026-10-20-uosc-source-archives.json)保存下载URL、大小、SHA-256、模块h1、许可镜像对应关系，以及固定uosc提交的94个Git blob/大小/模式。下载文件保留在忽略目录 .local/source-research，不作为应用依赖或打包资源。

| 归档 | 字节 | 核对依据 | 实际结果 |
| --- | ---: | --- | --- |
| go1.23.1.src.tar.gz | 28164249 | Go官方发布JSON的大小和SHA-256；归档VERSION | 14694条目，121759452展开字节；33份独立许可/PATENTS等文件 |
| clipboard-v0.1.4.zip | 8606 | 固定helper内嵌h1与固定go.sum | 13文件、14573展开字节，h1一致 |
| browser-5ac0b6a4141c.zip | 7927 | 固定helper内嵌h1与固定go.sum | 15文件、7647展开字节，h1一致 |
| sys-v0.28.0.zip | 1983555 | 固定helper内嵌h1与固定go.sum | 534文件、9374406展开字节，h1一致 |
| uosc-d124c2c9.tar.gz | 2619287 | 固定提交GitHub Git树API的全部文件blob/大小/模式 | 113条目、3438295展开字节；全部94文件一致，无缺失或额外文件 |

Go归档摘要 6ee44e298379d146a5e5aa6b1c5b5d5f5d0a3365eabdd70741e6e21340ec3b0d 与[官方发布元数据](https://go.dev/dl/?mode=json&include=all)一致。三个模块ZIP的SHA-256只记录本次归档编码身份，独立内容依据是helper及go.sum中的h1；不能把本地计算的ZIP摘要单独当作来源真实性证明。h1排序/逐文件SHA-256汇总算法按[固定Go官方实现](https://github.com/golang/mod/blob/v0.21.0/sumdb/dirhash/hash.go)核对，使用Python标准库hashlib。

uosc归档来自固定提交 d124c2c930d69446448022851373e00ae592390d，其归档SHA-256为 c0df0c04cae467b6b20256a7c6a52dcbefc55f1ebd3b9794e907ca2481603bc6。逐文件核对包括完整ziggy源码、go.mod/go.sum及tools/build、tools/build.ps1；不是只核对入口文件。两个上游构建脚本都以 src/ziggy/ziggy.go 为入口、GOOS=windows/GOARCH=amd64、-ldflags "-s -w"，未执行它们。go.mod还包含tools使用的额外依赖，不能把go.mod的全部依赖都认作已链接进Windows helper；本轮以实际二进制内嵌的三个模块为准。

## 可重复入口与实际执行

可选开发验证需要Python 3.11或以后版本，仅标准库；不是应用构建/运行依赖。归档不会自动下载，维护者按清单准备五份固定文件后执行：

~~~powershell
python -X utf8 scripts/verify-uosc-sources.py .local/source-research
python -X utf8 scripts/verify-uosc-sources.test.py
~~~

入口[verify-uosc-sources.py](../../../scripts/verify-uosc-sources.py)先核对大小/SHA-256，再使用标准库tarfile/zipfile只读解析。拒绝缺归档、重复/越界/控制字符路径、不支持的链接或设备类型、条目/展开规模超限；不调用extract/extractall，不产生源码落盘或执行内容。模块逐文件读取同时核对ZIP CRC；uosc逐Git blob与执行位核对；最后核对固定go.sum摘要及三个内嵌模块记录。任何一步失败不会打印成功JSON。

本机Windows x64、Python3.12.14实际执行全部五归档通过；[机器结果](2026-10-20-uosc-source-verification.json)保存Go的33份独立声明位置/大小/摘要、模块及uosc声明，以及已保留许可原文的逐字节镜像核对。Go LICENSE/PATENTS、三个模块LICENSE、x/sys PATENTS（与已保留Go PATENTS字节相同）和uosc LGPL原文均一致。本结果只枚举文件名含LICENSE/COPYING/NOTICE/PATENTS的文件；没有把它当成完整逐文件版权扫描。

[拒绝路径测试](../../../scripts/verify-uosc-sources.test.py)5项通过，覆盖缺文件不能假通过、大小/哈希/h1不符、已保留许可不符、路径/重复/规模、TAR链接及uosc文件集合/blob/模式不同。临时目录仅位于仓库 .local/v010-tests，测试后清理各自新建目录。实际命令输出保留 .local/v010-tests/uosc-source-check.log 和 .local/v010-tests/uosc-source-negative-tests.log。

最终许可收集器--check通过：321包/566材料，react-remove-scroll-bar2.3.8缺口继续明确列出；自有89份文本和144个本地链接通过，另检查两个Python文件的语法/缩进及UTF-8/LF。15份Mantine第三方原始快照摘要、现有两个Release产物摘要与来源报告清单摘要均复核通过，git diff --check通过。首次许可复核未加载Cargo路径而报ENOENT；补入既定PATH后重验通过，没有更换依赖或工具链。

首轮实归档核对发现Go TAR合法根目录条目名为go，而不是以go/开头；修正目录成员用于边界检查的尾斜杠后，完整五归档重验通过。没有把首轮拒绝记成通过。

## 限制与交接

- 新增的Python仅为离线来源核对工具；没有新增产品运行时、组件下载或部署文件，因此沿用Mantine阶段产物身份和既有前后端/真实播放证据。本轮没有重跑Rust、原生UI、下载或图形播放测试，不声称它们已在本轮重新通过。
- Go独立声明清单包含编译工具链vendor、标准库vendor和boring等不同目录；哪些文件参与Windows非CGO helper、源码中的其他版权头如何保留仍需核对。33份文件的摘要清单不能证明完整第三方声明已随组件分发。
- 五源码归档取得并核对，不证明发布helper由该提交和这些材料重建；二进制主模块为(devel)，无VCS声明，尚未进行固定工具链重建比较。completeDistributionReview和reproducibleBinary仍为false。
- uosc两份字体完整许可映射、MPV聚合构建对应源码/材料、react-remove-scroll-bar2.3.8精确对应许可原文以及系统/设备W/P验收仍未完成。全部releaseReady保持false，不标记整版完成。
- 下一任务：依据已核对源码审阅Go逐文件版权/实际链接范围和uosc字体来源，保留适用原文后接入已有部署事务；继续核对其余L材料。用户确认只有当前电脑，系统/实体设备缺口按原决定保留。
