# 历史目录说明：v0.1.3

v0.1.4 已删除目标选择、自定义路径、外部播放器和引导状态。当前完整结构见 [当前基线](../current.md)，本文件仅保留历史证据，不能作为当前实现要求。

# v0.1.3 实际配置与目录

Windows x64，新建受管理环境。固定四组件：MPV、uosc、thumbfast、文楷字体；本机真实部署清单含 79 个归属文件。安装过程和用户使用还会生成下列条件记录，不能仅用固定文件数描述全部磁盘内容。

## 用户需要知道的两类配置

- **MPV**：在播放设置点击“打开配置目录”，进入 portable_config。播放参数直接改 mpv.conf；快捷键改 input.conf；脚本选项按 MPV 官方语法放 script-opts。无需创建 user.conf。
- **MpvKit**：在应用设置调整语言、主题与强调色；全局 mpvkit.json 保存偏好和所选环境。播放器目录内的 .mpvkit 属于软件内部状态，不需要手改。

~~~text
%LOCALAPPDATA%/Programs/MpvKit/       管理器程序；新安装默认位置
%LOCALAPPDATA%/MpvKit/
├── mpvkit.json                     全局应用偏好与所选环境
├── .mpvkit/
│   ├── owner.json                  全局数据目录归属
│   ├── operation.lock             全局目录归属写入保护
│   └── selection.lock                 目标与应用偏好共用锁
└── player/                         默认播放器目录，可在高级选项自定义
    ├── mpv.exe
    ├── d3dcompiler_43.dll          固定 MPV 构建所附运行依赖
    ├── portable_config/
    │   ├── mpv.conf                默认参数、GUI 管理范围与原生 profiles
    │   ├── input.conf              最小键位
    │   ├── fonts.conf              固定构建附带的字体配置
    │   ├── fonts/                  文楷、uosc 字体
    │   ├── scripts/                mpvkit.lua、thumbfast.lua、uosc/
    │   ├── script-opts/            thumbfast.conf
    │   └── watch_later/            MPV 按需创建，仅保存位置
    ├── licenses/                   上游许可和来源材料
    └── .mpvkit/
        ├── settings.json          GUI 设置、修订、引导确认、手改状态
        ├── owner.json             环境归属
        ├── install.json           文件摘要、组件版本、环境及安装代际
        ├── associations.json      系统登记及原值（登记后）
        ├── operation.lock         环境互斥
        ├── pending.json           进行中或中断的文件事务
        ├── association-pending.json 进行中或中断的登记事务
        ├── previous.json          最近一次可恢复文件事务
        ├── backups/               恢复需要的内容
        ├── staging/               部署暂存
        ├── receipts/              暂存与备份归属
        ├── cache/sessions/        受管理启动后的有界会话日志与记录
        └── launcher/              稳定助手及应用许可材料
~~~

安装状态/事务记录按需存在；不强制生成每个空目录。截图默认在系统 Pictures/MpvKit，MPV 实际截图时才创建。下载校验暂存由部署/组件流程管理，临时 thumbfast 图像按上游实现位于系统临时目录，正常退出清理；不属于长期配置。全局与环境各有锁，因为切换目标和播放器部署不是同一生命周期。

## 当前格式与手动修改

尚无正式版，只支持本文列出的当前布局和设置格式。不提供开发版升级、旧文件导入或兼容分支；不主动删除已有旧文件。测试使用隔离的新环境。

外部 MPV 模式只在选择的 MpvKit 环境下保存 portable_config 和管理状态，程序及依赖保持外部位置；启动明确传配置目录。直接双击外部 MPV 不保证读取这一配置。

GUI 管理范围外的文本在保存时保留；管理范围内手改需明确替换。恢复推荐主配置会重建整个 mpv.conf，先展示完整差异并备份；不会重置 input.conf。恢复入口以最近一次文件事务为单位（含引导确认、设置、组件），不是任意版本历史。

## 固定部署文件完整清单

下列路径相对于播放器目录；系统字体、动态缓存、事务记录不在这份归属文件清单中。

