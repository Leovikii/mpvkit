# 高刷同步与简化倍速交互调查

日期：2026-10-22。范围：公开问题、固定源码、已锁定 uosc 配置核对；没有实体高刷对照测量，也未改用户播放器、驱动或屏幕模式。

## 结论

用户关心的是高刷新率下 display-resample 的额外开销和掉帧风险，而不是改变屏幕刷新率。现有依据支持保留这一风险，但不支持断言“当前 Windows 普遍仍有同一缺陷”或“问题已经彻底修复”。应分开看呈现频率、插值负载、驱动/交换链异常以及低帧率内容本身的运动顿挫。

MpvKit 固定构建默认 video-sync=audio、interpolation=no，当前没有全局启用示例中的显示同步策略。建议继续保留此兼容基线；没有性能证据前，不全局开启 display-resample，也不把社区 profile 的 32/3.2 阈值当作通用最优解。

## 原始问题与当前状态

用户示例直接引用 [mpv #11122](https://github.com/mpv-player/mpv/issues/11122)，2023 年报告 240–360 Hz 下 display 同步的掉帧/延迟，作者明确说未开启插值，开启插值会进一步增加 GPU 开销。因此不能把所有额外负载只归因于 interpolation。

问题于 2025-10-22 关闭，API state_reason=not_planned。[作者最后说明](https://github.com/mpv-player/mpv/issues/11122#issuecomment-3432773105)：收到 Windows 上不再发生的反馈，但自己已用 60 Hz 超过一年，无法复测；并再次说明此问题不是 VRR 讨论。关闭不是带复现对照的修复证明，亦不能将此处旧 VRR 评论作为今天的通用兼容结论。

[维护者解释](https://github.com/mpv-player/mpv/issues/11122#issuecomment-1378054365)：单独把报告的显示频率改成 120，实际 vsync 却仍以 240 返回，会破坏 display 同步的计时关系。不得以伪造 display-fps 作为降负载默认。后续讨论提出 sync interval 与频率覆盖配合，但依赖后端/驱动正确执行，不作为跨屏通用方案。

## 当前固定源码中的机制

核对项目固定提交 6a262d11f6：

- [player/video.c](https://github.com/mpv-player/mpv/blob/6a262d11f6/player/video.c) 按帧时长和 vsync 周期计算 num_vsyncs，形成 display_synced 帧。
- [video/out/vo.c](https://github.com/mpv-player/mpv/blob/6a262d11f6/video/out/vo.c) 在 num_vsyncs 尚未耗尽时继续重复呈现；每次递减并维护显示计时。因此即使没有插值，显示同步仍有按显示节奏工作的成本。
- [video/out/vo_gpu_next.c](https://github.com/mpv-player/mpv/blob/6a262d11f6/video/out/vo_gpu_next.c) 根据 will_redraw/cache_frame 启用单帧缓存，插值则有独立 can_interpolate 条件和混合缓存。高刷重复呈现不等于每次重新解码或完整运行全部缩放/着色器，不能简单按刷新率倍数推算 GPU/功耗增长。

60/144/240 Hz 的每次呈现时间预算约为 16.67/6.94/4.17 ms，但是否错过呈现期限还取决于渲染负载、缓存、驱动和调度。该机制分析不是目标硬件的掉帧复现。

## 相关修复及不能混为一谈的报告

[Windows 卡顿 #15196](https://github.com/mpv-player/mpv/issues/15196)中维护者指向 [PR #16817](https://github.com/mpv-player/mpv/pull/16817)，2025-09-22 合并为 ee0f7013478abad3f7fcd05a7f61d360b7447b18。修复将 display-* 下的有效 swapchain depth 统一为 1，并修正 D3D11 等后端的延迟/队列设置。GitHub compare 确认项目固定提交在该合并之后（ahead=1405、behind=0）。因此不能再把该修复前的行为无条件套用到当前构建。

但该 PR 不等于实现“以更低目标帧率运行 display-sync”，也没有取消每个 vsync 的呈现机制。#15196 后续仍有与驱动/窗口化 G-Sync 相关的用户反馈，不能据此证明同一问题复发。

[165 Hz 运动顿挫 #14620](https://github.com/mpv-player/mpv/issues/14620)的后续讨论包含屏幕响应、低源帧率运动观感；“看起来不流畅”不必然等于解码/呈现丢帧。不将不同后端、VRR、低帧率运动和性能瓶颈合并成一个高刷缺陷。

## 对 0.1.2 的建议与验证门槛

默认保留 audio/no interpolation；显示同步平滑保留为高级候选。若后续启用社区式自动策略，应作为经过测量的兼容取舍，不标为上游必需修复。先用同一媒体/画质/窗口模式比较三组，隔离变量：

1. audio + interpolation=no（基线）。
2. display-resample + interpolation=no（只看同步策略变化）。
3. display-resample + interpolation=yes（再看插值成本）。

在实际可用 60 Hz/高刷设备上重复，覆盖 23.976/24/30/60 fps 和有无字幕，预热后采集帧呈现耗时、vo-delayed-frame-count、frame-drop-count、decoder-frame-drop-count、音视频偏差与实际显示同步状态。呈现重复次数不能直接算作异常掉帧；不凭 GPU 占用率或截图宣称性能/能效收益。跨屏和倍速切换单列，不能混入稳定段数据。没有设备则明确留空，不用修改系统显示模式或伪造 FPS 代替。

## 倍速替代方案：复用 uosc

用户明确要求默认机制不能简单实现组合键时暂缓，不为“右键保持＋上下调倍率”添加 Windows 按键辅助、全局钩子或常驻服务。取消该组合操作的当前实现任务。

已核对固定 uosc 5.13.0 提交 d124c2c930d69446448022851373e00ae592390d：

- [src/uosc.conf](https://github.com/tomasklaen/uosc/blob/d124c2c930d69446448022851373e00ae592390d/src/uosc.conf) 默认 controls 已含 speed，speed_step=0.1、speed_step_is_factor=no。
- [Speed.lua](https://github.com/tomasklaen/uosc/blob/d124c2c930d69446448022851373e00ae592390d/src/uosc/elements/Speed.lua) 已实现速度控件内鼠标拖动/滚轮调整，secondary_click 恢复 speed=1。按当前鼠标映射通常为右键；不按配置注释中笼统的 click 擅称左键点击恢复。

推荐直接使用播放器控制栏中的倍率控件：鼠标移到控件才调整速度，不占用全局滚轮或新增易误触字母键。常用固定倍率菜单仅在实际需要时增加，不重复实现现有控件。首次使用的提示应明确“拖动/滚轮调速，右键恢复 1 倍”。本轮已做源码核对，尚未复验精简键位后控件实际鼠标事件；实现最小键集合时列为回归项。

单独长按右键临时 2 倍速可留为后续可选项；若做，需要 Lua 和松手/失焦清理，本轮不作为已确定交付，不恢复组合调速范围。基本方向键仍维持跳转与音量功能。
