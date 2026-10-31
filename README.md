# MpvKit

A simple interface for installing, configuring and updating MPV on **Windows x64** and **Arch Linux / CachyOS x86_64 (KDE Wayland)**.

Current preview: **0.2.0-beta.1**. See the [test status and known limits](docs/v0.2.0-beta.1/plan.md).

## Features

- Edit playback settings before installation in a narrow two-page interface.
- Install configuration and playback extras together; manage system MPV in Settings.
- Three playback presets, HDR options, naturally sorted folder playlists, subtitle discovery and playback resume.
- Preview configuration changes and protect manual edits when saving. Confirmed full removal deletes the entire MPV configuration folder; removing system MPV is optional.
- Simplified Chinese and English, with system, light and dark themes.

See [Preset Design](docs/presets.md) for defaults and configuration choices.

## Installation

Download packages from [Releases](https://github.com/Leovikii/mpvkit/releases). Preview builds are marked **Pre-release**.

**Windows:** Download `MpvKit_<version>_x64-setup.exe` from [Releases](https://github.com/Leovikii/mpvkit/releases) and run it to install for the current user. Edit your settings on **Configuration**, then choose **Install all**. Core management uses WinGet and requires Windows App Installer.

**Arch Linux / CachyOS:** Download `mpvkit-bin-<version>-<release>-x86_64.pkg.tar.zst` from Releases and install it with Pacman. For the current preview:

```sh
sudo pacman -U ./mpvkit-bin-0.2.0beta1-1-x86_64.pkg.tar.zst
```

Launch MpvKit from the application menu and choose **Install all** on **Configuration**. If MPV is missing, MpvKit opens a native terminal for Pacman first; package operations require polkit authorization. For future app updates, open **Settings → Check for updates**. MpvKit downloads and verifies the package, shows the installation preview, then requests system authorization for Pacman. Regular `pacman -Syu` does not update this locally installed package.

Both platforms keep progress and results in one dialog, with one app confirmation before installation. System MPV operations keep package-manager prompts in a native terminal. Preview updates require **Include preview releases**. Linux app updates require a system installation managed by Pacman. Successful app updates request an automatic restart.

For a single download, checksum and installation command, see the [release notes](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-beta.1). Packages use Tauri update signatures; these are not Pacman OpenPGP signatures. Custom systems requiring signed local packages need a trusted OpenPGP signature; MpvKit does not change that policy.

After saving your settings, open media with system MPV as usual. MpvKit does not change your default player.

## License

[GPL-3.0-or-later](COPYING.md). Third-party components, fonts and dependencies retain their own licenses and notices.