~~~text
.mpvkit/launcher/LICENSE.txt
.mpvkit/launcher/PROJECT-LICENSE.md
.mpvkit/launcher/mpvkit.exe
.mpvkit/launcher/third-party/APP-DEPENDENCY-NOTICES.txt
.mpvkit/launcher/third-party/RUST-COPYRIGHT-library.html
.mpvkit/launcher/third-party/WEBVIEW2-SDK-LICENSE.txt
.mpvkit/launcher/third-party/WEBVIEW2-SDK-NOTICE.txt
.mpvkit/launcher/third-party/app-dependency-inventory.json
.mpvkit/launcher/third-party/app-distribution-materials.json
.mpvkit/settings.json
d3dcompiler_43.dll
licenses/LXGWWenKaiScreen-OFL.txt
licenses/THIRD-PARTY.txt
licenses/catalog.json
licenses/ffmpeg-GPL-3.txt
licenses/material-icons-Apache-2.txt
licenses/mpv-Copyright.txt
licenses/mpv-GPL-2.txt
licenses/mpv-LGPL-2.1.txt
licenses/source-materials.json
licenses/thumbfast-MPL-2.txt
licenses/uosc-LGPL-2.1.txt
licenses/uosc-browser-BSD-2.txt
licenses/uosc-clipboard-BSD-3.txt
licenses/uosc-go-BSD-3.txt
licenses/uosc-go-NOTICES.txt
licenses/uosc-go-PATENTS.txt
licenses/uosc-sys-BSD-3.txt
mpv.exe
portable_config/fonts.conf
portable_config/fonts/LXGWWenKaiScreen.ttf
portable_config/fonts/uosc_icons.otf
portable_config/fonts/uosc_textures.ttf
portable_config/input.conf
portable_config/mpv.conf
portable_config/script-opts/thumbfast.conf
portable_config/scripts/mpvkit.lua
portable_config/scripts/thumbfast.lua
portable_config/scripts/uosc/bin/ziggy-windows.exe
portable_config/scripts/uosc/char-conv/zh.json
portable_config/scripts/uosc/elements/BufferingIndicator.lua
portable_config/scripts/uosc/elements/Button.lua
portable_config/scripts/uosc/elements/Controls.lua
portable_config/scripts/uosc/elements/Curtain.lua
portable_config/scripts/uosc/elements/CycleButton.lua
portable_config/scripts/uosc/elements/Element.lua
portable_config/scripts/uosc/elements/Elements.lua
portable_config/scripts/uosc/elements/ManagedButton.lua
portable_config/scripts/uosc/elements/Menu.lua
portable_config/scripts/uosc/elements/PauseIndicator.lua
portable_config/scripts/uosc/elements/Speed.lua
portable_config/scripts/uosc/elements/Timeline.lua
portable_config/scripts/uosc/elements/TopBar.lua
portable_config/scripts/uosc/elements/Updater.lua
portable_config/scripts/uosc/elements/Volume.lua
portable_config/scripts/uosc/elements/WindowBorder.lua
portable_config/scripts/uosc/intl/de.json
portable_config/scripts/uosc/intl/es.json
portable_config/scripts/uosc/intl/fr.json
portable_config/scripts/uosc/intl/it.json
portable_config/scripts/uosc/intl/pl.json
portable_config/scripts/uosc/intl/pt.json
portable_config/scripts/uosc/intl/ro.json
portable_config/scripts/uosc/intl/ru.json
portable_config/scripts/uosc/intl/tr.json
portable_config/scripts/uosc/intl/uk.json
portable_config/scripts/uosc/intl/zh-HK.json
portable_config/scripts/uosc/intl/zh-hans.json
portable_config/scripts/uosc/lib/ass.lua
portable_config/scripts/uosc/lib/buttons.lua
portable_config/scripts/uosc/lib/char_conv.lua
portable_config/scripts/uosc/lib/cursor.lua
portable_config/scripts/uosc/lib/fzy.lua
portable_config/scripts/uosc/lib/intl.lua
portable_config/scripts/uosc/lib/menus.lua
portable_config/scripts/uosc/lib/std.lua
portable_config/scripts/uosc/lib/text.lua
portable_config/scripts/uosc/lib/utils.lua
portable_config/scripts/uosc/main.lua
~~~

参考：[实施范围](plan.md)、[验证范围](evidence/2026-10-22-no-compat.md)。
