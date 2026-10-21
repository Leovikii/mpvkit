# F01/F08/F11：查询前验证受管理播放器与运行依赖

日期：2026-10-20。未提交工作树。修复受管理启动与诊断的执行顺序，并补充真实进程见证和原生界面回归；没有增加产品依赖、修改组件版本或改变配置档位。

## 问题与实现

代码核对发现 prepare_managed 在核对受管理文件前调用 ensure_managed_options；diagnostics::prepare 虽统计文件变化，仍执行 mpv::inspect。因此播放器或其DLL已变更时，能力/版本查询仍可能先启动该程序，之后才报告完整性问题。

共享 mpv::managed_executable 现在先确认 runtime/mpv.exe 有明确归属，并校验所有已记录runtime文件的摘要/存在状态，再返回可查询的路径。外部播放器继续走用户选择路径，但查询前也通过现有本地路径、普通文件及mpv.exe名称检查。受管理启动把能力查询移到原有全部受管理文件检查之后；诊断使用同一入口，失败时保留计数及静态错误代码、实际版本为空，不执行版本查询。导出仍只读预览/create_new，不自动修复或覆盖修改文件。

实现位于 src-tauri/src/mpv.rs、launch.rs、diagnostics.rs；前端IPC/schema不变，继续使用Mantine。缺少归属返回managed_player_unowned，已记录运行文件缺失/摘要变化返回file_conflict；不可读取/路径错误保留已有错误代码。只检查已记录的运行文件，不按文件名猜测未知资产归属。

## 回归与本机构建

- 先增加fixture的可选record-probes模式：真实执行时向该测试播放器目录写probes.txt，其他模式不改变。新回归以四种状态（exe变更、同长度DLL变更、DLL缺失、exe无归属）分别检查启动和诊断；每种先验证完好运行文件确实会产生探针记录，再删除记录、制造变化，验证两入口均无新探针。诊断保留变化计数且预览不生成导出文件。
- 修复前单项失败，证据 .local/v010-tests/runtime-probe-red.log，明确命中“player: no player process may run before integrity/ownership checks”；不是仅断言错误文案。修复后同项通过，runtime-probe-green.log，2.09秒；随后全部普通后端56通过、0失败、6个显式入口本轮未全重跑（5.22秒），runtime-probe-backend.log。此前61项完整后端为旧阶段证据，不冒称当前62项全部重跑。
- cargo fmt --check、clippy --locked --all-targets -- -D warnings、前端类型/生产构建与Tauri调试构建通过；日志runtime-probe-clippy.log、runtime-probe-debug.log。链接器仅报告创建lib/exp的中文信息，静态检查通过。
- Release/NSIS通过，runtime-probe-release.log；八项资源源/暂存字节、NSIS精确写入/删除目标核对通过，runtime-probe-staged.json。应用材料check仍321包/566份，react-remove-scroll-bar2.3.8已知原文缺口保留。未改项目或第三方许可证。

## 原生交互的失败、修正与证据

新增完整主流程检查：真实部署后修改一个已记录DLL的单字节，诊断预览必须显示actualPlayerVersion=null、playerIssue=file_conflict和1个变化文件；预览不导出文件。展开完整JSON供可见断言与截图，再从主界面请求受管理启动，必须拒绝且无启动确认预览；修改后的DLL仍原样保留，测试finally恢复自己的原字节后继续既有播放/设置/维护流程。

首轮 .local/e2e-artifacts/runtime-probe-native.log：主流程在大型恢复预览的确认按钮处超过默认10秒，指定目录组达180秒总时限；0通过、1失败、1取消，总计563.20秒。发现一个本次临时目录的MPV已创建、启动PID尚未写入日志，原有依赖started记录的清理漏掉它。按实际PID及完整映像路径核对后，仅结束该测试播放器，原运行器随后正常退出并输出错误；没有因观察超时重启运行器。

