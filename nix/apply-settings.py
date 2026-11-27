"""Applies the Home Manager options of Clippit (nix/home-manager.nix).

The settings app also writes corretor.conf and config.toml, so they cannot be
read-only links to the Nix store: only the keys set in Nix are written, the
rest of each file is kept. Every `home-manager switch` writes them again.

    apply-settings.py PLAN.json
"""

import ast
import json
import os
import re
import subprocess
import sys
import tomllib

import tomli_w

KEYBINDINGS = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings"
CLIPPIT_KEYBINDING = f"{KEYBINDINGS}/clippit/"


def read_text(path):
    try:
        with open(path, encoding="utf-8") as f:
            return f.read()
    except FileNotFoundError:
        return None


def update_file(path, transform):
    """Rewrites `path` with transform(current text, or None if absent).

    The settings app writes the same files, so the file is read again right
    before the swap: if it changed meanwhile, the new text is computed again
    from the latest content instead of overwriting the other write.
    """
    for _ in range(5):
        current = read_text(path)
        text = transform(current)
        if text is None or text == current:
            return False
        if read_text(path) != current:
            continue
        os.makedirs(os.path.dirname(path), exist_ok=True)
        temporary = f"{path}.clippit-tmp"
        with open(temporary, "w", encoding="utf-8") as f:
            f.write(text)
        os.replace(temporary, path)
        return True
    raise RuntimeError(f"{path} keeps changing, giving up")


def conf_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return ", ".join(str(v) for v in value)
    return str(value)


def leaf_paths(tree, prefix=()):
    for key, value in tree.items():
        if isinstance(value, dict):
            yield from leaf_paths(value, prefix + (key,))
        else:
            yield list(prefix + (key,))


def remove_path(tree, keys):
    node = tree
    parents = []
    for key in keys[:-1]:
        if not isinstance(node.get(key), dict):
            return
        parents.append((node, key))
        node = node[key]
    node.pop(keys[-1], None)
    for parent, key in reversed(parents):
        if parent[key]:
            break
        del parent[key]


def apply_corretor(path, settings, stale=()):
    def transform(text):
        lines = (text or "").splitlines()
        for key in stale:
            pattern = re.compile(rf"^\s*{re.escape(key)}\s*=")
            lines = [current for current in lines if not pattern.match(current)]
        for key, value in settings.items():
            line = f"{key} = {conf_value(value)}"
            pattern = re.compile(rf"^\s*{re.escape(key)}\s*=")
            indexes = [i for i, current in enumerate(lines) if pattern.match(current)]
            if not indexes:
                lines.append(line)
                continue
            lines[indexes[0]] = line
            for i in reversed(indexes[1:]):
                del lines[i]
        return "\n".join(lines) + "\n"

    return update_file(path, transform)


def deep_merge(base, over):
    for key, value in over.items():
        if isinstance(value, dict) and isinstance(base.get(key), dict):
            deep_merge(base[key], value)
        else:
            base[key] = value
    return base


def apply_history(path, settings, stale=()):
    def transform(text):
        try:
            current = tomllib.loads(text or "")
        except tomllib.TOMLDecodeError:
            print(f"clippit: {path} is not valid TOML, leaving it as is", file=sys.stderr)
            return None
        for keys in stale:
            remove_path(current, keys)
        return tomli_w.dumps(deep_merge(current, settings))

    return update_file(path, transform)


