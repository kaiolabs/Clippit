# Configuration

Most settings are in the **Clippit settings app** (open "Clippit" from the app menu). This page lists
every option for people who prefer editing files. Both files live in `~/.config/clippit/` and are
re-read automatically: no restart needed, except where noted.

## `corretor.conf` — extension, AI and shortcuts

Plain `key = value` lines; `#` starts a comment. Every key is optional.

### AI and models

| Key | Default | Meaning |
|---|---|---|
| `device` | `auto` | Where the AI runs: `auto` (NVIDIA GPU only when it has room, otherwise CPU), `nvidia`, `amd`, `intel`, `cpu`. |
| `model_gpu` | `~/.local/share/clippit/models/Qwen3.5-4B-Q4_K_M.gguf` | Model used on the GPU. |
| `model_cpu` | `~/.local/share/clippit/models/Qwen3.5-0.8B-Q4_K_M.gguf` | Model used on the CPU (game mode, busy GPU, or no GPU). |
| `model` | — | If set, overrides both of the above. |
| `model_visao` | the `mmproj-…-F16.gguf` matching `model_gpu` | Vision projector so the GPU model can read screenshots. |
| `visao` | `true` | Allow vision (only on the GPU, only with enough free VRAM). |
| `llama_dir` | newest `~/.local/share/clippit/llama.cpp/llama-*`, then `llama-server` in `PATH` | Folder with `llama-server`. |
| `threads` | half of the CPU cores (2–8) | CPU threads for the model. |
| `cpu_mask` | — | Pin the model to some cores, e.g. `555` = performance cores of a hybrid Intel CPU. |
| `idle_seconds` | `60` | The model unloads after this many seconds without use. |
| `especulativo` | `false` | Speculative decoding with the small model (no gain measured on Qwen 3.5). |

Change models from the settings app (**Corretor → Modelos**), or from the terminal:

```bash
clippit-correct --modelos               # catalog, downloaded files, current choice (JSON)
clippit-correct --baixar qwen3.5-2b     # download a model from the catalog
clippit-correct --instalar-llama vulkan # (re)install the AI engine; or "cpu"
clippit-correct --stop                  # unload now so the next use picks up the change
```

Any GGUF model that `llama-server` can run works: put it in `~/.local/share/clippit/models/` and select
it, or point `model_gpu` / `model_cpu` to it.

### Corrector and assistant

| Key | Default | Meaning |
|---|---|---|
| `ativo` | `true` | Show the spell-check badge next to text fields. |
| `verificar_sozinho` | `true` | Check while typing; `false` = only when you click the badge. |
| `aprender` | `true` | Learn from accepted fixes and sent replies (stored locally). |
| `nomes` | — | How you appear in chats, comma-separated (`Maria Silva, Maria`), so the assistant knows which messages are yours. |
| `pastas_projetos` | — | Extra folders (comma-separated) where the AI looks for your git projects; your home folder is always searched. |
| `voz_modelo` | `small` | faster-whisper model for dictation (`tiny`, `base`, `small`, `medium`…). |
| `voz_idioma` | `pt` | Dictation language. |
| `brave_api_key` | — | Optional Brave Search key for fuller web results (otherwise DuckDuckGo). |

### Shortcuts

The extension checks these every 2 seconds and swaps the shortcut live. Values use the GTK accelerator
format; `desligado` disables a shortcut. Easier: **Settings → Atalhos → Trocar**.

| Key | Default | Action |
|---|---|---|
| `atalho_lancador` | `<Alt>space` | Open the launcher |
| `atalho_ia` | `<Alt>i` | Open the AI chat |
| `atalho_acoes` | `<Alt>a` | Actions on the selected text |
| `atalho_assistente` | `<Alt>k` | Reply assistant |
| `atalho_modo_jogo` | `<Control><Shift>e` | Toggle game mode |
| `atalho_sugestoes` | `<Alt>o` | Open the spell-check suggestions |
| `atalho_aplicar` | `<Alt>s` | Apply all corrections |
| `atalho_ajuste` | `<Alt>f` | Type an adjustment in the reply assistant |

Alt+C (history) is a GNOME custom shortcut; change it in **Settings → Atalhos** or in GNOME Settings →
Keyboard.

### Example

```ini
device = auto
model_gpu = ~/.local/share/clippit/models/Qwen3.5-4B-Q4_K_M.gguf
model_cpu = ~/.local/share/clippit/models/Qwen3.5-0.8B-Q4_K_M.gguf
nomes = Maria Silva, Maria
pastas_projetos = ~/dev
atalho_ia = <Super>i
```

## `config.toml` — history (daemon and popup)

Created on first run. The settings app edits the useful parts (**Geral**). The daemon reads it on
start: restart it after editing by hand (`systemctl --user restart clippit`).

| Section / key | Default | Meaning |
|---|---|---|
| `general.max_history_items` | `100` | Items kept; older ones are removed. |
| `general.poll_interval_ms` | `200` | How often the clipboard is checked. |
| `general.max_text_size` | `10485760` | Larger texts are not saved (bytes). |
| `privacy.enable_image_capture` | `true` | Save copied images and screenshots. |
| `privacy.max_image_size_mb` | `10` | Larger images are not saved. |
| `features.enable_ocr` | `true` | Read the text in images so you can search for it. |
| `ocr.languages` | `por+eng` | Tesseract languages. |
| `ui.language` | `en` | Language of the settings app and popup (`pt`, `en`). |
| `ui.show_notifications` | `true` | Notify on each copy (popup only). |
| `hotkeys.show_history_modifier` / `show_history_key` | `alt` / `c` | The history shortcut. |
