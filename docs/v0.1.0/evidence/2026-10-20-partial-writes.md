# 部分写入失败清理与安全重试

日期：2026-10-20。任务：M2/D、F05/F06/F10。Windows 11 x64 build 26200，Node24.19.0/pnpm11.25.0/Rust1.98.1；未提交、推送、发布或改变工具链/依赖。

## 问题与修复

write_new在create_new成功之后直接传播write_all/sync_all失败，可能留下不完整文件；atomic_write调用它写临时文件，部分失败时尚未进入替换阶段的清理。首次owner.json写失败也会留下不能被重新认领的控制目录，影响重试。

修复集中在共享写入入口：只有create_new已经成功且后续写入/同步失败时，关闭句柄后尽力移除本调用新建的半成品。create_new因目标已存在而失败时，保持已有文件。首次创建控制目录后owner.json失败，则仅尝试移除本调用刚创建的空目录；目录非空、删除失败或预先存在时均保留，不做递归清理或按名称接管。

磁盘/权限错误仍报告write_failed。清理不是成功保证：若文件删除也被系统拒绝，仍返回失败并保留内容；事务pending与备份遵循既有恢复流程，不声称多文件原子性。

## 新增验证

三个自动回归入口：

- partial_writes_remove_only_created_files_and_preserve_existing_targets：重复创建保护；部分新文件清理；原子替换前部分写入失败时已有目标不变，临时文件移除。
- partial_staging_backup_and_commit_writes_preserve_success_and_allow_recovery：分别在暂存、备份、安装记录提交中写入真实3字节前缀后报错；提交及回滚两次写失败保留pending，解除故障后恢复，再恢复上一成功版本。
- partial_owner_record_write_can_retry_without_taking_over_unknown_directories：首次归属记录部分失败后可重试；已有未知目录与用户文件始终保留。

每项覆盖Windows ERROR_DISK_FULL(112)和ERROR_ACCESS_DENIED(5)。线程局部、仅测试编译的注入按目录/文件与跳过计数命中实际write_new，RAII在失败/结束后撤销，不影响其他并行测试。模拟写错误并检查真实文件和事务状态；没有填满磁盘、修改ACL或操作开发者配置。

## 命令与结果

~~~powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
pnpm tauri build --debug --no-bundle
# 显式设置MPVKIT_TEST_ASSETS为已校验研究目录后：
pnpm test:e2e
pnpm tauri build --bundles nsis
~~~

fmt/clippy通过。当前共61项后端测试：55项普通通过、0失败，6个真实入口本轮未由cargo默认执行；4.27秒。日志 .local/v010-tests/partial-write-unit.log。此前58项含6个真实入口的完整回归作为历史保留，不冒称本轮61项全量都已运行。

前端类型/生产构建、Tauri调试程序与Release/NSIS构建通过。安装包未签名、未安装或发布；身份见[功能证据](2026-10-20-v010-functional.md)末尾。两组原生交互全部通过，2通过/0失败/0跳过，209.56秒；.local/e2e-artifacts/partial-write-native.log。主流程ff8CwC、指定配置目录KgOi7w，含固定真实组件部署/许可、GUI关闭后保存、两个真实播放器竞争保存、旧代际拒绝、指定目录原件恢复与真实CLI。发现/手动入口仍使用模拟播放器，受管理链使用固定真实MPV，fixture标签不表示全部由模拟程序完成。

本轮初次新测试误用control(root, "")，被正确拒绝为空相对路径；改为测试内明确root.join(CONTROL)，没有放宽生产校验。静态检查曾指出测试分支去除后的立即调用闭包多余，已改写并重新通过，未屏蔽warning。

## 边界

注入测试不能代替真实满磁盘、ACL/硬件损坏或系统掉电实验；它验证的是相同写入错误分支及恢复不变量。未知目录仍要求明确归属，不为重试删除整个已有环境。

隔离Windows的安装/卸载/默认应用，目标设备/实体HDR/画质/功耗，以及完整分发材料/O09授权仍未完成。releaseReady保持false。

文档检查：77份UTF-8/LF文本、76个本地链接及Git diff通过；应用依赖许可清单--check与固定uosc只读构建信息复验仍通过。
