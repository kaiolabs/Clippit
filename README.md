# Clippit

**A local-first clipboard manager, launcher and AI assistant for GNOME.** Everything runs on your
computer: clipboard history, a Raycast-style launcher, a spell checker for any text field, a reply
assistant and an AI chat that can actually use your system. Nothing goes to the cloud unless you ask
for a web search.

🇧🇷 [Leia em português](README.pt-BR.md)

## What it does

| | Default shortcut |
|---|---|
| **Launcher** — apps, files, websites, math and unit conversions, system controls, window tiling, emoji, snippets | `Alt+Space` |
| **Clipboard history** — text, links, code and images; screenshots are OCR'd so you can search the text inside them; pin, transform, paste | `Alt+C` |
| **AI chat** — a local LLM with 38 tools: web search, open apps, move windows, volume/brightness, read and organize files, convert PDFs and images, git status, background build tasks, notes, reminders… | `Alt+I` |
| **Text actions** — summarize, translate, explain or rewrite the selected text | `Alt+A` |
| **Spell checker** — a badge next to any text field; one key fixes everything | `Alt+O` / `Alt+S` |
| **Reply assistant** — reads the conversation in the focused window (WhatsApp, Slack…) and suggests replies | `Alt+K` |
| **Game mode** — moves the AI off the GPU; games are detected automatically | `Ctrl+Shift+E` |
| **Doctor** — checks every part of Clippit every 2 minutes and fixes what breaks | — |

Every shortcut can be changed in **Settings → Atalhos**, and the AI models in **Settings → Corretor**.
The settings app also has a **Funcionalidades** page that explains each feature.

> The interface is in Brazilian Portuguese for now. Translations are welcome.

## Requirements

- **GNOME Shell 46 on Wayland** (tested on Zorin OS 18, based on Ubuntu 24.04), or **Hyprland / Sway / niri** with
  the Quickshell interface (see below). Other GNOME versions may need a
  one-line change in `metadata.json`.
