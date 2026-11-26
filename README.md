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

- **GNOME Shell 46 on Wayland** (tested on Zorin OS 18, based on Ubuntu 24.04). Other versions may need a
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
