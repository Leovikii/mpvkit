# 项目许可决定与验收缺口交接

日期：2026-10-20。用户明确回复：暂时只有当前电脑，保留设备与系统验收缺口；项目选择GPL-3.0-or-later（第3版或以后）。

## 变更

新增[项目许可声明](../../../COPYING.md)，保留GNU GPL原文和第三方许可原始字节。Cargo、npm与Tauri bundle均声明GPL-3.0-or-later；安装包新增PROJECT-LICENSE.md，连同既有LICENSE.txt和六个third-party材料形成八个明确资源映射。没有增加依赖、升级工具链或改动播放/部署业务。

[分发材料清单](../../../resources/app-distribution-materials.json)保存项目SPDX与声明大小/SHA-256；现有许可收集脚本验证三处元数据一致性及声明摘要，并重新生成锁定输入摘要。第三方301包/546份原文不因项目授权选择而改变。

## 验证

脚本语法、离线收集及--check通过：301包、546材料、missingMaterials为空。pnpm install --frozen-lockfile --offline、cargo fmt --check、cargo clippy --locked --offline --all-targets -- -D warnings通过。前端类型/生产构建和Release/NSIS构建通过，日志.local/v010-tests/project-license-build.log。链接器报告创建导入库的stdout提示，不影响构建退出码0；clippy仍以-D warnings通过。八个资源逐项源/暂存字节相同，生成NSIS脚本均含准确写入/删除目标，记录.local/v010-tests/project-license-staged.json；这不是实际安装/卸载验收。

## 保留缺口

用户决定保留W/P缺口：资源管理器/默认应用真实交互、干净Windows安装/卸载/WebView2、两类实体设备/HDR/画质/功耗均未完成。第三方L缺口仍包括MPV聚合源码、uosc字体与Go标准库完整材料/对应源码审阅；O09已关闭不代表这些义务解决。所有releaseReady仍为false，整版未标记全部验收完成。

此前61项后端完整回归和两组原生交互通过的证据沿用[汇总报告](2026-10-20-v010-functional.md)，本轮仅许可元数据/资源与文档改变，未重复昂贵播放、下载或原生交互测试。未安装、提交、推送、签名或发布。

## 本轮构建身份

| 产物 | 字节 | SHA-256 | 签名 |
| --- | --- | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14673920 | 7a224ab584606488c4d40421bcf0774bbac6f645f306cd9fe051e3a4cc9a00bb | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3805682 | 9c711e2516f65e4f02cc7be0231e8791663f9523ac85f2e4783d532abdff0e75 | NotSigned |

两份文件Authenticode状态为NotSigned；原LICENSE与.gitattributes的git diff为空。锁文件没有因许可元数据变更而更新。

最终文档检查82份UTF-8/LF文本、103个本地链接及git diff --check通过。最后一次收集器--check与八个资源身份复核通过。临时检查初次将工作区CRLF许可文件与Git中LF blob直接逐字节比较，产生误报；改用变更前固定35802字节/SHA-256记录核对原文件字节，再规范行尾比较Git正文，两项均通过，未修改原许可文件。
