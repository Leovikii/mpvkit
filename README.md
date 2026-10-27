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

Public packages are not available yet; the steps below apply once the first preview is published. Preview builds are marked **Pre-release**.

**Windows:** Download `MpvKit_<version>_x64-setup.exe` from [Releases](https://github.com/Leovikii/mpvkit/releases) and run it to install for the current user. Open MpvKit, install the MPV core and enhancements as needed, then preview and save your settings. Core management uses WinGet and requires Windows App Installer.

**Arch Linux / CachyOS (recommended):** Install the AUR package with your preferred helper:

```sh
yay -S mpvkit-bin
```

Or use `paru -S mpvkit-bin`. The package is not on AUR yet; these commands become available after it is published.

Launch MpvKit from the application menu. Install the MPV core through the app if needed; package operations require polkit authorization. Use your AUR helper for future updates; the app will notify you when an update is available.

After saving your settings, open media with system MPV as usual. MpvKit does not change your default player.

## License

[GPL-3.0-or-later](COPYING.md). Third-party components, fonts and dependencies retain their own licenses and notices.
