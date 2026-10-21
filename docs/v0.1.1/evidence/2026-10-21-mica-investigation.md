# 启动标题栏 Mica 调查

用户报告：启动时原生标题栏呈黑色，与深灰内容壳层不一致。此次只调查，未修改产品代码或重建发布产物。

## 已确认

1. 材质初始化由 React boot 调用，窗口默认已经可见；没有原生 setup 阶段的首帧外观准备。
2. commands::apply_appearance 用 material_allowed && window.is_focused() 作硬门槛。无焦点时调用 set_effects(None)，其实现会将 DWMWA_SYSTEMBACKDROP_TYPE 设置成 DWMSBT_NONE(1)，并非保留 Mica 让 DWM 自行呈现失焦状态。
3. 重新评估使用网页 window focus/blur 事件和 15 秒轮询，而决定条件却是原生窗口焦点；两者不是同一种状态。不能保证原生激活时及时恢复，也不能仅凭网页 document.hasFocus 判定顶层窗口已激活。
4. 已锁定 tauri 2.12.4 的 Window::set_effects 返回主线程调度结果，闭包丢弃 set_window_effects 返回值；Windows vibrancy::apply_effects 也丢弃 window-vibrancy 的实际结果。当前 is_ok() 不等于 DWM 属性成功，更不等于视觉材质生效。
5. 纯色回退调用 set_background_color 只处理原生客户区/WebView 背景，未统一非客户区标题栏颜色；Tao 的 dark_mode 只设置 immersive dark mode，因此黑色系统标题栏与 #202020 应用壳层可能明显分离。前端 CSS 无法修复原生标题栏。

## 运行证据及边界

使用经过 defaultRoot 隔离校验的现有调试程序、独立临时目录复现。Windows build 26200、DPI 192；environment.materialAllowed=true、highContrast=false。apply_appearance=false，DwmGetWindowAttribute(38) 成功返回 backdrop=1；WebView document.hasFocus=true，而 GetForegroundWindow 判断顶层窗口并未前台激活，壳层为 rgb(32,32,32)。尝试激活仍未取得前台，故本轮不能声称活动态 Mica 或截图同一瞬间的状态已复现。证据：.local/v011-tests/mica-investigation.json。

用户原来打开的 Release 进程在查询窗口属性前已关闭，没有修改或重启它；用户截图本身不能区分启动瞬间、失焦回退或底层 DWM 调用失败。上述代码缺陷及关闭材质路径已确认，但不能把测试环境的无前台限制直接认定为用户截图的唯一原因。

依赖实现核对版本：tauri 2.12.4、tauri-runtime-wry 2.12.2、tao 0.37.1、window-vibrancy 0.8.1。apply_mica 在当前 Windows 路径设置 DWMWA_SYSTEMBACKDROP_TYPE=DWMSBT_MAINWINDOW(2)，不设置自定义 caption color。

## 建议修复路径

- 将原生外观初始化提前到窗口首次显示之前。系统支持时保持 Mica 注册，不以瞬时网页/输入焦点反复卸载材质；采用原生窗口激活/主题事件同步必要状态。
- 平台模块直接取得并检查 DWM HRESULT，设置后读取 backdrop；区分“配置已接受”与“当前材质视觉生效”，保留失败原因。
- 支持 Mica 时使用系统标题栏颜色；纯色回退明确统一客户区和非客户区背景，保留高对比度系统策略，不重写窗口按钮。
- 在隔离窗口验证冷启动、原生激活/失焦、浅深主题、透明关闭/恢复和回退；活动态标题栏需可交互桌面目视验收。不要只通过取消透明属性、加 CSS 模糊或延长定时器掩盖问题。

本轮未写真实偏好、MPV 配置、注册表或系统透明开关；无新依赖、未改源代码。下一任务为按上述路径修复及原生回归。