class Dconf:
    def __init__(self, exe):
        self.exe = exe

    def read(self, key):
        p = subprocess.run([self.exe, "read", key], capture_output=True, text=True)  # nosemgrep: argv list without a shell; values come from the Nix plan
        if p.returncode != 0:
            raise RuntimeError(f"dconf read {key} failed: {p.stderr.strip()}")
        return p.stdout.strip()

    def write(self, key, value):
        subprocess.run([self.exe, "write", key, value], check=True)  # nosemgrep: argv list without a shell; values come from the Nix plan

    def read_list(self, key):
        raw = self.read(key).removeprefix("@as").strip()
        try:
            return list(ast.literal_eval(raw)) if raw else []
        except (ValueError, SyntaxError):
            return []

    def add_to_list(self, key, item):
        items = self.read_list(key)
        if item not in items:
            self.write(key, str(items + [item]))

    def remove_from_list(self, key, item):
        items = self.read_list(key)
        if item in items:
            self.write(key, str([i for i in items if i != item]) if len(items) > 1 else "@as []")


def gvariant_string(text):
    return "'" + text.replace("\\", "\\\\").replace("'", "\\'") + "'"


WINDOW_MENU = "/org/gnome/desktop/wm/keybindings/activate-window-menu"
EXTENSIONS = "/org/gnome/shell/enabled-extensions"


def apply_gnome(plan, previous):
    """Returns what this run manages, to be undone when the options go away."""
    dconf = Dconf(plan["dconf"])
    managed = {}

    extension = plan["extension"]
    if previous.get("extension") and previous["extension"] != extension:
        dconf.remove_from_list(EXTENSIONS, previous["extension"])
    if extension:
        dconf.add_to_list(EXTENSIONS, extension)
        managed["extension"] = extension

    shortcut = plan["historyShortcut"]
    if shortcut:
        dconf.add_to_list(KEYBINDINGS, CLIPPIT_KEYBINDING)
        dconf.write(f"{CLIPPIT_KEYBINDING}name", "'Clippit'")
        dconf.write(f"{CLIPPIT_KEYBINDING}command", gvariant_string(shortcut["command"]))
        dconf.write(f"{CLIPPIT_KEYBINDING}binding", gvariant_string(shortcut["binding"]))
        managed["shortcut"] = True
    elif previous.get("shortcut"):
        dconf.remove_from_list(KEYBINDINGS, CLIPPIT_KEYBINDING)

    freed = previous.get("freedAltSpace", False)
    if plan["freeAltSpace"]:
        bindings = dconf.read_list(WINDOW_MENU)
        if "<Alt>space" in bindings:
            remaining = [b for b in bindings if b != "<Alt>space"]
            dconf.write(WINDOW_MENU, str(remaining) if remaining else "@as []")
            freed = True
        managed["freedAltSpace"] = freed
    elif freed:
        dconf.add_to_list(WINDOW_MENU, "<Alt>space")
    return managed


def state_path():
    base = os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state")
    return os.path.join(base, "clippit", "nix-managed.json")


def load_state():
    try:
        with open(state_path(), encoding="utf-8") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return {}


def main():
    with open(sys.argv[1], encoding="utf-8") as f:
        plan = json.load(f)
    config_dir = os.path.join(os.environ.get("XDG_CONFIG_HOME") or os.path.expanduser("~/.config"), "clippit")
    previous = load_state()

    stale_conf = [k for k in previous.get("settings", []) if k not in plan["settings"]]
    if plan["settings"] or stale_conf:
        apply_corretor(os.path.join(config_dir, "corretor.conf"), plan["settings"], stale_conf)

    current_history = list(leaf_paths(plan["historySettings"]))
    stale_history = [k for k in previous.get("historySettings", []) if k not in current_history]
    if plan["historySettings"] or stale_history:
        changed = apply_history(os.path.join(config_dir, "config.toml"), plan["historySettings"], stale_history)
        if changed:
            # The daemon only reads config.toml when it starts.
            subprocess.run([plan["systemctl"], "--user", "try-restart", "clippit.service"], check=True)  # nosemgrep: argv list without a shell; paths come from the Nix plan

    managed = apply_gnome(plan, previous.get("gnome", {}))
    state = {"settings": list(plan["settings"]), "historySettings": current_history, "gnome": managed}
    update_file(state_path(), lambda _: json.dumps(state))


if __name__ == "__main__":
    main()
