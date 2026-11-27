#!/usr/bin/env bash
# Clippit uninstaller — removes what scripts/install.sh installed.
# Your history, models and settings are only removed if you say so.

set -uo pipefail

BIN="$HOME/.local/bin"
EXT_UUID="clippit-corretor@clippit"
UNITS="$HOME/.config/systemd/user"

echo "==> Stopping services"
systemctl --user disable --now clippit.service clippit-doutor.timer clippit-shell.service 2>/dev/null
"$BIN/clippit-correct" --stop 2>/dev/null
rm -f "$UNITS/clippit.service" "$UNITS/clippit-doutor.service" "$UNITS/clippit-doutor.timer" "$UNITS/clippit-shell.service"
systemctl --user daemon-reload

echo "==> Removing the GNOME extension"
gnome-extensions disable "$EXT_UUID" 2>/dev/null
rm -f "$HOME/.local/share/gnome-shell/extensions/$EXT_UUID"

echo "==> Removing programs"
for f in clippit-daemon clippit-popup clippit-dashboard clippit-correct clippit-busca clippit-historico \
         clippit-doutor clippit-jogos-nvidia clippit-shell clippit-wl; do
    rm -f "${BIN:?}/$f"
done
rm -f "$HOME/.local/share/applications/clippit.desktop" "$HOME/.local/share/icons/hicolor/256x256/apps/clippit.png"

echo "==> Removing the Alt+C shortcut"
KB=org.gnome.settings-daemon.plugins.media-keys
KP=/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/
cur=$(gsettings get $KB custom-keybindings 2>/dev/null)
new=$(printf '%s' "$cur" | sed "s#, '$KP'##; s#'$KP', ##; s#'$KP'##")
[ "$new" = "[]" ] && new="@as []"
gsettings set $KB custom-keybindings "$new" 2>/dev/null
gsettings reset-recursively "$KB.custom-keybinding:$KP" 2>/dev/null
gsettings reset org.gnome.desktop.wm.keybindings activate-window-menu 2>/dev/null

read -r -p "Delete your clipboard history, learned data and AI models (~/.local/share/clippit)? [y/N] " r
if [[ "$r" =~ ^[Yy] ]]; then rm -rf "${HOME:?}/.local/share/clippit"; echo "    deleted"; fi
read -r -p "Delete your settings (~/.config/clippit)? [y/N] " r
if [[ "$r" =~ ^[Yy] ]]; then rm -rf "${HOME:?}/.config/clippit"; echo "    deleted"; fi

echo "Clippit was removed."
