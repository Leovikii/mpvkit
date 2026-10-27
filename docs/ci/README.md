# CI/CD 与分发流程

当前版本 **0.2.0-alpha.1，第一个 alpha 预览版**；0.2.0 是后续首个正式版目标。用户于 2026-10-27 明确解除旧 CI/CD 与安装包禁令，随后授权清理、提交并推送当前整批更新；不包含公开发布 Release 或 AUR。

前一轮惰性模板已转为 `.github/workflows/ci.yml` 和 `release.yml`，重复模板已删除，实际配置通过 actionlint 1.7.12（本机未安装 shellcheck）。2026-10-27 首次 PR 运行发现 Arch 测试环境遗漏系统 MPV，已同步修复两条工作流；运行结果见 [PR #1 检查](https://github.com/Leovikii/mpvkit/pull/1/checks)，冷/热缓存耗时和实际发布仍须另验。

## 工作流依赖与版本

Linux 默认 Rust 测试会实际查询 `/usr/bin/mpv`，因此 CI 和 Release 的 Arch 构建环境均安装 `mpv`。这只补齐测试环境依赖，分发包中的 `mpv` 仍为 optdepends，保留无核心启动能力。不能通过跳过原生目录或配置保护测试来绕过缺失依赖。

2026-10-27 按各官方仓库 `releases/latest`、tag 对象及对应 `action.yml` 核验以下最新版，工作流固定完整提交 SHA，并注释版本，全部使用 Node 24 运行时：

| Action | 已核验版本 |
| --- | --- |
| checkout | [v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1) |
| setup-node | [v7.1.0](https://github.com/actions/setup-node/releases/tag/v7.1.0) |
| cache | [v6.1.0](https://github.com/actions/cache/releases/tag/v6.1.0) |
| upload-artifact | [v7.0.2](https://github.com/actions/upload-artifact/releases/tag/v7.0.2) |
| download-artifact | [v8.0.2](https://github.com/actions/download-artifact/releases/tag/v8.0.2) |

Node 24 Actions 最低要求 runner 2.327.1；本次日志中的托管 runner 为 2.338.0，满足要求。checkout 保留 `persist-credentials: false`；触发器不涉及 v7 默认阻止的不安全 fork checkout。现有 pnpm 项目不启用 setup-node 的自动 npm 缓存；上传沿用默认归档、下载沿用自动解压，并保留 v8 默认的摘要不符即失败，不降低完整性检查。Arch `base` 官方当前摘要仍为 `sha256:52dfbd5afb23a220d5ee92dafcb4c788d96c026ed04179890c23adef4c897fa0`，无需变更。应用自身 Node/pnpm/Rust 工具链继续使用既定版本。

## PR 与 main 分支保护

日常 Checks 只监听目标 main 的 `pull_request` 和合并队列 `merge_group/checks_requested`；不监听 push，因此分支推送/PR 更新只运行一套 Checks。合并队列针对组合后的提交必须单独验证，不能用 PR 的旧结果替代。PR 更新自动取消前一轮，同一轮前端只构建一次，产物供 Windows/Arch Rust 任务并行复用。PR 不打安装包、不使用发布密钥；合并后的版本发布交由下述 Release 流程。

工作流本身不做路径过滤。轻量 changes 任务用 Git 比较完整变更，只有 README、AGENTS、docs 下 Markdown 和 vendor 文档跳过重型任务；预览代码、打包许可、工作流及未知文件均正常检查。检测使用完整历史的树信息及 blob:none 过滤，避免取回历史大文件；禁止重命名检测，源文件移入文档不能漏检。

固定汇总检查名为 **`CI`**。它始终运行：文档变更必须确认重型任务按预期 skipped；其他变更必须前端和两个 Rust 平台均 success。检测失败、编译/测试失败、取消或意外跳过均不算通过。不要把可能跳过的矩阵任务分别设为必需检查，也不要在工作流顶层加 paths-ignore，否则文档 PR 可能一直 Pending。

GitHub Settings → Rules → Rulesets 中，针对 main 设置 Require a pull request before merging 与 Require status checks to pass，必需检查选择 `CI`、来源 GitHub Actions；是否要求分支最新或使用合并队列由维护者选择。工作流不能替代仓库规则本身。2026-10-27 只读 API `/rules/branches/main` 返回空列表，当前没有生效的 ruleset；本批不擅自修改远端合并政策。首次 PR 运行后可从检查列表选取 `CI`。

## main 自动进入 CD

`release.yml` 监听 `push` 到 main，覆盖普通合并、squash 和 rebase，也覆盖直接推送 main。每次使用事件中的精确提交 SHA 检出源码，核对 package.json、Tauri 与 Cargo 版本及本版日志 `docs/v<版本>/notes.md`。日志统一为极简英文 Markdown：一个版本标题、数条用户可见变更，不写开发交接或验收台账。

1. 查询 `v<版本>` tag；存在则正常结束，不重建、不移动既有 tag。文档合并或未升版代码合并仍执行轻量版本检查。
2. 不存在则用当前 main 事件 SHA 创建 tag，直接将输出传给本次工作流的 Windows / Arch 构建与草稿任务。只把真实 404 视为缺 tag；权限、网络或服务错误均失败，不伪装成缺失。并发同版本只有建 tag 成功的一次继续构建。
3. 完成签名和双平台资产收集后创建 Release 草稿；alpha/beta/rc 标记 prerelease，草稿不更新 latest 或 channels.json。不同 main 提交互不取消，推广任务另行串行执行以保护更新索引。

自动建 tag 使用最小任务权限的 `GITHUB_TOKEN`。GitHub 不会因这个 token 创建 tag 而再触发普通 tag push 工作流，所以流程直接用 `needs.prepare.outputs.tag` 继续执行，不依赖第二次事件，也不需要 PAT。

已有 tag 的构建若失败，可在 Actions → Release → Run workflow，选择 main 工作流、原有 tag 和 `build-draft` 重试；也可重跑失败的构建任务。重新运行整个自动工作流会因 tag 已存在而跳过。不要删除或移动 tag 来重试。`promote` 仍只允许手动触发。

## 启用前提

2026-10-27 首次自动 CD 已成功创建 `v0.2.0-alpha.1` tag，Arch 构建完成，Windows 在签名打包前因缺少公钥失败；两个发布环境现已创建，并已配置同一正式公钥变量 `MPVKIT_UPDATER_PUBLIC_KEY`。公钥由 `scripts/build-windows.mjs` 注入临时 Tauri 配置并编入安装包，源码的开发配置仍保持空公钥，无需手动修改源码或提交密钥文件。

`release-build` 必须在 **Environment secrets** 保存 `TAURI_SIGNING_PRIVATE_KEY`，值为私钥文件完整内容；不能放入 Environment variables。私钥有密码才添加 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` Secret；无密码时省略，工作流取空字符串。`release-publish` 只需同名公钥变量，不需要私钥。所有正式版本持续使用同一密钥对，私钥在本机另行安全备份。

本轮已由用户纠正私钥误存 Variable 的问题，核对 Secret 名称后重跑失败任务，[第二次执行](https://github.com/Leovikii/mpvkit/actions/runs/40502127238/attempts/2) 的 Windows 签名构建及草稿任务成功，复用此前成功的 Arch 产物。独立 Minisign 验证安装器签名与版本注释通过。草稿上传曾因 `*.*` 漏掉无扩展名 `SHA256SUMS`，现有草稿已补齐，工作流两条创建分支均显式上传该文件；全部 9 项摘要及 updater 元数据核验通过。草稿仍未公开，不能将签名构建成功视为安装升级或公开推广验收。

两个环境仍需按实际 GitHub 设置配置保护规则，允许 main 上的自动 / 手动流程使用可信版本 tag；`release-publish` 独立保护并要求人工批准，不向不可信 PR 提供密钥。部署分支规则按工作流事件的 main 设置，不能因实际检出 tag 而只允许 tag 事件。仓库不保存正式私钥；重试前只核对 Secret 名称，实际签名及公私钥匹配以构建和验签结果为准。

构建只产生草稿。维护者验收双平台安装/升级与许可材料后，再运行 promote；不重复编译。`verify-release.mjs gate` 当前按真实 releaseReady=false 拒绝推广，完整审阅后再更新来源材料及生成规则。首次由维护者明确创建 `updates` 元数据 Release，channels.json 为 `{"schemaVersion":1,"stable":null,"preview":null}`；不在失败时自行重置索引。

AUR 配方由 Arch 构建产物中的 aur-recipe.tar.gz 提供，维护者审阅并发布后才在推广时选择 mpvkit-bin；未发布时选 none。流程不自动推送 AUR，避免为首版引入额外凭据和发布系统。

Linux 应用只提醒并展示日志，提示用户用已选择的 yay/paru 等 AUR 助手手动更新 MpvKit，再重新启动。`在 AUR 查看` 仅打开已发布包页面。预览开关只影响应用内通知，不限制包管理器选择版本；普通 pacman -Syu 不会更新 AUR 包。

## 本地命令

Windows 先 `pnpm build`，配置专用签名环境后执行 `node scripts/build-windows.mjs`；默认开发仍用 `pnpm tauri build --debug --no-bundle`。正式签名同时用于打包和运行时内嵌公钥，不能只设置一个 Rust 编译变量。

Windows 包采用[一键安装模板](../../packaging/windows/README.md)：启动即显示进度，固定 `%LOCALAPPDATA%\Programs\MpvKit`，无语言/确认/目录向导，成功自动关闭。当前用户权限，不将安装界面隐藏；失败和系统安全提示仍保留。只为安装体验定制锁定上游模板，不另写安装引擎。

Arch 原生依赖齐备时执行 `pnpm tauri build --no-bundle`，再以非 root 用户运行 `node scripts/release.mjs arch .local/release`。输出目录需全新，避免混入旧资产；makepkg 只封装一次编译的程序。手动下载包使用 pacman -U，AUR helper 更新使用其自身流程，普通 pacman -Syu 不会订阅 AUR。

双端产物汇总后执行 `node scripts/release.mjs manifest .local/release`；要求一份 Windows 安装器及 .sig、一份版本 Linux tar.zst 与 Arch 包。生成 updater.json 与 SHA256SUMS。签名与完整性校验后，`node scripts/release.mjs promote <现有channels.json> <none|mpvkit-bin|mpvkit-preview-bin>` 仅写本地索引，拒绝渠道倒退；上传由受保护流程显式处理。
