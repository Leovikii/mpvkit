# MpvKit

A simple interface for installing, configuring and updating MPV on **Windows x64** and **Arch Linux / CachyOS x86_64 (KDE Wayland)**.

Current version: **0.2.0-alpha.1**, an early preview.

## Features

- Manage the MPV core, uosc controls, seek previews and playback settings in one place.
- Three playback presets, HDR options, naturally sorted folder playlists, subtitle discovery and playback resume.
- Preview configuration changes and protect manual edits. Removing enhancements preserves system MPV and user content.
- Simplified Chinese and English, with system, light and dark themes.

See [Preset Design](docs/presets.md) for defaults and configuration choices.

## Installation

Download packages from [Releases](https://github.com/Leovikii/mpvkit/releases). No public packages are available yet; the steps below apply once the first preview is published. Preview builds are marked **Pre-release**.

**Windows:** Run `MpvKit_<version>_x64-setup.exe` to install automatically for the current user. Open MpvKit, install the MPV core and enhancements as needed, then preview and save your settings. Core management uses WinGet and requires Windows App Installer.

**Arch / CachyOS:** Download the `.pkg.tar.zst` package and install it:

```sh
sudo pacman -U ./mpvkit-bin-0.2.0alpha1-1-x86_64.pkg.tar.zst
```

Launch MpvKit from the application menu. Install the MPV core through the app if needed; package operations require polkit authorization. The AUR recipe is not public yet. Once available, use your AUR helper to update MpvKit; the app will notify you of updates.

After saving your settings, open media with system MPV as usual. MpvKit does not change your default player.

## License

[GPL-3.0-or-later](COPYING.md). Third-party components, fonts and dependencies retain their own licenses and notices.
