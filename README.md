# MpvKit

A simple interface for installing, configuring and updating MPV on **Windows x64** and **Arch Linux / CachyOS x86_64 (KDE Wayland)**.

Current version: **0.2.0-alpha.2**, an early preview.

## Features

- Manage the MPV core, uosc controls, seek previews and playback settings in one place.
- Three playback presets, HDR options, naturally sorted folder playlists, subtitle discovery and playback resume.
- Preview configuration changes and protect manual edits. Removing enhancements preserves system MPV and user content.
- Simplified Chinese and English, with system, light and dark themes.

See [Preset Design](docs/presets.md) for defaults and configuration choices.

## Installation

Download packages from [Releases](https://github.com/Leovikii/mpvkit/releases). Preview builds are marked **Pre-release**.

**Windows:** Download `MpvKit_<version>_x64-setup.exe` from [Releases](https://github.com/Leovikii/mpvkit/releases) and run it to install for the current user. Open MpvKit, install the MPV core and enhancements as needed, then preview and save your settings. Core management uses WinGet and requires Windows App Installer.

**Arch Linux / CachyOS:** Download `mpvkit-bin-<version>-<release>-x86_64.pkg.tar.zst` from Releases and install it with Pacman. For the current preview:

```sh
sudo pacman -U ./mpvkit-bin-0.2.0alpha2-1-x86_64.pkg.tar.zst
```

Launch MpvKit from the application menu. Install the MPV core through the app if needed; package operations require polkit authorization. To update MpvKit, download the new package and run `pacman -U` again. Regular `pacman -Syu` does not update this locally installed package. AUR publication is pending.

After saving your settings, open media with system MPV as usual. MpvKit does not change your default player.

## License

[GPL-3.0-or-later](COPYING.md). Third-party components, fonts and dependencies retain their own licenses and notices.
