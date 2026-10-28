# CI/CD 与分发

当前 v0.2.0-alpha.3，验证与实际发布见[交接](../v0.2.0-alpha.3/plan.md)。配置以 [.github/workflows](../../.github/workflows) 为准。

## PR 检查

Checks 监听目标 main 的 PR 和 merge_group。纯文档可跳过构建；工作流、预览代码、许可材料和未知文件仍检查。汇总 **CI** 必须成功，取消或意外跳过不能通过。前端构建一次，dist 供 Windows/Arch Rust fmt、clippy、test 共用；PR 不使用正式签名密钥。

Arch 容器额外构建 debug 程序、makepkg、真实 Pacman 只读事务检查及隔离系统根安装/重装/卸载。隔离安装使用 --nodeps/--noscriptlet，只验证包载荷与文件归属，不代表真实桌面、依赖首次拉取或 Polkit 验收。系统 mpv 为测试依赖，分发包中仍为可选依赖。

## 自动发布与重试

1. main 推送在精确事件 SHA 核对 package/Tauri/Cargo 版本及极简英文 `docs/v<版本>/notes.md`。
2. tag 已有则跳过；只有真实 404 才创建。不移动 tag。并发仅创建成功者继续。
3. 双平台测试、编译和正式签名后汇总六个发布资产：Windows NSIS + .sig，Arch pkg.tar.zst + .tauri.sig，SHA256SUMS，latest.json。诊断文本只保留 Actions 产物，不分发 tar/配方归档。
4. REST 从创建起明确 draft=false，alpha/beta/rc 保持 Pre-release、latest=false。先上传包/签名/SHA256SUMS，读取 API 核验全部摘要后，最后上传标准 latest.json；客户端只选择必要资产完整的版本。不创建 updates Release 或渠道索引。
5. 匿名读取 manifest 并比较实际字节摘要。成功构建不等于公开发布完成。

禁止 `gh release create <tag> <assets...>` 的隐式草稿流程。发现已有草稿、资产变化、提前上传的不完整 manifest 时失败；不覆盖已有资产。失败在 Actions → Release → Run workflow 选择 main 与既有 tag；release 重建，publish 复用完整 publish-assets（填 build_run），或已有 Release 资产。重试只补缺项。版本源码取 tag，prepare/publish/验证脚本取工作流提交，以便修复发布而不移动 tag。同一版本互斥；发布动作串行，不增加人工批准门槛。

## 签名与材料

| 环境 | 配置 |
| --- | --- |
| release-build | Secret TAURI_SIGNING_PRIVATE_KEY；有密码才设 TAURI_SIGNING_PRIVATE_KEY_PASSWORD |
| release-build、release-publish | Variable MPVKIT_UPDATER_PUBLIC_KEY，同一正式公钥 |

两端公钥编入程序，私钥只用于可信发布构建，不入库、不用于 PR。Linux 直接用现有 Tauri CLI 签原生包；.tauri.sig 是 Minisign 格式，避免 Pacman 把相邻 .sig 当作 OpenPGP。manifest 的 signature 字段保持 Tauri 标准。发布前用 minisign 验证两端真实签名，SHA256SUMS 及资产契约均需通过。系统 Pacman 签名策略不修改。

扩大 updater 到 Linux 后更新两平台依赖材料及 collector --check；缺失材料继续阻断。releaseReady=false 是完整许可审阅仍未完成的事实，不作为人工发布门槛。

## 本地构建

- Windows：先 pnpm build，设置正式签名环境后运行 `node scripts/build-windows.mjs`；[用户级一键模板](../../packaging/windows/README.md)。
- Arch：注入公钥后 Tauri build --no-bundle；非 root `node scripts/release.mjs arch <新目录> [executable]`，仓库 packaging/arch/PKGBUILD 只封装 stage/usr，不重复编译。executable 仅供 CI debug 包验收，正式默认 target/release/mpvkit。
- 双平台汇总：`node scripts/release.mjs manifest <目录>` 生成 latest.json 与 SHA256SUMS；`node scripts/verify-release.mjs <目录>` 核对六项资产及日志。
- Arch 首次下载本地包再 pacman -U，后续应用内下载验签/预览/原生授权安装；无 AUR 发布、参数、配方分发或自动同步。普通 pacman -Syu 不更新本地包。

Actions 继续固定完整 SHA 与 Node 24.19.0 / pnpm 11.25.0 / Rust 1.98.1；Arch base 镜像固定摘要。Actionlint 本地检查，系统/桌面测试边界见[测试入口](../testing.md)。
