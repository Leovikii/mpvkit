# Windows 安装器

当前 rc.1 使用用户级 NSIS。双击直接显示安装进度，固定安装到 %LOCALAPPDATA%/Programs/MpvKit；成功后显示原生完成页，默认勾选“启动 MpvKit / Launch MpvKit”，点击“完成 / Finish”才按选择启动一次。取消勾选不启动，启动请求失败可见反馈。无欢迎、许可确认、安装目录或开始菜单向导，不提供路径修改，忽略 /D 和旧自定义路径。语言跟随系统，非中文/英文回退中文；许可随包提供。

本轮已实现[确认方案](../../docs/v0.2.0-rc.1/plan.md#windows-安装后启动)。普通安装保留直接进度流；/UPDATE 或显式 /P 不显示完成页，官方 updater 的 /P /R /ARGS 仍只经既有路径自动重启。/S 保持静默，显式 /R 保留上游语义。失败/取消/卸载不因本需求启动程序，不新增登录自启或媒体登记。安全页面探针与未测系统边界见[证据](../../docs/v0.2.0-rc.1/evidence.md)。

重装/升级直接覆盖程序及自有资源，保留应用偏好与 MPV 配置；保留上游文件占用检查、WebView2 前提检查、错误提示、卸载和 updater `/UPDATE /P /R` 行为。正常启动不使用静默模式；失败时显示具体错误，不能为了免点击而忽略失败。构建禁止降级。安全软件/SmartScreen、缺少系统依赖和安装失败可能仍需系统交互，不承诺绕过系统提示。

`installer.nsi` 来自锁定的 [Tauri CLI 2.12.3 官方模板](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.3/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)，按 MIT 使用，见 [许可原文](LICENSE.tauri.txt)。上游原始字节 SHA-256：`8ae02c66f8adb4a0d15d97f984fbacd2b44262d8a4d03fefaf6177f67f7008e5`。许可原文取自锁定的 `@tauri-apps/cli@2.12.3/LICENSE-MIT`。

修改默认 $PassiveMode=1、固定安装位置、passive 重装直接继续，以及独立完成页条件、短本地化文字、RunAsUser 返回值检查；移除完成页桌面快捷方式复选框，保留现有快捷方式创建。其余保留上游模板，由 Tauri 渲染资源/版本/卸载列表。官方 installerHooks 仅支持复制文件前后的钩子，无法在向导显示前设置这些选项，因此使用其正式支持的 `template` 配置。升级 CLI 时须重新对比模板，不能直接替换或沿用生成后的绝对路径脚本。

开发验证可运行 node scripts/test-windows-installer.mjs；它从模板提取真实页面/收尾函数，编译只写本机 .local 的夹具，不运行产品安装、写注册表或真实配置。它不代替正式首装/重装/升级、运行中程序处理和真实 updater 验收。

先 `pnpm build`，再按 [构建说明](../../docs/ci/README.md)注入匹配的签名密钥执行 `node scripts/build-windows.mjs`。构建不执行安装器；真实安装/卸载和应用更新验收须在隔离 Windows 用户或 VM 进行。
