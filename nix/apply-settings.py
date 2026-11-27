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


def write_if_changed(path, text):
    try:
        with open(path, encoding="utf-8") as f:
            if f.read() == text:
                return False
    except FileNotFoundError:
        pass
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.clippit-tmp"
    with open(temporary, "w", encoding="utf-8") as f:
        f.write(text)
    os.replace(temporary, path)
    return True


def conf_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return ", ".join(str(v) for v in value)
    return str(value)


def apply_corretor(path, settings):
    try:
        with open(path, encoding="utf-8") as f:
            lines = f.read().splitlines()
    except FileNotFoundError:
        lines = []
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
    return write_if_changed(path, "\n".join(lines) + "\n")


def deep_merge(base, over):
    for key, value in over.items():
        if isinstance(value, dict) and isinstance(base.get(key), dict):
            deep_merge(base[key], value)
        else:
            base[key] = value
    return base


def apply_history(path, settings):
    try:
        with open(path, "rb") as f:
            current = tomllib.load(f)
    except FileNotFoundError:
        current = {}
    except tomllib.TOMLDecodeError:
        print(f"clippit: {path} is not valid TOML, leaving it as is", file=sys.stderr)
        return False
    return write_if_changed(path, tomli_w.dumps(deep_merge(current, settings)))


class Dconf:
    def __init__(self, exe):
        self.exe = exe

    def read(self, key):
        p = subprocess.run([self.exe, "read", key], capture_output=True, text=True)
        if p.returncode != 0:
            raise RuntimeError(f"dconf read {key} failed: {p.stderr.strip()}")
        return p.stdout.strip()

    def write(self, key, value):
        subprocess.run([self.exe, "write", key, value], check=True)

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


def gvariant_string(text):
    return "'" + text.replace("\\", "\\\\").replace("'", "\\'") + "'"


def apply_gnome(plan):
    dconf = Dconf(plan["dconf"])
    if plan["extension"]:
        dconf.add_to_list("/org/gnome/shell/enabled-extensions", plan["extension"])
    shortcut = plan["historyShortcut"]
    if shortcut:
        dconf.add_to_list(f"{KEYBINDINGS}", CLIPPIT_KEYBINDING)
        dconf.write(f"{CLIPPIT_KEYBINDING}name", "'Clippit'")
        dconf.write(f"{CLIPPIT_KEYBINDING}command", gvariant_string(shortcut["command"]))
        dconf.write(f"{CLIPPIT_KEYBINDING}binding", gvariant_string(shortcut["binding"]))
    if plan["freeAltSpace"]:
        key = "/org/gnome/desktop/wm/keybindings/activate-window-menu"
        bindings = dconf.read_list(key)
        if "<Alt>space" in bindings:
            remaining = [b for b in bindings if b != "<Alt>space"]
            dconf.write(key, str(remaining) if remaining else "@as []")


def main():
    with open(sys.argv[1], encoding="utf-8") as f:
        plan = json.load(f)
    config_dir = os.path.join(os.environ.get("XDG_CONFIG_HOME") or os.path.expanduser("~/.config"), "clippit")

    if plan["settings"]:
        apply_corretor(os.path.join(config_dir, "corretor.conf"), plan["settings"])
    if plan["historySettings"]:
        if apply_history(os.path.join(config_dir, "config.toml"), plan["historySettings"]):
            # The daemon only reads config.toml when it starts.
            subprocess.run([plan["systemctl"], "--user", "try-restart", "clippit.service"], check=True)
    apply_gnome(plan)


if __name__ == "__main__":
    main()
