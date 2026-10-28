# MpvKit

A simple interface for installing, configuring and updating MPV on **Windows x64** and **Arch Linux / CachyOS x86_64 (KDE Wayland)**.

Source version: **0.2.0-alpha.5** (release preparation). Latest public preview: **0.2.0-alpha.4**.

## Features

- Install missing player components together or manage them individually.
- Configure playback on a separate page after the core and required components are ready.
- Three playback presets, HDR options, naturally sorted folder playlists, subtitle discovery and playback resume.
- Preview configuration changes and protect manual edits. Removing enhancements preserves system MPV and user content.
- Simplified Chinese and English, with system, light and dark themes.

See [Preset Design](docs/presets.md) for defaults and configuration choices.

## Installation

Download packages from [Releases](https://github.com/Leovikii/mpvkit/releases). Preview builds are marked **Pre-release**.

**Windows:** Download `MpvKit_<version>_x64-setup.exe` from [Releases](https://github.com/Leovikii/mpvkit/releases) and run it to install for the current user. Open **Player management** and use **Quick install**, then open **Configuration** to preview and install your settings. Core management uses WinGet and requires Windows App Installer.

**Arch Linux / CachyOS:** Download `mpvkit-bin-<version>-<release>-x86_64.pkg.tar.zst` from Releases and install it with Pacman. For the current preview:

```sh
sudo pacman -U ./mpvkit-bin-0.2.0alpha4-1-x86_64.pkg.tar.zst
```

Launch MpvKit from the application menu. Install the MPV core through the app if needed; package operations require polkit authorization. For future updates, open **App settings → About → Check for updates**. MpvKit downloads and verifies the package, shows the installation preview, then requests system authorization for Pacman. Regular `pacman -Syu` does not update this locally installed package.

Both platforms keep download progress, installation and results in one dialog, with one confirmation before installation. Preview updates require **Include preview releases**. Linux updates require a system installation managed by Pacman; close and reopen MpvKit after installation.

For a single download, checksum and installation command, see the [release notes](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-alpha.4). Packages use Tauri update signatures; these are not Pacman OpenPGP signatures. Custom systems requiring signed local packages need a trusted OpenPGP signature; MpvKit does not change that policy.

After saving your settings, open media with system MPV as usual. MpvKit does not change your default player.

## License

[GPL-3.0-or-later](COPYING.md). Third-party components, fonts and dependencies retain their own licenses and notices.
