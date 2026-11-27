#!/usr/bin/env bash
# Clippit installer — builds and installs everything for the current user.
#
#   ./scripts/install.sh            build + install (asks before downloading the AI)
#   ./scripts/install.sh --deps     also installs the system packages with apt (asks for sudo)
#   ./scripts/install.sh --no-ai    skip the AI engine and models (history and launcher still work)
#   ./scripts/install.sh --ai       download the AI engine and default models without asking
#
# Nothing is installed system-wide: binaries go to ~/.local/bin, data to
# ~/.local/share/clippit, settings to ~/.config/clippit.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$HOME/.local/bin"
EXT_UUID="clippit-corretor@clippit"
EXT_DIR="$HOME/.local/share/gnome-shell/extensions"
UNITS="$HOME/.config/systemd/user"
APPS="$HOME/.local/share/applications"

WANT_DEPS=0
AI="ask"
for a in "$@"; do
    case "$a" in
        --deps) WANT_DEPS=1 ;;
        --no-ai) AI="no" ;;
        --ai) AI="yes" ;;
        -h|--help) sed -n '2,11p' "$0"; exit 0 ;;
        *) echo "Unknown option: $a"; exit 1 ;;
    esac
done

# A path as the command in a .desktop Exec= line (Desktop Entry spec): quoted when it has spaces or
# other special characters, with ", `, $ and \ escaped (twice: quoting rule + string escape rule).
desktop_exec() {
    local p=$1
    if [[ "$p" =~ [^A-Za-z0-9_./+-] ]]; then
        p=${p//\\/\\\\\\\\}
        p=${p//\"/\\\\\"}
        p=${p//\`/\\\\\`}
        p=${p//\$/\\\\\$}
        p=${p//%/%%}
        printf '"%s"' "$p"
    else
        printf '%s' "${p//%/%%}"
    fi
}

say() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
ok() { printf '    ✓ %s\n' "$*"; }
warn() { printf '    ! %s\n' "$*"; }

# ---------------------------------------------------------------- 1. dependencies
say "Checking system packages"
PKGS=(build-essential pkg-config libgtk-4-dev libadwaita-1-dev libtesseract-dev libleptonica-dev
      tesseract-ocr tesseract-ocr-por tesseract-ocr-eng hunspell-pt-br hunspell-en-us python3 python3-gi
      gir1.2-glib-2.0 wl-clipboard imagemagick poppler-utils ghostscript playerctl libnotify-bin curl sqlite3)
MISSING=()
if command -v dpkg >/dev/null; then
    for p in "${PKGS[@]}"; do
        dpkg -s "$p" >/dev/null 2>&1 || MISSING+=("$p")
    done
fi
if ((${#MISSING[@]})); then
    if ((WANT_DEPS)); then
        sudo apt-get update && sudo apt-get install -y "${MISSING[@]}"
    else
        warn "Missing: ${MISSING[*]}"
        warn "Install with:  sudo apt install ${MISSING[*]}   (or rerun with --deps)"
        exit 1
    fi
fi
ok "system packages"
if ! command -v cargo >/dev/null; then
    warn "Rust is not installed. Get it from https://rustup.rs and run this again."
    exit 1
fi
ok "rust $(rustc --version | cut -d' ' -f2)"

# ---------------------------------------------------------------- 2. build
say "Building (release)"
cargo build --release --manifest-path "$ROOT/Cargo.toml" -p clippit-daemon -p clippit-popup -p clippit-dashboard
ok "built"

# ---------------------------------------------------------------- 3. binaries and scripts
say "Installing to $BIN"
mkdir -p "$BIN"
systemctl --user stop clippit.service 2>/dev/null || true
for b in clippit-daemon clippit-popup clippit-dashboard; do
    install -m755 "$ROOT/target/release/$b" "$BIN/$b"
done
# Scripts are linked, so a `git pull` updates them.
for s in clippit-correct clippit-busca clippit-historico clippit-doutor clippit-jogos-nvidia; do
    chmod +x "$ROOT/scripts/$s"
    ln -sfn "$ROOT/scripts/$s" "$BIN/$s"
done
ok "binaries and scripts"
case ":$PATH:" in *":$BIN:"*) ;; *) warn "$BIN is not in your PATH (add it to ~/.profile)";; esac

# ---------------------------------------------------------------- 4. GNOME extension
say "Installing the GNOME Shell extension"
mkdir -p "$EXT_DIR"
ln -sfn "$ROOT/gnome-extension/$EXT_UUID" "$EXT_DIR/$EXT_UUID"
gsettings set org.gnome.shell disable-user-extensions false 2>/dev/null || true
if gnome-extensions enable "$EXT_UUID" 2>/dev/null; then
    ok "extension enabled"
else
    warn "GNOME only sees new extensions after you log out and back in. Then run:"
    warn "  gnome-extensions enable $EXT_UUID"
fi

# ---------------------------------------------------------------- 5. services
say "Starting the services"
mkdir -p "$UNITS"
cp "$ROOT/assets/systemd/clippit.service" "$ROOT/assets/systemd/clippit-doutor.service" \
   "$ROOT/assets/systemd/clippit-doutor.timer" "$UNITS/"
systemctl --user daemon-reload
systemctl --user enable --now clippit.service clippit-doutor.timer
ok "clipboard history (clippit.service) and doctor (clippit-doutor.timer)"

# ---------------------------------------------------------------- 6. shortcuts and menu entry
say "Shortcuts and menu entry"
KB=org.gnome.settings-daemon.plugins.media-keys
KP=/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/
cur=$(gsettings get $KB custom-keybindings)
if [[ "$cur" != *"$KP"* ]]; then
    if [[ "$cur" == "@as []" || "$cur" == "[]" ]]; then new="['$KP']"; else new="${cur%]*}, '$KP']"; fi
    gsettings set $KB custom-keybindings "$new"
fi
gsettings set "$KB.custom-keybinding:$KP" name "Clippit"
gsettings set "$KB.custom-keybinding:$KP" command "$BIN/clippit-historico"
gsettings set "$KB.custom-keybinding:$KP" binding "<Alt>c"
ok "Alt+C opens the history"
# Alt+Space opens the launcher; GNOME uses it for the window menu by default.
if gsettings get org.gnome.desktop.wm.keybindings activate-window-menu | grep -q "<Alt>space"; then
    gsettings set org.gnome.desktop.wm.keybindings activate-window-menu "[]"
    ok "freed Alt+Space (it was GNOME's window menu)"
fi
mkdir -p "$APPS" "$HOME/.local/share/icons/hicolor/256x256/apps"
install -m644 "$ROOT/assets/logo_clippit.png" "$HOME/.local/share/icons/hicolor/256x256/apps/clippit.png"
# GNOME starts with a PATH that lacks ~/.local/bin: with a bare "Exec=clippit-dashboard" it can't
# find the program and hides the app from the menu and search. So use the full path.
DASH_EXEC=$(desktop_exec "$BIN/clippit-dashboard")
DASH_TRY=${BIN//\\/\\\\}/clippit-dashboard
DASH_EXEC="$DASH_EXEC" DASH_TRY="$DASH_TRY" awk '
    /^Exec=clippit-dashboard/ { print "Exec=" ENVIRON["DASH_EXEC"]; print "TryExec=" ENVIRON["DASH_TRY"]; next }
    { print }
' "$ROOT/assets/clippit.desktop" > "$APPS/clippit.desktop"
chmod 644 "$APPS/clippit.desktop"
update-desktop-database "$APPS" 2>/dev/null || true
ok "\"Clippit\" in the app menu opens the settings"

# ---------------------------------------------------------------- 7. AI engine and models
if [[ "$AI" == "ask" ]]; then
    printf '\nDownload the local AI now? llama.cpp + 2 models, about 3.5 GB. [Y/n] '
    read -r r
    [[ "$r" =~ ^[Nn] ]] && AI="no" || AI="yes"
fi
if [[ "$AI" == "yes" ]]; then
    say "Downloading the local AI (you can change models later in Settings → Corretor)"
    "$BIN/clippit-correct" --modelos | grep -q '"llama": null' && "$BIN/clippit-correct" --instalar-llama | tail -1
    "$BIN/clippit-correct" --baixar qwen3.5-0.8b | tail -1
    if command -v nvidia-smi >/dev/null || [ -e /dev/dri/renderD129 ]; then
        "$BIN/clippit-correct" --baixar qwen3.5-4b | tail -1
    fi
    ok "AI ready"
else
    warn "AI skipped. Later: Settings → Corretor → Modelos, or ./scripts/install.sh --ai"
fi

say "Done"
echo "    Alt+Space  launcher      Alt+C  history      Alt+I  AI chat"
echo "    Settings: open \"Clippit\" from the app menu (Funcionalidades lists everything)."
