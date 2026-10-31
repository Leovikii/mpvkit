# beta.1 本地验收证据

日期：2026-10-31。环境：CachyOS x86_64、KDE Wayland、GTK/WebKit；固定 Node 24.19.0、pnpm 11.25.0、Rust 1.98.1。用户本轮授权提交/推送 dev，暂不合并，待 Windows 测试，不发布。

## 最终程序

- 入口：`.local/beta1-test/run.sh`；ELF：同目录 `mpvkit`，由本轮源码 Tauri debug no-bundle 构建并 strip。
- SHA-256：`1cbb0e97527e29ccf5bb0ccd581323c7a703e61fee8eb874fd03a8162c794eb3`。
- 固定组件随 `assets/` 提供，应用写入位于 `session/data/`，完整配置删除仅作用于 `session/data/config/mpv`。不预写语言/主题，不覆盖 XDG_CONFIG_HOME；默认跟随系统。真实包管理器及应用更新安装被测试模式阻断。
- `bash -n` 通过；实际原生验收运行最终复制的 ELF 和自带资产，不依赖 Vite。

## 自动检查

在仓库根加载 `.local/linux-v018/env.sh` 后运行：

| 命令 | 结果 | 耐久日志（均在 .local/beta1-test/evidence/） |
| --- | --- | --- |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | 通过 | 无输出 |
| cargo test --manifest-path src-tauri/Cargo.toml --locked | 59 通过、5 忽略 | tests.log |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings | 通过 | clippy-current.log |
| pnpm build；pnpm tauri build --debug --no-bundle（复用 dist，跳过重复前端构建） | TypeScript / Vite / Rust 构建通过 | ui-build.log、native-build.log |
| pnpm test:frontend | 1 通过 | frontend.log |
| node --test scripts/release.test.mjs | 17 通过，仅 mock API，无实际发布 | release-contract.log |
| node scripts/collect-app-notices.mjs --linux --check | 通过 | notices.log |
| node scripts/collect-app-notices.mjs --check | 通过 | notices-windows.log |
| node docs/v0.1.6/preview/build.mjs；使用本地 Chromium 运行 check.mjs | 通过 | preview-build.log、preview-test.log |

默认 Rust 测试覆盖实际风险，包括核心失败的阶段编排、等待真实进程退出及有界输出、目录快照/过期/竞态/路径/锁/部分删除、事务恢复、日志配额/轮转/过期/链接/争锁/存储失败降级。最初在沙箱内运行时，两项回环端口绑定被拒绝；获准运行后全套通过，不把受限失败计为产品失败或通过。

Vite 保留约 532 kB 主 chunk 超过 500 kB 默认阈值的警告；本轮未为消除提示增加拆分框架。

## 系统装饰修正

用户指出窗口装饰和阴影与 KDE 设置不一致。已删除 GTK HeaderBar、图标拼装、标题栏高度/颜色/阴影样式，使用 GTK 默认服务端装饰协商。当前 KWin 配置只读核对为 Klassy，未更改系统设置。Wayland 日志确认 request_mode(2)、mode(2)，没有客户端装饰 request_mode(1)；截图人工复核为系统 Klassy 标题栏和窗口边框，阴影由 KWin 负责。

修正后重新运行 Rust fmt/clippy、Tauri debug 构建、真实 GTK/WebKit 流程和单实例测试，均通过。未重复与本次原生外壳无关的 Rust 业务/前端/发布检查，原有通过日志保留。最初测试被仍在运行的旧测试实例唤回；关闭旧隔离实例后继续。随后一轮被 KWin 恢复旧外框，内容552×769，作为尺寸覆盖事实保留；显式调整后的布局通过，最终 ELF 不启用尺寸调整也通过560×760。

新证据位于 `.local/beta1-test/evidence/`，初次 GTK CSD 证据归档为 `evidence-gtk-csd/`，不能当作当前装饰证明。独立 ELF 未安装 desktop/icon 资产，截图标题栏图标为默认 Wayland 图标；正式包图标关联仍未测，不向宿主注册资产伪造通过。

## 内容区统一修正

用户反馈窗口套层过多。移除页面外围 8px 留白、描边、大圆角及独立底色，客户区使用统一窗口基底；底部固定操作区沿用页面背景，设置分组保留一层轻边框。清理高级选项重复顶部边线。产品样式与实际 App 离线预览同步。

本次仅修改展示 CSS，重新运行 TypeScript/Vite、离线实际 App 布局/交互检查和 Tauri debug 构建。浏览器双语、深浅主题、480/560/720 宽、200% 文字、减少动态效果/高对比检查通过；最终复制并 strip 的 ELF 使用自带资产执行 GTK/WebKit 流程，浅深截图人工复核通过。初始内容560×760、最小480×600、系统装饰及原有安装/保存/卸载流程通过。未重复无关的 Rust 业务和单实例测试；先前结果与未测边界保留。

