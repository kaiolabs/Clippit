# Contributing to Clippit

Thanks for helping! This document is the contract for working on Clippit: how it is built, how to run
it while you develop, what each script does, and the rules every change must follow. Read it once
before your first pull request.

🇧🇷 Pode escrever issues e PRs em português ou inglês.

---

## 1. How Clippit is put together

Clippit is a few small programs that talk to each other on the user's machine. Nothing runs as root
and nothing is installed outside `~/.local`.

| Part | Language | Where | What it does |
|---|---|---|---|
| **clippit-daemon** | Rust | `crates/clippit-daemon` | Systemd user service (`clippit.service`). Watches the clipboard, saves text and images to SQLite, OCRs images, answers IPC from the other programs. Also registers Alt+C through X11 as a fallback outside GNOME. |
| **GNOME extension** | GJS | `gnome-extension/clippit-corretor@clippit` | Everything on screen: launcher (Alt+Space), history, AI chat, text actions, spell-check badge, reply assistant. Exposes the D-Bus service `com.clippit.Lancador` (open a view, list/move windows, take a screenshot, clipboard hash). |
| **clippit-correct** | Python | `scripts/clippit-correct` | The AI backend. A local HTTP server on `127.0.0.1:18088` that starts `llama-server` (port 18089) and an embeddings server (18087) on demand and shuts them down when idle. Spell checker, reply assistant, the chat agent and its tools, model catalog and downloads. |
| **clippit-dashboard** | Rust (GTK 4 + libadwaita) | `crates/clippit-dashboard` | The settings app ("Clippit" in the app menu). |
| **clippit-popup** | Rust (GTK 4) | `crates/clippit-popup` | The older history window. Used only when the extension is not running. |
| **clippit-core / clippit-ipc** | Rust | `crates/` | Shared code: config, SQLite storage, i18n, the IPC protocol. |
| **helper scripts** | Python / sh | `scripts/` | See [section 3](#3-scripts). |

### Data and settings (per user)

| Path | Contents |
|---|---|
| `~/.config/clippit/config.toml` | Daemon and popup settings (history size, images, OCR, language, Alt+C). |
| `~/.config/clippit/corretor.conf` | Extension and AI settings (`key = value`): models, GPU, shortcuts, names. See [docs/configuration.md](docs/configuration.md). |
| `~/.local/share/clippit/history.db` | Clipboard history (SQLite) and `images/`. |
| `~/.local/share/clippit/models/` | AI models (`.gguf`). |
| `~/.local/share/clippit/llama.cpp/` | The llama.cpp build downloaded by the installer. |
| `~/.local/share/clippit/aprendizado/` | What the corrector and assistant learned (local only). |
| `~/.local/share/clippit/doutor/` | The doctor's log, state and backups of data files. |
| `$XDG_RUNTIME_DIR/clippit-correct/` | Runtime files of the AI server (`server.log`, status, game-mode flag). |

## 2. Running it while you develop

```bash
./scripts/install.sh --deps     # once: builds, installs, links the scripts, enables the extension
```

The installer **links** the scripts and the extension to your clone, so most edits are live:

- **Python scripts** (`scripts/clippit-*`): the next call uses the new code. To restart the AI server:
  `clippit-correct --stop` (it starts again on the next use).
- **Extension** (`core.js`): the loader in `extension.js` reloads `core.js` about one second after the
  file is rewritten. Changes to `extension.js` or `metadata.json` need a log out / log in.
- **Rust programs**: rebuild and reinstall the one you changed:
  ```bash
  cargo build --release -p clippit-dashboard && install -m755 target/release/clippit-dashboard ~/.local/bin/
  cargo build --release -p clippit-daemon && install -m755 target/release/clippit-daemon ~/.local/bin/ \
    && systemctl --user restart clippit
  ```

### Logs

```bash
journalctl --user -f | grep clippit-corretor          # extension
journalctl --user -u clippit -f                       # daemon
tail -f $XDG_RUNTIME_DIR/clippit-correct/server.log   # llama-server
tail ~/.local/share/clippit/doutor/doutor.log         # doctor
clippit-doutor --relatorio                            # health of every part
```

### Testing without disturbing your session

- **Settings app**: run it in a separate GTK Broadway display and look at it in a browser:
  ```bash
  gtk4-broadwayd :7 &
  GDK_BACKEND=broadway BROADWAY_DISPLAY=:7 XDG_CONFIG_HOME=/tmp/clippit-test ADW_DISABLE_PORTAL=1 \
    dbus-run-session -- ./target/release/clippit-dashboard
  # open http://127.0.0.1:8087 — and close it when done (pkill -x gtk4-broadwayd)
  ```
  `dbus-run-session` matters: the app is single-instance, so a test copy on the shared session bus
  would swallow the user's click on "Clippit" in the app menu and nothing would appear.
  `ADW_DISABLE_PORTAL=1` stops libadwaita from waiting for a desktop portal on that private bus
  (without it the page stays blank).
  A separate `XDG_CONFIG_HOME` keeps your real settings untouched. Put a `gtk-4.0/settings.ini` with
  `gtk-xft-dpi=98304` there, or the pages render too narrow.
- **Extension screenshots**: write an output folder to `$XDG_RUNTIME_DIR/clippit-corretor/selftest-foto`
  and rewrite `core.js`; on load the extension photographs its main views into that folder. This probe
  does not load the AI. (`selftest-lancador` and `selftest-novidades` do load the AI: don't run them
  while gaming.)

## 3. Scripts

| Script | Purpose |
|---|---|
| `scripts/install.sh` | Installs everything for the current user. `--deps` installs apt packages, `--ai` / `--no-ai` controls the AI download. |
| `scripts/uninstall.sh` | Removes what the installer added; asks before deleting data. |
| `scripts/clippit-correct` | AI backend and CLI: `--status`, `--stop`, `--modelos`, `--baixar <id>`, `--instalar-llama`, `--prova` (corrector benchmark), `--modo-jogo on\|off`. Run with `-h` for all. |
| `scripts/clippit-busca` | Queries used by the launcher: history, files (GNOME index), emoji, pin/unpin. |
| `scripts/clippit-historico` | What Alt+C runs: opens the history in the launcher, or the old popup if the extension is off. |
| `scripts/clippit-doutor` | The doctor: checks and fixes known problems; unknown errors go to the local AI. Run by `clippit-doutor.timer` every 2 minutes. |
| `scripts/clippit-jogos-nvidia` | On hybrid-GPU laptops, makes game launchers use the NVIDIA GPU. `--desfazer` reverts. |
| `scripts/gerar-icones.py` | Regenerates the Lucide icons used by the extension. |
| `scripts/prova-corretor.jsonl` | Test sentences for `clippit-correct --prova`. |

## 4. Rules for code

### Everyone

- **Privacy first.** Never send user data anywhere unless the user explicitly asked for it (a web
  search). No telemetry. No personal data, keys, tokens or machine-specific paths in the repository.
- **Nothing that changes the system runs without the user's OK.** In the AI backend, any tool that
  writes, moves, kills or runs commands must go through the confirmation flow (`PENDING` → the user
  presses *Rodar*). Read-only tools may run directly. Files matching `SECRET_PATH` are never read.
- **Don't hog the GPU.** The AI only uses the GPU with enough free VRAM (`GPU_FREE_NEEDED`) and leaves
  it when a game starts. Keep it that way.
- **User-facing text is Brazilian Portuguese** (for now). Code comments may be in Portuguese or English.

### Rust

- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` must pass (CI checks).
- No `unwrap()` on anything that can fail at runtime in the daemon; it must keep running.

### Python (`scripts/`)

- Standard library only (plus the system `gi` bindings). No pip dependencies for the core features.
- Scripts start with `#!/usr/bin/env -S python3 -I`.

### GNOME extension (GJS)

These rules come from real crashes. Breaking them can log the user out:

- **Never set `min_height` / `min_width` on children of `St.BoxLayout`.** On GNOME 46 the
  "natural height < minimum" warning aborts the whole shell. For scrollable text use `FullHeightBox`.
- **Never `GLib.source_remove()` a source that already ended** (returned `SOURCE_REMOVE`); keep a flag.
- **`core.js` reloads as soon as it is saved**: a syntax error disables the extension until the next
  save. Check first: `cp core.js /tmp/c.mjs && node --check /tmp/c.mjs`.
- **One visual standard.** Use the tokens and components at the top of `core.js` (`T`, `TX`, `UI`,
  `emptyState`, `keycap`, `makeFooter`, `iconTile`). Glass surfaces, 4–8 px corners, no divider lines,
  blue (`#4c9dff`) only for primary actions.
- Global shortcuts come from `SHORTCUTS` + `keyOf()` so users can change them; don't hard-code keys.

## 5. Git workflow

### Branches

- `master` is protected: no direct pushes, no force pushes. Everything goes through a pull request.
- Create a branch from `master` named `<type>/<short-description>`:
  `feat/window-tools`, `fix/history-stops-after-crash`, `docs/install`, `chore/cleanup`.

### Commits — [Conventional Commits](https://www.conventionalcommits.org)

```
<type>(<optional scope>): <what changed, in the imperative>

<why, if it isn't obvious>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.
Scopes: `daemon`, `painel` (dashboard), `extensao`, `ia`, `corretor`, `doutor`, `popup`.
English or Portuguese, but keep one language per commit. Example:
`fix(daemon): reconnect the clipboard when Xwayland restarts`.

### Pull requests — what it takes to merge

1. **CI is green** (format, clippy, tests, build, script and extension syntax).
2. **One approving review** from a code owner ([CODEOWNERS](.github/CODEOWNERS)).
3. **All review conversations resolved.**
4. The branch is **up to date with `master`**.
5. Merged with **squash**: one commit per PR on `master`, titled like a Conventional Commit.
   The branch is deleted after the merge.

Keep PRs small and about one thing. Fill in the template: what changed, why, how you tested it, and
screenshots for anything visual. Add a line to `CHANGELOG.md` under **Unreleased**.

### Versions

[Semantic Versioning](https://semver.org). The version lives in the workspace `Cargo.toml`; releases are
tagged `vX.Y.Z` from `master`.

## 6. Bugs and security

- Bugs and ideas: open an issue (there are templates). Include `clippit-doutor --relatorio` output.
- Security problems: **don't open a public issue** — see [SECURITY.md](SECURITY.md).
- Be kind: [Code of Conduct](CODE_OF_CONDUCT.md).
