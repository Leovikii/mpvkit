# 当前开发基线：v0.1.6

实施与验证见 [v0.1.6 计划](v0.1.6/plan.md)。未正式发布；继续遵守无 CI/CD、无历史开发版兼容、只构建单 EXE 的约束。

## 当前界面与操作

侧栏只有“播放器设置”和“应用设置”。首次即可编辑画质/HDR 与高级偏好；草稿切页或安装失败时保留，关闭时提示放弃。组件可单独安装，也可在配置预览后安装组件，再确认配置与登记。只有所有必需组件健康才可应用配置，保存前复核文件摘要并保留手改、备份和恢复保护。

底部固定“预览配置”和“保存配置 / 安装并保存配置 / 修复并保存配置”。未变化且配置健康时禁用保存；预览始终可用。主状态显示 mpv.conf 的真实修改时间，预览显示对应文件自己的 mtime，到秒；未变更文件不重写。

高级选项保留系统打开方式意图、播放结束与截图偏好、组件明细和维护。安装/配置目录使用同一个设置行：左侧“打开安装目录 / 打开配置目录”，副标题实际路径，右侧“打开”；无自定义路径。应用设置包含双语、主题、强调色、关于和诊断。遵守 [界面规范](design/ui-guidelines.md)。

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

本版证据在 [v0.1.6 记录](v0.1.6/evidence.md) 更新；历史结果不等于本版验收。实体高刷/HDR/功耗、不同 DPI/系统策略、干净系统及完整来源仍有缺口，参见[验收台账](v0.1.0/acceptance.md)。[来源政策](v0.1.0/sources.md)、第三方许可证和历史证据继续保留。不得执行历史安装器测试。
