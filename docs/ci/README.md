# CI/CD 与分发

当前 v0.2.0-alpha.2；实际验证和远端状态见[交接](../v0.2.0-alpha.2/plan.md)。配置以 [.github/workflows](../../.github/workflows) 为准。

## PR 检查

Checks 只监听目标 main 的 PR 和 merge_group，不重复监听 push。changes 识别纯文档修改并跳过编译；工作流、预览代码、打包许可和未知文件仍检查。固定汇总检查 **CI** 始终返回，构建变更必须前端及 Windows/Arch Rust 均成功，取消或意外跳过不能通过。

前端只构建一次，dist 供两端复用；PR 不使用签名密钥、不打发布包。Arch 测试环境需安装系统 mpv，分发包中仍为可选依赖。

main 的 ruleset 如启用，应要求 PR 和 GitHub Actions 的 `CI`；不要把可能跳过的矩阵任务分别设为必需。当前流程不增加人工发布批准步骤。

## 自动发布与重试

1. main 每次推送（含 PR 合并）在事件精确 SHA 核对 package/Tauri/Cargo 版本及 `docs/v<版本>/notes.md`。日志用极简英文 Markdown。
2. `v<版本>` 已存在则跳过；仅真实 404 才创建 tag。并发时只有创建成功的一次继续，不移动 tag。
3. 同一工作流直接继续双平台构建、正式签名、完整性校验与公开发布，不依赖 GITHUB_TOKEN 创建 tag 再触发其他事件。
4. REST 创建版本 Release 时明确 `draft=false`，再上传已校验资产，齐备后更新公开 `updates/channels.json`。alpha/beta/rc 为 Pre-release、latest=false；稳定版才更新 stable/latest。禁止先创建草稿，即使稍后公开也不符合要求。

不要使用 `gh release create <tag> <assets...>`：它会[隐式先建草稿](https://github.com/cli/cli/blob/v2.103.2/pkg/cmd/release/create/create.go)。唯一历史例外是将已有 alpha.1 草稿公开；新版本发现意外草稿则失败。

失败后在 Actions → Release → Run workflow，选择 main、既有 tag：`release` 重建并公开，`publish` 复用完整资产；如尚未创建版本 Release，填 `build_run` 为已有完整 publish-assets 的运行 ID，直接复用 Actions 产物。也可只重跑失败任务；整个自动流程重跑会因 tag 已存在而跳过。源码取 tag，prepare/publish/验证脚本取工作流提交，允许修复流程而不改历史资产。

已有资产逐项核对 GitHub SHA-256，重试只补传缺项，不覆盖版本资产。更新任务串行，保留另一渠道并拒绝倒退；仅认证 API 确认 updates Release 不存在才初始化。已有索引缺失、损坏或不可读时失败，不能清空重建。

创建 Release 的 `target_commitish` 固定为 main（API 只接受分支或 SHA，不接受 tag 名）；版本 tag 已经校验存在，不受该字段影响。首次创建 updates 元数据 tag 也锚定 main，避免 GITHUB_TOKEN 为旧工作流提交隐式创建新 tag 时的权限限制。不为此增加 PAT；跨运行读取构建产物仅需 actions:read。

## Secrets 与有效检查

| 环境 | 配置 |
| --- | --- |
| release-build | Secret `TAURI_SIGNING_PRIVATE_KEY`：完整正式私钥；有密码才设置 Secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |
| release-build、release-publish | Variable `MPVKIT_UPDATER_PUBLIC_KEY`：同一正式公钥 |

私钥不能放 Variables 或提交仓库。Windows 构建脚本将公钥注入临时 Tauri 配置，同时编入程序。release-publish 不需要私钥。环境仅供可信 main 发布使用；全自动发布不设置人工审批。

保留版本/tag、测试、缺失许可原文、SHA256SUMS、Minisign、updater 和渠道契约检查。用户于 2026-10-27 明确删除完整审阅状态的硬阻断，`releaseReady=false` 仅保留在材料记录中，不代表已完成审阅，也不再阻止发布。没有其他人工状态 gate。

## 本地构建

- Windows：先 `pnpm build`，设置签名环境后 `node scripts/build-windows.mjs`。安装器采用[用户级一键模板](../../packaging/windows/README.md)。
- Arch：先 `pnpm tauri build --no-bundle`，非 root 执行 `node scripts/release.mjs arch <全新输出目录>`；makepkg 只封装，不重复编译。
- 双端汇总：`node scripts/release.mjs manifest <目录>` 生成 updater.json 和 SHA256SUMS；公开前校验签名及全部资产。默认开发见[必要测试](../testing.md)。

AUR 注册暂时关闭，未接入发布/自动同步。自动发布 `arch_package=none`，Arch 用户从 Releases 下载 `.pkg.tar.zst` 后用 `pacman -U` 安装/升级。普通 `pacman -Syu` 不更新本地下载包。配方归档保留，真正上线 AUR 后才选择对应包名。

## 工作流依赖

2026-10-27 核验官方最新稳定版，固定完整 SHA，使用 Node 24：checkout 7.0.1、setup-node 7.1.0、cache 6.1.0、upload-artifact 7.0.2、download-artifact 8.0.2。Arch base 镜像固定摘要，应用 Node/pnpm/Rust 不随之升级。Actionlint 1.7.12 通过；本机未安装 shellcheck。
