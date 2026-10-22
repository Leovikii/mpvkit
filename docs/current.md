# 当前开发基线

更新：2026-10-22。源码 v0.1.5，Windows x64。实施与本地验证状态见 [v0.1.5 交接](v0.1.5/plan.md)。

[v0.1.5 实施](v0.1.5/plan.md) 已获授权，执行 [界面开发规范](design/ui-guidelines.md)。以下为当前产品行为。

## 持续约束

先读 [AGENTS.md](../AGENTS.md)。正式发布前无 GitHub Actions/CI/CD、无历史开发版兼容或迁移、无安装包；仅本地检查与单 EXE。未知实际数据保留并报错，不搜索或清理用户旧环境。Linux 尚未实现，不宣称可跨平台编译。

## 当前用户流程

- 左侧三页：播放器、播放设置、应用设置。设置固定底部，窄窗显示图标和提示；无引导、编号步骤或设置弹窗。
- 未安装或损坏核心时隐藏播放设置入口；当前草稿留在内存，切回管理并提示。设置始终可访问，切标签不提交草稿或取消下载。
- 管理页显示组件版本/健康、配置和登记状态。更新检查只对项目固定支持资产作比较，不声称是上游最新。
- 核心和固定播放增强一次安装；不生成 mpv.conf/input.conf。安装后留在管理页，由“生成配置”进入播放设置页选择档位、预览和安装。
- 系统打开方式默认启用，保存独立意图。核心和配置就绪后才登记；首次配置确认同时显示文件与具体登记变更，不修改 UserChoice。
- 配置保存后登记失败会保留配置，允许单独重试或关闭登记；部分登记需先恢复。重启只检查，不自动执行系统写入。
- 路径固定为 Known Folder LocalAppData/MpvKit/player，只显示和打开。无自定义目录、外部 MPV、PATH 扫描、目标切换或相关 IPC。
- 主配置修改时间取当前 mpv.conf 文件 mtime，按界面语言与本地时区显示。进入管理页、操作完成、窗口聚焦和手动刷新时重查，不另存时间戳、不启用监听服务。
- 高级维护可卸载播放器：先安全移除系统登记，再删除归属且未改变的组件；保留配置、续播、未知用户文件及必要归属记录。同一当前格式重装不重置配置。

## UI 与配置规范

Mantine core/hooks 固定 9.7.3，遵守[前端规范](v0.1.0/frontend.md)和[官方快照](vendor/mantine/README.md)。采用可键盘操作的 AppShell/NavLink 侧栏、内容区滚动；页面/折叠/弹窗使用 Mantine 动效，尊重减少动态效果；自定义选项收纳高级区。长许可在设置页展开。语言、主题和强调色是应用偏好，与播放语种独立。

图标同源同色：用户批准的 MPV SVG、#A84BD6、无边框。Mica 根据 Windows 能力和策略自动使用，其他情况同主题纯色；无材质开关。首帧显示前初始化，原生主题/焦点事件及策略复查保持有效。

配置沿用原生 portable_config：直接编辑 mpv.conf，键位 input.conf，脚本参数 script-opts；无 user.conf。GUI 保存仅改明确管理的范围，手改冲突需差异预览和备份，标记损坏不猜测合并。恢复推荐配置明确重建主配置。音量 ±10、位置续播、文楷字幕字体和最小快捷键保持现状。

无播放硬件检测、分辨率输入、显示器选择或供电自动推荐；默认均衡、档位旁给简短建议。HDR 默认 Auto，高级提供强制开启/关闭；MPV 自行适配，不修改系统开关。音轨/字幕默认汉→日→英→西，写在非 GUI 管理区；GUI 无语种控件或解释文字，后续保存保留手改/删除。GUI 展示上次保存的结构化设置，不能完整反映任意手写配置。

## 当前目录

单 EXE 由用户放置；不规定管理器安装目录，不生成安装包。

~~~text
%LOCALAPPDATA%/MpvKit/
├── mpvkit.json                     应用语言、主题、强调色；无 target 字段
├── .mpvkit/
│   ├── owner.json                  软件目录归属
│   └── operation.lock             软件偏好跨进程写入锁
└── player/                         唯一播放器根
    ├── mpv.exe
    ├── d3dcompiler_43.dll          固定 MPV 构建所附运行依赖
    ├── portable_config/
    │   ├── mpv.conf                用户明确安装配置后生成
    │   ├── input.conf              用户明确安装配置后生成
    │   ├── script-opts/thumbfast.conf
    │   ├── fonts.conf              核心归档附带
    │   ├── fonts/                  文楷与 uosc 字体
    │   ├── scripts/                mpvkit.lua、thumbfast.lua、uosc/
    │   └── watch_later/            MPV 按需生成，仅保存位置
    ├── licenses/                   固定上游许可和来源
    └── .mpvkit/
        ├── settings.json          GUI 设置、修订、手改标记、集成意图
        ├── owner.json             播放器目录归属
        ├── install.json           文件摘要、组件版本、安装代际
        ├── operation.lock         部署/启动/登记短时互斥
        ├── associations.json      登记及原值；登记后生成
        ├── association-pending.json 登记事务进行中或中断时存在
        ├── pending.json           文件事务进行中或中断时存在
        ├── previous.json          最近一次可恢复文件事务
        ├── backups/               恢复需要的内容
        ├── staging/               暂存
        ├── receipts/              备份/暂存归属
        ├── cache/sessions/         受管理启动的有界日志
        └── launcher/              稳定 helper 及应用许可
~~~

不强制创建空目录。截图默认 Pictures/MpvKit，实际截图时才创建；thumbfast 临时图像按上游实现存于系统临时目录。Debug 隔离根需显式设置 MPVKIT_TEST_MODE=1 与合法绝对 MPVKIT_TEST_ROOT，登记使用私有文件后端；Release 忽略测试覆写。测试记录不属于正式数据结构。

## 模块与保护

commands 只派生固定根、存放预览并调用业务；setup 汇总状态和组织具体文件/登记顺序；config 生成和保护配置；components 获取标准资产；deploy 负责原有归属、摘要、锁、备份和恢复。integration 对外提供操作入口，Windows 注册数据、登记事务与恢复位于 platform/integration.rs。无 Linux 空适配器或通用工作流框架。

全局偏好不再依赖目标选择锁，直接使用全局目录的部署互斥。受管理 helper 校验固定根；原程序移动或 GUI 关闭不影响稳定入口。文件/注册表分别有事务，不宣称跨资源原子性；所有相关状态在执行前校验。无文件变更时使用实际保留的安装记录摘要，登记预览同时保护已有且无需改写的注册值。

## 验证边界

本版证据在 [v0.1.4 计划](v0.1.4/plan.md) 更新；历史结果不等于本版验收。实体高刷/HDR/功耗、不同 DPI/系统策略、干净系统及完整来源仍有缺口，参见[验收台账](v0.1.0/acceptance.md)。[来源政策](v0.1.0/sources.md)、第三方许可证和历史证据继续保留。不得执行历史安装器测试。
