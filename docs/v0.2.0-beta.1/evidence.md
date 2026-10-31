# beta.1 本地验收证据

2026-10-31 Windows 续验已完成自动化部分，见[本轮记录](#windows-自动化续验2026-10-31)。以下 Linux 记录和当时的 Windows 待测状态按历史保留。

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

## Windows 自动化续验（2026-10-31）

### 源码、工具与安全边界

`git fetch origin` 后本地 dev 和 origin/dev 均为 `ecd534ddcb527a59fe10a93989f66c231c3e5421`，初始无未提交改动。本轮增加 Windows 验证修正，未合并 main、建 tag 或发布。Windows x64 build 26200、原生192 DPI、WebView2 156.0.4314.50；Node24.19.0、pnpm11.25.0、Rust1.98.1 MSVC目标，沿用已有固定工具，不升级依赖。

MPV 来自此前核验的 WinGet 提取目录，执行前重新校验 mpv.exe SHA-256 `1645e57569c0af58a70c61257f3716f2ee6eb31bb170edc51f3f25a0b8d0d99c`，实际 v0.41.0-244-gaf9c81fa1。FFmpeg SHA-256 `7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1`。增强资产由本版清单逐项验证。每个原生用例创建新临时根，并在写入前通过 setup_status 确认配置根；WinGet仅为本地Rust夹具。未运行真实安装器、修改宿主配置、默认播放器或注册表登记。

### 命令与结果

本机先将 `.local/bin/node-v24.19.0`、`.local/bin` 和用户 `.cargo/bin` 加入当前进程PATH；通过 `.local/beta1-windows/env.ps1` 选择固定资产。下表日志均在 `.local/beta1-windows/`。

| 命令/入口 | 结果与范围 | 日志 |
| --- | --- | --- |
| pnpm install --frozen-lockfile --offline | 通过，锁文件未变 | install.log |
| pnpm build | TypeScript/Vite通过，约532kB chunk警告保留 | frontend-build.log |
| node --test scripts/frontend.test.mjs scripts/release.test.mjs | 19通过，未实际发布 | js-tests.log |
| node scripts/collect-app-notices.mjs --check | 331包、587材料，无缺项 | notices.log |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | 通过 | fmt-final.log |
| cargo test --manifest-path src-tauri/Cargo.toml --locked --offline | 51通过、5忽略 | rust-tests-native.log |
| cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings | 通过 | clippy.log |
| pnpm tauri build --debug --no-bundle --config .local/beta1-windows/build.conf.json | 通过；配置文件只清空已运行的beforeBuildCommand，避免PowerShell原生命令的JSON引号丢失 | native-build-final.log |
| pnpm test:e2e:smoke | 初次原生两页通过 | smoke.log |
| pnpm test:e2e | 初轮9项中8通过；配置用例旧选择器失败，随后修复并定向复验 | native-full.log |
| node --test --test-concurrency=1 --test-name-pattern='native v016 install, manual' scripts/native-v016.test.mjs | 修复后通过，含52项真实MPV音轨/字幕优先级检查、手改保护、重置、GUI关闭后的helper保存 | configuration-final.log |
| 最终交付目录EXE/自带资产：native single instance 与 native combined install | 两项通过；期间前台截图探针失败不计视觉通过，随后改回WebView截图单独复验 | test-app-final.log |
| 最终交付目录EXE/自带资产：pnpm test:e2e:smoke | 最终两页/浅深/草稿/日志状态/署名通过，等待偏好保存结束后取WebView截图 | test-app-smoke-final.log |
| node docs/v0.1.6/preview/build.mjs；node docs/v0.1.6/preview/check.mjs | Edge无头完整两页流程及双语/浅深/480/560/720/200%/高对比/动效矩阵通过 | preview-build.log、preview-final.log |

Rust最初在沙箱内为24通过、27失败、5忽略，失败涉及Windows替换文件、junction、日志轮转和回环绑定权限；获得受控执行授权后在新隔离根重跑，51项通过。原始失败日志保留为 rust-tests.log，不计产品失败或通过。最终debug构建保留增量缓存不能硬链接而回退复制、MSVC本地化linker stdout两项非致命警告，构建退出0；clippy检查退出0。

原生首次通过范围包括：更新启动/偏好/竞态/纯文本日志、mock下载/进度/失败/取消/重试；控制台stdin句柄、固定WinGet参数、实际退出码/取消/失败/假成功、核心与配置独立；五种媒体尺寸和缩放/切集保持；uosc真实布局/播放模式/stats；自然连播/字幕发现；真实播放快捷键/字体/仅位置续播；安装取消、健康组件不重装、过期预览、手改与续播完整删除、核心卸载失败仍清理及日志保留。更新mock不执行安装；官方updater实际下载/验签失败路径由Rust回环测试覆盖。

### 发现与修正

- Windows测试导入Linux包参数合约时，check/launch成为未使用函数；以target_os限定仅Linux实际调用的函数和Outcome导入，不忽略整个模块或丢失参数合约。
- 配置预览同时显示Before文本与Before折叠按钮。旧getByText严格匹配失败，改为定位具名region中的pre。GUI重开后高级区恢复折叠，重置前显式展开；完整配置用例复验通过。
- 离线布局对隐藏的高级设置行计算右边缘，零宽边界导致误报。改为只比较可见行并要求至少存在一行；全部布局矩阵通过，没有为通过测试修改产品CSS。
- 新增Windows单实例/首启尺寸/默认系统偏好/helper分派用例，以及原生窗口探针的minimized字段；最终交付EXE复验通过。

### 本机产物和证据

隔离入口 `.local/beta1-windows/app/run.cmd`，脚本将全部运行数据限制到旁边session/，复制的MPV、固定增强资产随目录提供。EXE大小25,256,448字节，SHA-256 `5fd253a46f9d126aa33bbc51935c1bb2ea252bf580a9721d4f39fe3aa0924e12`。版本0.2.0-beta.1，基于ecd534d及本轮平台条件编译修正；debug no-bundle，无Vite依赖。真实WinGet缺失时明确失败，不退回宿主包管理器；应用更新安装受测试模式阻止。单程序及增强部署许可材料保留，不是正式NSIS分发。

最终交付目录EXE/自带资产的证据：

- `.local/e2e-artifacts/mpvkit-beta1-instance-x7O5r5/result.json`：首启内容560×760、dpr2、偏好system/system、192 DPI、Mica属性backdrop2/HRESULT0；六次重复启动退出0，最小化后同句柄恢复且foreground=true，两个helper独立返回1，退出后可重开。
- `.local/e2e-artifacts/mpvkit-beta1-ui-fahG3U/`：两页浅深WebView内容截图；透明区不包含DWM背景/标题栏，不能作为完整原生视觉通过。
- `.local/e2e-artifacts/mpvkit-v016-zbrPtG/`：完整配置用例定向复验、result.json、配置diff及播放日志。
- `.local/e2e-artifacts/mpvkit-updates-7f8IJD/`、`mpvkit-v018-window-ol80gF/`、`mpvkit-v018-vw9lMC/`、`mpvkit-v018-autoplay-vesFkU/`：首轮通过的更新UI、窗口、uosc和连播/字幕记录；媒体及临时配置仍在日志所示私有临时根。

本轮原生屏幕截图探针未能始终取得前台，失败日志保留；没有把非本程序前台或透明WebView截图算作系统装饰视觉验收。实际标题栏、Mica观感、阴影、物理键盘和跨屏DPI交由用户；原生最小尺寸尚未以物理拖动验收。真实WinGet问答/UAC、NSIS跨版本升级重启、原生辅助技术、系统高对比/动画开关映射与磁盘满仍未实测。Rust中的无效存储/锁冲突不等于真实磁盘满通过。用户步骤见[交接](plan.md#交给用户的实际交互验收)。Linux原有未测边界继续保留。

## 公开发布核验（2026-10-31）

用户明确授权清理无用文档/文件、推送dev、合并main、dev同步main并发布beta.1，覆盖前文当时的不合并/不发布限制。

- 发布源码：`c7fda9289f0a0dfaf2b6293d10b9bce21882102a`；[PR #14](https://github.com/Leovikii/mpvkit/pull/14)合并提交及固定标签 `v0.2.0-beta.1`：`82d9be46c8cca0da235b0b56c7ba06b8da736b52`。
- [Checks](https://github.com/Leovikii/mpvkit/actions/runs/41195367594)成功：前端、Windows及Arch的fmt/clippy/test通过，Windows51通过/5忽略、Arch59通过/5忽略；Arch包与隔离Pacman检查通过；CI汇总成功后才合并。
- [Release](https://github.com/Leovikii/mpvkit/actions/runs/41195918873)成功：双端测试/正式构建、NSIS及Arch包签名、minisign验签、六项资产契约、公开上传和匿名manifest核验全部通过。正式Secrets由CI使用，未落本地或入库。
- [公开预发布](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-beta.1)：draft=false、prerelease=true，创建时make_latest=false；标签与发布源码一致，不移动已有tag。
- 本机未携带认证头并发下载全部六项公开资产，长度及GitHub资产digest逐项匹配；scripts/verify-release.mjs核对SHA256SUMS、签名字段、双平台URL、版本与日志通过。证据副本 `.local/beta1-release/public-verification.json`、`public-verification.log`，下载位于同目录public/。

| 资产 | 字节 | SHA-256 |
| --- | ---: | --- |
| latest.json | 1741 | `3dac06aa937333ab9145caf09700c4ee7019f6ac369ae446f21f60b9f6da6ede` |
| mpvkit-bin-0.2.0beta1-1-x86_64.pkg.tar.zst | 6976935 | `78a730ccd91663e15d76226e7d7ac0472e01f74566e11c029c0b431a989a73ee` |
| mpvkit-bin-0.2.0beta1-1-x86_64.pkg.tar.zst.tauri.sig | 436 | `94f1c82e2cc07ae9bad68bb3db2b9ee1a0f74e079d972ec5cc7caf4e706b326a` |
| MpvKit_0.2.0-beta.1_x64-setup.exe | 4134905 | `3e3c2f5d4ba2cea6ce3997932cb78d96edabc6dc4dd8ff057b67d480c67f4111` |
| MpvKit_0.2.0-beta.1_x64-setup.exe.sig | 452 | `13d1a0ccd7d24c57ff726b52c5178fc12ff0dde5e3e646c494d5478039f76d8d` |
| SHA256SUMS | 510 | `fe57a11e590aaf0ea85a0f7395eb05add7d6ef08a42ff4bf1a0535753420537e` |

Arch产物诊断已从release-Linux归档只读提取到 `.local/beta1-release/namcap.txt`、`elf-dependencies.txt` 和 `arch-system.txt`。namcap无E项：unstripped来自现有!strip原样打包；unused ld-linux为ELF解释器的分析提示；glibc、gdk-pixbuf2、dbus、cairo、hicolor-icon-theme、libsoup3、glib2、libgcc均被现有依赖链或系统基础包满足；xdg-utils、polkit和konsole通过运行时外部命令使用，不能以ELF未链接为由删除。隔离根无仓库数据库及同版重装警告是刻意不刷新数据库的载荷验收边界，不代表真实依赖首次拉取或Polkit通过。Actions运行时保留Buffer()弃用警告，不影响本次退出状态。

清理旧v0.1.5预览跳转文档及本轮三个一次性诊断脚本，合并重复Windows交接并更新README为beta.1流程；保留来源、许可、历史验收与本机固定工具、日志和测试版。所有仓库Markdown的本地文件链接及git diff检查通过。

实际WinGet问答/UAC、真实Polkit/Pacman、NSIS跨版本更新自动重启、完整原生视觉/物理多屏DPI及辅助技术仍未实测；发布成功不将这些边界改写为通过。发布后记录以文档补交合并，dev/main再次快进同步；发布tag继续固定在上述源码。

## MPV 启动回缩单项修复（2026-10-31）

本批源码基线 `4adc6b9`，用户授权修复、自行测试，随后授权整理文档并提交推送 dev；版本继续 `0.2.0-beta.1`，不合并 main 或发布。MPV v0.41.0、KWin 6.8.2、DP-1 3840×2160/150%，另有竖屏 DP-2。

此前直接对照已完成：Remember Window Positions 开启时回缩，临时卸载后连续两次仍回缩，恢复开启后仍回缩，插件已恢复。记录 `/tmp/mpvkit-kwin-ab-results.json`，副本 `.local/window-fix-test/evidence/kwin-ab.json`。原策略在约 0.65 秒首次落屏重算到 2880×1620，随后约 0.76 秒的 xdg configure 又带回 1080×608；不是停用窗口记忆即可解决。

本次把立即重算改为相关窗口属性稳定 0.5 秒后复核并仅重算一次。尺寸、输出和有效选项仍在等待期间观察；用户尺寸变化、显式几何/scale、全屏/最大化/最小化、跨屏和切集取消待执行定时器，结束后注销观察与 start-file 事件。当前 autofit 在执行时读取。使用 MPV v0.41.0 [官方 Lua API](https://github.com/mpv-player/mpv/blob/v0.41.0/DOCS/man/lua.rst) 的一次定时器、kill及注销接口；无依赖变更。

| 本次检查 | 结果 |
| --- | --- |
| 实际策略无窗口测试 | 通过；确定性时钟验证非立即执行、取整事件重启稳定期、回调复核、尺寸/跨屏/晚到选项/切集取消、只执行一次及音频先行 |
| Rust默认回归 | 59通过、5忽略；首轮57通过、两项回环端口因沙箱PermissionDenied失败，允许回环后的完整复验通过 |
| 静态检查与构建 | Rust fmt、clippy all-targets/-D warnings、debug本地构建通过；未改前端，不重复前端矩阵 |
| 完整已安装脚本临时副本 | 新源码替换隔离副本中的mpvkit.lua，KWin插件保持开启，5秒采样最终2880×1620，没有再次回缩；根 `/tmp/mpvkit-beta1-window-he5joh0w` |
| 原生扩展矩阵 | 首轮通过；新增竞态取消用例后的最终完整复验通过（31.50秒）。1080p/4K均2880×1620，竖屏911×1620、超宽2880×1215，自定义60%为2304×1296；音频切视频、显式几何/scale、最大化/全屏、手动尺寸后的切集保持及既有uosc/渲染矩阵通过 |
| 修正前实际操作 | 在窗口已出现、尚未修正时设置900宽或切集，等待1.2秒后尺寸保持，日志中无重算。测试明确检查干预发生在重算前 |
| 交付播放入口 | 实际运行 `.local/window-fix-test/play.sh`，首次重算后再观察3秒，日志最终2880×1620；结果 `.local/window-fix-test/result.json` |

最终扩展根 `/tmp/mpvkit-window-fix-extended-20261031-b/config/mpv`。原生测试不再在首屏400ms时采样：改为等待1.2秒，并对首次结果再连续检查1秒，尺寸与正确落屏的原生参考比较；每个测试使用唯一app_id，避免宿主记忆干扰。日志、layouts.json及无窗口策略检查的耐久副本在 `.local/window-fix-test/evidence/`。

隔离播放入口无参数播放本机合成视频，也可传媒体绝对路径；配置、缓存、状态及日志都在该测试目录。普通已安装的MPV配置未覆盖。Windows分支未修改且无窗口测试验证其跳过，本批未重跑Windows原生。Lua不提供configure队列完成信号，0.5秒属性稳定期是本机实测的有保护规避，不宣称覆盖任意慢合成器或所有物理拖动/混合DPI竞态；出现新设备复现时应优先使用可用的原生完成信号。未改KWin脚本或持久插件开关。
