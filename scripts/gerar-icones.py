#!/usr/bin/env python3
"""Gera os ícones do lançador (Lucide, licença ISC) como SVG já colorido.

    python3 scripts/gerar-icones.py [pasta do lucide-react/dist/esm/icons]

St (GNOME Shell) não recolore ícones de traço vindos de arquivo, então cada
ícone sai em algumas cores fixas: branco (dentro dos quadradinhos coloridos)
e cinza (ícone solto). Saída: gnome-extension/clippit-corretor@clippit/icons/.
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "gnome-extension", "clippit-corretor@clippit", "icons")
SRC = sys.argv[1] if len(sys.argv) > 1 else \
    "/usr/share/antigravity/resources/app/node_modules/lucide-react/dist/esm/icons"

NAMES = """clipboard-list sparkles gamepad-2 settings sliders-horizontal lock moon rotate-cw power log-out trash-2
spell-check messages-square text-cursor-input smile wand-sparkles wifi bluetooth sun-moon bell-off volume-x volume-2
volume-1 sun alarm-clock panel-left panel-right maximize-2 minimize-2 square-dashed monitor fullscreen
arrow-down-to-line square-x globe youtube github book-open map-pin languages calculator pencil plus file-text pin
eraser case-upper case-lower wrap-text braces link list reply
type image check triangle-alert circle-help search clock app-window folder file terminal camera mic copy
history chevron-left""".split()

COLORS = {"w": "#ffffff", "g": "#a1a1aa", "ok": "#4ade80", "err": "#ff6b6b", "acc": "#4c9dff"}
# Só alguns precisam das cores de estado.
EXTRA = {"check": ["ok"], "triangle-alert": ["err"], "circle-help": ["acc"], "pin": ["acc"]}


def node_list(name):
    js = open(os.path.join(SRC, f"{name}.js"), encoding="utf-8").read()
    alias = re.search(r"export \{ default \} from '\./([\w-]+)\.js'", js)
    if alias and "__iconNode" not in js:
        return node_list(alias.group(1))  # nome antigo que aponta para outro
    body = re.search(r"const __iconNode = (\[.*?\]);\n", js, re.S).group(1)
    body = re.sub(r"(\w+):", r'"\1":', body)          # chaves sem aspas → JSON
    body = re.sub(r",\s*([\]}])", r"\1", body)         # vírgula sobrando
    return json.loads(body)


def svg(nodes, color):
    parts = []
    for tag, attrs in nodes:
        a = " ".join(f'{k}="{v}"' for k, v in attrs.items() if k != "key")
        parts.append(f"<{tag} {a}/>")
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" '
            f'stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">'
            + "".join(parts) + "</svg>\n")


def main():
    os.makedirs(OUT, exist_ok=True)
    n = 0
    for name in NAMES:
        nodes = node_list(name)
        for c in ["w", "g", *EXTRA.get(name, [])]:
            with open(os.path.join(OUT, f"{name}-{c}.svg"), "w", encoding="utf-8") as f:
                f.write(svg(nodes, COLORS[c]))
            n += 1
    with open(os.path.join(OUT, "LICENSE-lucide.txt"), "w", encoding="utf-8") as f:
        f.write("Ícones: Lucide (https://lucide.dev), licença ISC.\n"
                "Copyright (c) for portions of Lucide are held by Cole Bemis 2013-2022 as part of Feather (MIT). "
                "All other copyright (c) for Lucide are held by Lucide Contributors 2022.\n\n"
                "Permission to use, copy, modify, and/or distribute this software for any purpose with or without "
                "fee is hereby granted, provided that the above copyright notice and this permission notice appear "
                "in all copies.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH "
                "REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.\n")
    print(f"{n} ícones em {OUT}")


if __name__ == "__main__":
    main()
