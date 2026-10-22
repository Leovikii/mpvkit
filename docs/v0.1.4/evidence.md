# v0.1.4 本地验证

日期：2026-10-22。Windows x64，固定组件基线不变。所有写入测试使用全新临时目录，登记使用隔离文件后端及原生 Windows 私有蜂巢；不改变实际用户目录、注册表或默认应用。

## 验证范围

- Rust：文件归属/未知文件/路径边界、并发与过期预览、跨文件恢复、配置范围保护、真实 mtime、独立核心/配置状态、联合登记失败、登记重试、卸载失败保留入口、主配置保留、被修改 helper 拒绝登记。
- 原生登记：使用 Offreg 私有蜂巢核验 Windows API 的登记、外部修改保留、过期预览与中断恢复。没有测试实际 HKCU 默认应用。
- 前端：双语完整性、系统语种优先级、两/三标签切换、键盘导航、640×600、深浅主题、设置页、固定路径及已删除接口拒绝。
- 原生生命周期：核心安装不生成播放配置；配置预览取消与确认；默认集成、关闭与重试；手动修改保护；更新不重置；卸载保留配置/续播/未知文件；同格式重装；核心丢失后隐藏配置页并保留内存草稿。
- 真实 MPV：直接运行与受管理启动、GUI 关闭/源 EXE 移动后的稳定 helper 保存、原生画质 profile、最小键位/音量 ±10、菜单优先级、缩略图、字幕字体、PNG/JPEG 截图、仅位置续播和配置手动关闭续播。

## 命令与结果

| 检查 | 结果 |
| --- | --- |
| cargo test --locked | 56 通过，0 失败；6 个需显式环境的入口默认忽略，11.18 秒 |
| Windows 私有蜂巢入口 --ignored | 1 通过，0 失败；0.24 秒，原生登记 API 与恢复 |
| pnpm test:frontend | 2 通过，0 失败 |
| native-v014.test.mjs（最终 EXE） | 4 通过，0 失败/忽略；122.08 秒 |
| cargo fmt --check / clippy --all-targets -D warnings | 通过 |
| pnpm build（类型检查 + Vite） | 通过 |
| collect-app-notices.mjs --check | 321 个包、567 份材料，缺失项为空；不等于完整发布来源审阅 |
| 文档链接 / git diff --check | 22 份变更 Markdown 的本地链接有效，diff 无空白错误 |
| pnpm tauri build --debug --no-bundle | 单 EXE 成功，bundle.active=false；无安装包 |

默认忽略的真实归档/网络/媒体矩阵等入口没有全部重跑。本版四组原生测试已实际使用固定组件、真实 MPV 与合成媒体，但不冒充历史完整来源矩阵或实体高刷/HDR测试。

最终原生证据目录（本机，Git 忽略）：

- .local/e2e-artifacts/mpvkit-v014-3iyRa0：配置、差异、直启与 helper。
- .local/e2e-artifacts/mpvkit-v014-ui-U0D4l9：640×600、双语和主题。
- .local/e2e-artifacts/mpvkit-v014-lifecycle-5pIW9Y：卸载重装、集成意图、mtime 和草稿。
- 真实播放组保存合成截图、字体日志和结果 JSON 于 .local/e2e-artifacts/mpvkit-v014-player-*。

日志：.local/v014-unit.log、v014-private-hive.log、v014-native-final.log、v014-clippy.log、v014-build.log。首次回归发现并修正：无变化预览列表为空时测试等待可见性错误；设备在电池状态下推荐省电时，测试不应写死均衡档；Windows 刚退出的探针短暂锁定测试 EXE 时，仅针对共享冲突进行有界等待。产品事务还补齐无变化安装记录摘要和未变化注册项的并发核对，相关失败路径已有运行测试。

## 最终测试程序

- 路径：src-tauri/target/debug/mpvkit.exe
- 版本：0.1.4；构建时间：2026-10-22 18:07:30（Asia/Hong_Kong）。
- 大小：31,565,824 字节。
- SHA-256：CF96E63E7691B2523DE07E76F4DB3C036347AF8E9F9B17EC88C90CD39D3B1FF7。
- 最终四组原生测试使用此 EXE；没有随后修改产品源代码或重建产物。未提交或推送 GitHub。


## 范围限制

没有构建/执行安装包，没有 GitHub Actions/CI/CD，没有历史开发版本兼容测试。未将静态截图、GPU 型号或隔离回归当作实体高刷/HDR/功耗验证；干净系统、真实默认应用 UI、完整来源许可和 Linux 仍不在本轮完成范围。