最新证据为 `evidence/`，前次带大圆角底板的系统装饰截图归档为 `evidence-ssd-layered/`，不作为当前布局截图。旧 GTK HeaderBar 证据仍在 `evidence-gtk-csd/`。

## 操作对齐修正

用户指出主页与设置页按钮左右散落。定位为普通设置行、更新区与520px窄屏覆盖规则冲突；统一为共享弹性行，说明/状态居左、控件靠右，换行后仍保持右边缘。MPV 状态移到版本说明，管理按钮独立靠右；选择框宽度统一，开关不占满整行。页内动作、日志/GitHub成组靠右，与设置行控件对齐；主页底部主要动作位于组内最右。长按钮文字允许换行。未改变包管理、删除范围或确认流程。

重新运行 TypeScript/Vite、前端测试、实际 App 预览及 Tauri debug 构建。预览增加两页控件实际右边缘检查，480/560/720、双语、浅深色和200%文字全部通过，并保存两页截图。首轮暴露了独立预览工具栏在放大文字后遮挡页头的问题，已限制该工具栏尺寸并复验；没有用强制点击绕过遮挡。原生 GTK/WebKit 验收也新增设置页截图；最后一版 ELF 的两页浅深截图人工核对通过。未重跑无关 Rust 业务和单实例检查，先前结果保留。

中间一次 KWin 恢复用户调整的497宽窗口，随后显式调整到560的布局运行通过；此运行不计无调整首启通过。最终测试包运行的实际尺寸标记见 result.json。

## 原生窄窗与任务

最终命令设置 `MPVKIT_TEST_EXECUTABLE` 指向 `.local/beta1-test/mpvkit`，`MPVKIT_TEST_ASSETS` 指向其 `assets/` 后运行 `node scripts/test-linux-ui.mjs`，**未设置 MPVKIT_TEST_RESIZE_WINDOW**。

结果源 `/tmp/mpvkit-beta1-ui-kRlqYc/result.json`，耐久副本 `evidence/result.json`：

- 调整前首启内容 560×760，devicePixelRatio=2；截图期间宽度仍为 560。最小内容尺寸实测 480×600。最后单独通过临时测试脚本缩小窗口验证最小值，不影响首启证据。
- 偏好原值 language/system、theme/system；当前桌面语言 zh-CN，实际解析 zh-Hans。没有依靠测试启动脚本强制中文。
- 草稿切页保留，准备及取消不写配置；增强与配置统一安装；保存不改健康 thumbfast 文件时间；一键卸载删除未知手改和 watch_later；测试模式拒绝核心卸载后仍清理已确认配置，结果分别反馈。
- 两页无侧栏、组件大按钮、裸露仓库 URL、版权许可证行和诊断路径输入；固定 app_id、内容材质及 KWin 服务端装饰通过。系统标题栏与内容区不再强制同色。截图仅本程序窗口，浅深截图人工复核通过。

本机 Remember Window Positions 插件曾在早期测试中恢复 1027 宽。早期失焦/宽窗截图及开启显式尺寸调整的运行不作为无干预首启证据。本次最终运行 `layoutExplicitlyResized=false`、`windowManagerOverride=false`。应用默认560、最小480在通用与 Linux 覆盖配置一致；不改宿主插件或全局规则。窗口管理器以后仍可能覆盖尺寸。

## 单实例

`python3 scripts/test-single-instance.py` 在私有会话总线上执行两轮，每轮一个主实例和六个并发重复启动。12 个重复进程均返回0，未创建 Wayland 窗口；主实例始终拥有唯一 GUI 总线名，退出后可重新启动。实现收到重复启动后恢复、显示并聚焦原窗口。播放保存与核心命令 helper 在单实例前分派，未被拦截。缺会话总线时 GUI 明确失败而 helper 仍独立执行。

证据源 `/tmp/mpvkit-single-instance-2ljor1q5/result.json`，副本 `evidence/single-instance.json`、`single-instance.log`。Windows 复用相同 Tauri 插件与唤回回调，但本轮未在 Windows 原生执行。

## 未测边界

Windows 编译、WebView2、控制台及 WinGet 原生交互；实际 Polkit/Pacman 安装和卸载；两端跨版本更新成功后自动重启；正式 Arch 包的 Wayland 任务栏/切换器图标；真实多屏 DPI 和原生辅助技术。仅完成 Windows updater 重启选项及 Linux 成功核验后释放锁并 request_restart 的代码接通，不写成实际更新通过。

没有修改真实 MPV 配置、默认应用、宿主桌面注册或系统软件包。下一步为 Windows x64 接手当前 dev 提交，执行交接中的编译、隔离原生及独立VM验证，补齐上述边界。见[方案与交接](plan.md)。

提交交接前已同步 Windows 测试的更新区旧选择器及窄窗矩阵，fixture 编译入口改为 PATH 中 rustc。Node 语法检查通过；这些更改不构成 Windows 实际运行通过。Windows 接手顺序及本轮不合并约束见 [Windows 测试交接](plan.md#windows-测试交接)。
