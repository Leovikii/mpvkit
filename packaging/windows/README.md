# Windows 安装器

当前 0.2.0-alpha.4 使用用户级 NSIS。双击后直接显示安装进度，自动安装到 `%LOCALAPPDATA%\Programs\MpvKit`，成功后自动关闭；不显示欢迎、许可确认、安装目录、开始菜单或完成向导，不提供路径修改，忽略 `/D` 和此前记录的自定义安装路径。界面语言跟随系统，非中文/英文系统回退中文。许可随包提供，应用内仍可查看。

重装/升级直接覆盖程序及自有资源，保留应用偏好与 MPV 配置；保留上游文件占用检查、WebView2 前提检查、错误提示、卸载和 updater `/UPDATE /P /R` 行为。正常启动不使用静默模式；失败时显示具体错误，不能为了免点击而忽略失败。构建禁止降级。安全软件/SmartScreen、缺少系统依赖和安装失败可能仍需系统交互，不承诺绕过系统提示。

`installer.nsi` 来自锁定的 [Tauri CLI 2.12.3 官方模板](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.3/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)，按 MIT 使用，见 [许可原文](LICENSE.tauri.txt)。上游原始字节 SHA-256：`8ae02c66f8adb4a0d15d97f984fbacd2b44262d8a4d03fefaf6177f67f7008e5`。许可原文取自锁定的 `@tauri-apps/cli@2.12.3/LICENSE-MIT`。

仅修改默认 `$PassiveMode=1`、固定安装位置，以及 passive 重装时直接继续；其余保留上游模板，由 Tauri 渲染资源/版本/卸载列表。官方 installerHooks 仅支持复制文件前后的钩子，无法在向导显示前设置这些选项，因此使用其正式支持的 `template` 配置。升级 CLI 时须重新对比模板，不能直接替换或沿用生成后的绝对路径脚本。

先 `pnpm build`，再按 [构建说明](../../docs/ci/README.md)注入匹配的签名密钥执行 `node scripts/build-windows.mjs`。构建不执行安装器；真实安装/卸载和应用更新验收须在隔离 Windows 用户或 VM 进行。
