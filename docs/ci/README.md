# CI/CD 与分发流程

当前版本 **0.2.0-alpha.1，第一个 alpha 预览版**；0.2.0 是后续首个正式版目标。用户于 2026-10-27 明确解除旧 CI/CD 与安装包禁令，随后授权清理、提交并推送当前整批更新；不包含公开发布 Release 或 AUR。

前一轮惰性模板已转为 `.github/workflows/ci.yml` 和 `release.yml`，重复模板已删除，实际配置通过 actionlint 1.7.12（本机未安装 shellcheck）。Action commit 与 Arch OCI 摘要于 2026-10-27 从官方远端只读核对。远端运行、冷/热缓存耗时和实际发布仍须另验。

## PR 与 main 分支保护

日常 Checks 只监听目标 main 的 `pull_request` 和合并队列 `merge_group/checks_requested`；不监听 push，因此分支推送/PR 更新只运行一套，合并 main 后也不重复构建。合并队列针对组合后的提交必须单独验证，不能用 PR 的旧结果替代。PR 更新自动取消前一轮，同一轮前端只构建一次，产物供 Windows/Arch Rust 任务并行复用。PR 不打安装包、不使用发布密钥；Release 仍须手动触发。

工作流本身不做路径过滤。轻量 changes 任务用 Git 比较完整变更，只有 README、AGENTS、docs 下 Markdown 和 vendor 文档跳过重型任务；预览代码、打包许可、工作流及未知文件均正常检查。检测使用完整历史的树信息及 blob:none 过滤，避免取回历史大文件；禁止重命名检测，源文件移入文档不能漏检。

固定汇总检查名为 **`CI`**。它始终运行：文档变更必须确认重型任务按预期 skipped；其他变更必须前端和两个 Rust 平台均 success。检测失败、编译/测试失败、取消或意外跳过均不算通过。不要把可能跳过的矩阵任务分别设为必需检查，也不要在工作流顶层加 paths-ignore，否则文档 PR 可能一直 Pending。

GitHub Settings → Rules → Rulesets 中，针对 main 设置 Require a pull request before merging 与 Require status checks to pass，必需检查选择 `CI`、来源 GitHub Actions；是否要求分支最新或使用合并队列由维护者选择。工作流不能替代仓库规则本身。2026-10-27 只读 API `/rules/branches/main` 返回空列表，当前没有生效的 ruleset；本批不擅自修改远端合并政策。首次 PR 运行后可从检查列表选取 `CI`。

## 启用前提

启用配置并不表示远端环境已配置。`release-build` 环境只供可信版本 tag，配置 MPVKIT_UPDATER_PUBLIC_KEY 变量、TAURI_SIGNING_PRIVATE_KEY 与可选密码 Secret；`release-publish` 独立保护并要求人工批准，不向不可信 PR 提供密钥。仓库不保存正式私钥；离线备份由维护者保管。

构建只产生草稿。维护者验收双平台安装/升级与许可材料后，再运行 promote；不重复编译。`verify-release.mjs gate` 当前按真实 releaseReady=false 拒绝推广，完整审阅后再更新来源材料及生成规则。首次由维护者明确创建 `updates` 元数据 Release，channels.json 为 `{"schemaVersion":1,"stable":null,"preview":null}`；不在失败时自行重置索引。

AUR 配方由 Arch 构建产物中的 aur-recipe.tar.gz 提供，维护者审阅并发布后才在推广时选择 mpvkit-bin；未发布时选 none。流程不自动推送 AUR，避免为首版引入额外凭据和发布系统。

Linux 应用只提醒并展示日志，提示用户用已选择的 yay/paru 等 AUR 助手手动更新 MpvKit，再重新启动。`在 AUR 查看` 仅打开已发布包页面。预览开关只影响应用内通知，不限制包管理器选择版本；普通 pacman -Syu 不会更新 AUR 包。

## 本地命令

Windows 先 `pnpm build`，配置专用签名环境后执行 `node scripts/build-windows.mjs`；默认开发仍用 `pnpm tauri build --debug --no-bundle`。正式签名同时用于打包和运行时内嵌公钥，不能只设置一个 Rust 编译变量。

Windows 包采用[一键安装模板](../../packaging/windows/README.md)：启动即显示进度，固定 `%LOCALAPPDATA%\Programs\MpvKit`，无语言/确认/目录向导，成功自动关闭。当前用户权限，不将安装界面隐藏；失败和系统安全提示仍保留。只为安装体验定制锁定上游模板，不另写安装引擎。

Arch 原生依赖齐备时执行 `pnpm tauri build --no-bundle`，再以非 root 用户运行 `node scripts/release.mjs arch .local/release`。输出目录需全新，避免混入旧资产；makepkg 只封装一次编译的程序。手动下载包使用 pacman -U，AUR helper 更新使用其自身流程，普通 pacman -Syu 不会订阅 AUR。

双端产物汇总后执行 `node scripts/release.mjs manifest .local/release`；要求一份 Windows 安装器及 .sig、一份版本 Linux tar.zst 与 Arch 包。生成 updater.json 与 SHA256SUMS。签名与完整性校验后，`node scripts/release.mjs promote <现有channels.json> <none|mpvkit-bin|mpvkit-preview-bin>` 仅写本地索引，拒绝渠道倒退；上传由受保护流程显式处理。
