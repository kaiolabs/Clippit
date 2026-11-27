#!/usr/bin/env python3
"""Checks nix/known-keys.json against docs/configuration.md.

The Home Manager module rejects keys that are not in that file. Every documented
key has to be in it; `config.toml` has more keys than the docs list, so there the
file is the full list of fields and the docs only need to be a subset.
It is also compared with the code: `corretor.conf` keys must be read by the scripts or
the extension, and a test in clippit-core compares `config.toml` keys with `Config::default()`.

    scripts/check-known-keys.py
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def doc_keys(section):
    keys = []
    for line in section.splitlines():
        if not line.startswith("| `"):
            continue
        first = line.split("|")[1]
        previous = None
        for token in re.findall(r"`([^`]+)`", first):
            if previous and "." in previous and "." not in token:
                token = previous.rsplit(".", 1)[0] + "." + token
            keys.append(token)
            previous = token
    return keys


def compare_quickshell_shortcuts(extension):
    """Quickshell's default shortcuts (Conf.qml) must match the extension's (core.js)."""
    conf = (ROOT / "quickshell/services/Conf.qml").read_text(encoding="utf-8")

    def table(text, marker, end):
        return dict(re.findall(r"(\w+):\s*[\"']([^\"']+)[\"']", text.split(marker, 1)[1].split(end, 1)[0]))

    failed = False
    for name, marker in (("shortcuts", "const SHORTCUTS = {"), ("innerShortcuts", "const INNER_SHORTCUTS = {")):
        quickshell = table(conf, f"{name}: ({{", "})")
        gnome = table(extension, marker, "};")
        for id_ in sorted(set(quickshell) | set(gnome)):
            if quickshell.get(id_) != gnome.get(id_):
                print(f"default shortcut differs: {id_}: quickshell={quickshell.get(id_)} extension={gnome.get(id_)}")
                failed = True
    return failed


def main():
    docs = (ROOT / "docs/configuration.md").read_text(encoding="utf-8")
    corretor_doc, config_doc = docs.split("## `config.toml`", 1)
    documented = {
        "corretor": set(doc_keys(corretor_doc.split("## `corretor.conf`", 1)[1])),
        "config": set(doc_keys(config_doc)),
    }
    known = json.loads((ROOT / "nix/known-keys.json").read_text(encoding="utf-8"))

    failed = False
    code = "".join(
        p.read_text(encoding="utf-8", errors="ignore")
        for p in [
            *(ROOT / "scripts").glob("clippit-*"),
            *(ROOT / "gnome-extension").rglob("*.js"),
            *(ROOT / "quickshell").rglob("*.qml"),
        ]
        if p.is_file()
    )
    extension = (ROOT / "gnome-extension/clippit-corretor@clippit/core.js").read_text(encoding="utf-8")
    shortcut_ids = []
    for table in ("const SHORTCUTS = {", "const INNER_SHORTCUTS = {"):
        shortcut_ids += re.findall(r"(\w+):\s*'[^']+'", extension.split(table, 1)[1].split("};", 1)[0])
    failed |= compare_quickshell_shortcuts(extension)
    correct = (ROOT / "scripts/clippit-correct").read_text(encoding="utf-8")
    ai_tasks = re.findall(r'^    "(\w+)":', correct.split("AI_TASKS = {", 1)[1].split("}", 1)[0], re.M)
    for key in known["corretor"]:
        if key.startswith("atalho_"):
            used = key.removeprefix("atalho_") in shortcut_ids or re.search(rf"\b{key}\b", code) is not None
        elif key.startswith(("ia_", "modelo_")) and key.split("_", 1)[1] in ai_tasks:
            used = True
        else:
            used = re.search(rf"\b{re.escape(key)}\b", code) is not None
        if not used:
            print(f"in nix/known-keys.json but no code reads it (corretor): {key}")
            failed = True
    for kind, keys in documented.items():
        for key in sorted(keys - set(known[kind])):
            print(f"documented but missing from nix/known-keys.json ({kind}): {key}")
            failed = True
        if kind == "corretor":
            for key in sorted(set(known[kind]) - keys):
                print(f"in nix/known-keys.json but not documented ({kind}): {key}")
                failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