- Debian/Ubuntu packages (the installer lists the missing ones).
- [Rust](https://rustup.rs) to build.
- For the AI: about 4 GB of disk. A GPU with Vulkan (NVIDIA, AMD or Intel) makes it fast; it also runs on
  the CPU with the small model.

## Install

```bash
git clone https://github.com/kaiolabs/Clippit.git
cd Clippit
./scripts/install.sh --deps
```

The installer builds everything, installs it for your user only (`~/.local`), starts the services,
sets up the shortcuts and offers to download the AI engine ([llama.cpp](https://github.com/ggml-org/llama.cpp))
and the default models ([Qwen 3.5](https://huggingface.co/unsloth)). On the first install, log out and
back in so GNOME loads the extension.

To remove it: `./scripts/uninstall.sh` (your data is only deleted if you say so).

### Hyprland, Sway, niri (Quickshell)

Outside GNOME the launcher, the spell checker and the assistant run as a [Quickshell](https://quickshell.org)
interface instead of the GNOME extension, with the same screens, keys and settings. `install.sh` picks it by
itself when GNOME isn't running; with Nix, set `programs.clippit.quickshell = true;` (and
`gnomeExtension = false;`).

- `clippit-shell` starts it (the `clippit-shell` user service does it at login). It registers Alt+Space, Alt+C,
  Alt+I, Alt+A, Alt+K and Ctrl+Shift+E with Hyprland or Sway by itself, again after every config reload.
  On niri, bind the keys in its config to `clippit-shell lancador`, `historico`, `ia`, `acoes`, `assistente`
  and `modo_jogo`.
- `clippit-wl` reads and edits the focused text field (input-method-v2) and draws the badge next to the
  cursor. Only one input method can run at a time: it doesn't work together with fcitx5 or ibus.
- Needs `quickshell`, `wtype`, `grim`, `wl-clipboard` (the Nix package brings them).
- The compositor must export `WAYLAND_DISPLAY` (and, on Hyprland, `HYPRLAND_INSTANCE_SIGNATURE`) to systemd,
  e.g. `systemctl --user import-environment WAYLAND_DISPLAY HYPRLAND_INSTANCE_SIGNATURE` at startup.

Differences from GNOME, because the compositor doesn't expose it: the suggestions panel opens at the mouse
(Hyprland) or at the bottom of the window instead of right above the cursor; the reply assistant reads the
whole window as it appears on screen (a notification or another window on top of it ends up in the
screenshot too); scrolling outside the launcher doesn't release it (a click does, on Hyprland); no blur behind
the panels. On Sway 1.12 there is no badge next to the cursor (Sway crashes when it closes); Alt+O still
opens the suggestions. Do Not Disturb needs swaync, mako or dunst; brightness needs brightnessctl.
`clippit-shell estado` shows what it sees (compositor, input method, shortcuts, spell checker).

### Nix and NixOS

The repository is a flake. It has the package, a Home Manager module (recommended: it also sets up the
services, the GNOME extension, the shortcuts and your settings), a NixOS module and a dev shell.

**Try it without installing:**

```bash
nix run github:kaiolabs/Clippit            # opens the settings app
nix shell github:kaiolabs/Clippit          # every clippit-* command in this shell
```

Without flakes enabled, add `--extra-experimental-features 'nix-command flakes'`.

**Home Manager.** Add the input to your `flake.nix`:

```nix
inputs.clippit = {
  url = "github:kaiolabs/Clippit";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then, in your Home Manager configuration (with `inputs` passed in `extraSpecialArgs`):

```nix
{ inputs, pkgs, ... }:
{
  imports = [ inputs.clippit.homeManagerModules.default ];

  programs.clippit = {
    enable = true;

    # ~/.config/clippit/corretor.conf: AI, models, spell checker, shortcuts
    settings = {
      device = "auto";
      nomes = [ "Maria Silva" "Maria" ];
      atalho_ia = "<Super>i";
    };

    # ~/.config/clippit/config.toml: history, privacy, OCR, language
    historySettings = {
      general.max_history_items = 500;
      ui.language = "pt";
    };
  };
}
```

Every key of `settings` and `historySettings` is in [docs/configuration.md](docs/configuration.md).
Only the keys you set are written: the settings app keeps working, and those keys go back to the Nix
value on the next `home-manager switch`.

Other options:

| Option | Default | Meaning |
|---|---|---|
| `package` | the flake's package | Use `.override { llama-cpp = pkgs.llama-cpp.override { vulkanSupport = true; }; }` for the AI on the GPU, or `.override { withAI = false; }` to leave the AI out. |
| `gnomeExtension` | `true` | Turn on the GNOME extension (added to the list, your other extensions stay). |
| `historyShortcut` | `"<Alt>c"` | GNOME shortcut for the history; `null` leaves GNOME's shortcuts alone. |
| `freeAltSpace` | `true` | Free Alt+Space (GNOME's window menu) for the launcher. |
| `popupAutostart` | `true` | Start the history popup hidden at login. |
| `doctor` | `true` | Run the doctor every 2 minutes. |

**NixOS only** (no Home Manager): installs the package for every user and starts the services. The
settings stay in the settings app.

```nix
{ inputs, ... }:
{
  imports = [ inputs.clippit.nixosModules.default ];
  programs.clippit.enable = true;
}
```

Or just the package, through the overlay: `nixpkgs.overlays = [ inputs.clippit.overlays.default ];` and
`environment.systemPackages = [ pkgs.clippit ];` (start the services yourself:
`systemctl --user enable --now clippit.service`).

**Development:** `nix develop` gives Rust, the GTK/Tesseract libraries and clippy, so `cargo build`
works like on Ubuntu.

Notes for Nix:

- The AI engine is the `llama-cpp` from nixpkgs (CPU by default). Don't use "install llama.cpp" in the
  settings app: it downloads an Ubuntu build that doesn't run on NixOS. Models are downloaded as usual.
- The GNOME extension declares GNOME Shell 46 in `metadata.json`; on newer versions GNOME may refuse to
  load it.
- Voice dictation needs `faster-whisper`, which is not in the package.

## Privacy

- History, AI, spell checking, voice dictation and OCR run locally.
- The internet is only used when you ask (web search, news, exchange rates, weather) and to download
  models.
- The AI never reads files that look like keys or passwords, and anything that changes your system
  (moving files, killing a program, running a command) waits for you to press **Rodar**.

## Documentation

- [Contributing](CONTRIBUTING.md) — how the project is organized, how to run it in development, the
  scripts, the rules for code, commits and pull requests.
- [Configuration](docs/configuration.md) — every setting in `config.toml` and `corretor.conf`.
- [Changelog](CHANGELOG.md)

## License

[MIT](LICENSE)
