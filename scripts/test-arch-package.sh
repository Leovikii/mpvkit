#!/usr/bin/env bash
# Package payload validation only: isolated database/root, no host package writes.
set -euo pipefail
package=$(realpath "$1")
package_root=$(mktemp -d /tmp/mpvkit-package-test.XXXXXXXX)
mkdir -p "$package_root/var/lib/pacman" "$package_root/cache" "$package_root/hooks"
args=(--root "$package_root" --dbpath "$package_root/var/lib/pacman" --logfile "$package_root/pacman.log" --cachedir "$package_root/cache" --hookdir "$package_root/hooks" --noscriptlet --noconfirm)
pacman "${args[@]}" -dd -U "$package"
test "$(pacman --root "$package_root" --dbpath "$package_root/var/lib/pacman" -Qq)" = mpvkit-bin
test -x "$package_root/usr/bin/mpvkit"
test -f "$package_root/usr/share/applications/io.github.mpvkit.desktop"
desktop-file-validate "$package_root/usr/share/applications/io.github.mpvkit.desktop"
cmp src-tauri/icons/icon.png "$package_root/usr/share/icons/hicolor/256x256/apps/io.github.mpvkit.png"
test -f "$package_root/usr/lib/mpvkit/third-party/APP-DEPENDENCY-NOTICES.txt"
# Reinstallation validates the upgrade transaction without changing the host.
pacman "${args[@]}" -dd -U "$package"
pacman "${args[@]}" -dd -R mpvkit-bin
test ! -e "$package_root/usr/bin/mpvkit"
echo "PASS: isolated package install, reinstall, resources and uninstall ($package_root)"
