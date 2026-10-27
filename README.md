# MpvKit

为 MPV 提供简洁的安装、配置与更新界面。支持 **Windows x64** 和 **Arch Linux / CachyOS x86_64（KDE Wayland）**。

当前版本为 **0.2.0-alpha.1**，仍处于预览阶段。

## 功能特色

- 一处管理 MPV 核心、uosc 控制栏、进度条缩略图和播放配置。
- 三档画质预设、HDR 选项、同目录自然排序连播、字幕搜索与位置续播。
- 保存前预览变更，保护手动修改；移除增强时保留系统 MPV 和用户内容。
- 简体中文 / English，支持系统、浅色与深色主题。

预设的取舍与默认行为见[预设配置设计](docs/presets.md)。

## 安装

从 [Releases](https://github.com/Leovikii/mpvkit/releases) 获取安装包。当前尚无公开安装包，以下步骤适用于首个预览版发布后；预览版请认准 **Pre-release** 标记。

**Windows**：下载并运行 `MpvKit_<版本>_x64-setup.exe`，自动安装到当前用户目录。打开 MpvKit 后安装 MPV 核心与播放增强，再预览并保存配置；核心由 WinGet 管理，需要可用的 Windows 应用安装程序。

**Arch / CachyOS**：下载 `.pkg.tar.zst` 后安装：

```sh
sudo pacman -U ./mpvkit-bin-0.2.0alpha1-1-x86_64.pkg.tar.zst
```

从应用菜单打开 MpvKit。MPV 核心可在应用内通过系统包管理器安装，授权需要 polkit。AUR 配方尚未公开；发布后可通过 AUR 助手更新，应用内会提示可用更新。

保存后，用系统 MPV 正常打开媒体即可。MpvKit 不更改默认播放器。

## 许可证

[GPL-3.0-or-later](COPYING.md)。第三方组件、字体与依赖保留各自许可证及署名。