测试修正包括大型恢复预览确认等待90秒、指定目录预览90秒/组总时限300秒，以及停止测试应用后关闭CDP、按本次精确映像路径兜底清理孤儿播放器。test-process.ps1新增cleanup-image，仅在钉住进程句柄且MainModule.FileName与传入完整绝对路径一致时停止；同名的其他路径、不能确认映像的进程不处理。新增真实双fixture回归验证无PID日志时仍能清理目标、另一目录的同名进程存活且重复清理安全；单项通过1.657秒，runtime-probe-cleanup.log。

第二轮 runtime-probe-native-retry.log：指定目录ICEl4K通过78.74秒，清理回归通过1.598秒；主流程Lr3gTx在读取折叠details内的JSON时因空字符串解析失败，界面实际已显示未获得（file_conflict）及1个变化文件。修正测试先展开“查看将保存的完整 JSON”，不改业务实现或判定条件。失败页面和原始日志保留，未把失败算成通过。

第三轮主流程bIHzP4已通过新增DLL/诊断/启动拒绝检查，但在再次生成暂存清理预览时，确认按钮仍使用默认10秒等待而失败（133.31秒），日志runtime-probe-native-final.log。首次同类预览实际耗时10488毫秒，超过该默认值；补齐第二次确认及同类设置/受管理启动预览的90秒有界等待，不改变业务时限或拒绝断言。

最终主流程XO3hPl单项通过204.82秒（运行器205.16秒），日志.local/e2e-artifacts/runtime-probe-native-main.log；三类原生用例在分次运行中各自通过，不声称同一次全套通过。最终主流程第一次暂存预览10580毫秒；passed.json、runtime-integrity-diagnostics.json、runtime-integrity-blocked.png与既有播放/维护证据保存在.local/e2e-artifacts/mpvkit-native-XO3hPl。诊断实际为70个受管理文件、1个变化文件，actualPlayerVersion=null、playerIssue=file_conflict；后续设置、受管理真实MPV启动、GUI关闭后保存、旧代际拒绝及CLI重启等完整流程继续通过。主流程mode=fixture指初始外部播放器使用fixture；设置MPVKIT_TEST_ASSETS后组件安装与后续受管理播放使用固定真实MPV。不是视频/HDR/功耗验收。

指定目录ICEl4K及清理回归沿用第二轮各自通过的78.74/1.598秒结果；最终修正只影响主流程的读取与等待，不重复未受影响用例。所有失败日志保留。

## 本阶段产物与门槛（历史）

| 产物 | 字节 | SHA-256 | 签名 |
| --- | ---: | --- | --- |
| src-tauri/target/release/mpvkit.exe | 14771712 | aa8999e4b961f82a73470e956d7f7a149ce10ca3b2ae30e110e9dc4e4d3acd1e | NotSigned |
| src-tauri/target/release/bundle/nsis/MpvKit_0.1.0_x64-setup.exe | 3863689 | e36d7639779b3d4ae080fd580920e7fd6ea10ce016a95605237206f09828ea81 | NotSigned |

身份记录.local/v010-tests/release-artifacts-runtime-probe.json；74个产品输入快照runtime-probe-build-inputs.json，签名记录runtime-probe-signatures.json。Go声明阶段产物转为历史，新包未安装或发布。

本轮未改变画质、Lua或下载契约，不重跑未受影响的慢网、九组图形矩阵或全部六个显式入口；沿用对应历史证据并保留范围。探针见证覆盖本轮四类检查时已发生的变更；不是目标设备、实体HDR/功耗或隔离Windows验收。完整上游材料和W/P门槛仍未通过，releaseReady继续false；未提交、推送或发布。

后续同代码MIT原文补充重建了同路径产物；本报告保留修复/交互的真实阶段结果，当前产物及许可清单见[后续证据](2026-10-20-scroll-bar-license.md)。收束时102份自有文本/199本地链接及Git diff通过，74产品输入、2产物、15份Mantine原文和LICENSE均复核未变；此为后续许可资源变更之前的身份。
