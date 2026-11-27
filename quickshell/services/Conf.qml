pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// ~/.config/clippit/corretor.conf (key = value), re-read when it changes.
Singleton {
    id: conf

    property var values: ({})
    // Default shortcuts (GTK accelerator format). "desligado" turns one off.
    readonly property var shortcuts: ({
        lancador: "<Alt>space", ia: "<Alt>i", acoes: "<Alt>a", assistente: "<Alt>k", modo_jogo: "<Control><Shift>e",
        sugestoes: "<Alt>o", aplicar: "<Alt>s", ajuste: "<Alt>f", emoji: "<Alt>e", snippets: "<Alt>semicolon",
    })
    // Shortcuts that only work with the launcher open (the same table as INNER_SHORTCUTS in core.js).
    readonly property var innerShortcuts: ({
        fechar: "Escape", ditado: "<Alt>v", lado: "<Alt>c", nova_conversa: "<Control>n", conversas: "<Control>h",
        transformar: "Tab", fixar: "<Control>p", apagar: "<Control>Delete", ampliar: "<Control>space",
        copiar: "<Control>Return", voltar: "BackSpace", categoria: "Tab",
    })

    function value(key, fallback) {
        return values[key] !== undefined ? values[key] : fallback;
    }

    function on(key, fallback) {
        const v = String(value(key, fallback === undefined ? true : fallback)).toLowerCase();
        return ["false", "nao", "não", "0"].indexOf(v) < 0;
    }

    function aiOn(task) {
        return on("ia") && on("ia_" + task);
    }

    function aiOffNotice(what) {
        Sys.notify("Clippit", what + " está desligado. Para ligar: Configurações do Clippit → Geral → Inteligência artificial.");
    }

    function defaultOf(id) {
        return shortcuts[id] || innerShortcuts[id];
    }

    // `copiar` only means something with Enter: any other key goes back to the default.
    function keyOf(id) {
        const v = value("atalho_" + id, defaultOf(id)) || defaultOf(id);
        if (id === "copiar" && v !== "desligado" && (parseAccel(v) || {}).key !== "return")
            return defaultOf(id);
        return v;
    }

    // Modifier bits: Ctrl 1, Alt 2, Shift 4, Super 8.
    readonly property var modBits: ({control: 1, primary: 1, ctrl: 1, alt: 2, mod1: 2, shift: 4, "super": 8, mod4: 8, meta: 8})
    readonly property var keyAliases: ({iso_left_tab: "tab", kp_enter: "return", iso_enter: "return", kp_delete: "delete", enter: "return",
        backtab: "tab"})

    // "<Control><Shift>e" → {mods, key}; null if not understood (an unknown modifier would otherwise
    // read as "no modifier" and fire while typing).
    function parseAccel(accel) {
        let rest = String(accel).trim(), mods = 0, ok = true, m;
        while ((m = rest.match(/^<(\w+)>/)) !== null) {
            const bit = modBits[m[1].toLowerCase()];
            if (bit === undefined)
                ok = false;
            mods |= bit || 0;
            rest = rest.slice(m[0].length);
        }
        const key = rest.toLowerCase();
        return ok && key ? {mods, key: keyAliases[key] || key} : null;
    }

    readonly property var qtKeyNames: ({
        [Qt.Key_Escape]: "escape", [Qt.Key_Return]: "return", [Qt.Key_Enter]: "return", [Qt.Key_Tab]: "tab", [Qt.Key_Backtab]: "tab",
        [Qt.Key_Backspace]: "backspace", [Qt.Key_Delete]: "delete", [Qt.Key_Space]: "space", [Qt.Key_Semicolon]: "semicolon",
        [Qt.Key_Colon]: "colon", [Qt.Key_Period]: "period", [Qt.Key_Comma]: "comma", [Qt.Key_Slash]: "slash",
        [Qt.Key_Up]: "up", [Qt.Key_Down]: "down", [Qt.Key_Left]: "left", [Qt.Key_Right]: "right",
        [Qt.Key_PageUp]: "page_up", [Qt.Key_PageDown]: "page_down", [Qt.Key_Home]: "home", [Qt.Key_End]: "end",
    })

    function keyName(k) {
        if (qtKeyNames[k])
            return qtKeyNames[k];
        if (k >= Qt.Key_A && k <= Qt.Key_Z || k >= Qt.Key_0 && k <= Qt.Key_9)
            return String.fromCharCode(k).toLowerCase();
        if (k >= Qt.Key_F1 && k <= Qt.Key_F35)
            return "f" + (k - Qt.Key_F1 + 1);
        return "";
    }

    function heldMods(event) {
        const m = event.modifiers;
        return (m & Qt.ControlModifier ? 1 : 0) | (m & Qt.AltModifier ? 2 : 0) | (m & Qt.ShiftModifier ? 4 : 0) | (m & Qt.MetaModifier ? 8 : 0);
    }

    // Is the key pressed the shortcut `id`? Modifiers must match exactly (Ctrl+Shift+N is not Ctrl+N).
    // `ignoreShift`: Shift only reverses the direction (Tab / Shift+Tab).
    function keyIs(event, id, ignoreShift) {
        const accel = keyOf(id);
        const p = accel === "desligado" ? null : parseAccel(accel);
        if (!p)
            return false;
        let held = heldMods(event), want = p.mods;
        if (event.key === Qt.Key_Backtab || ignoreShift) {
            held &= ~4;
            want &= ~4;
        }
        return held === want && keyName(event.key) === p.key;
    }

    // Does the shortcut use Ctrl/Alt/Shift/Super? Without them it only applies to an empty field.
    function hasMods(id) {
        const p = parseAccel(keyOf(id));
        return !!p && p.mods !== 0;
    }

    // Enter with the modifiers of `id` (default Ctrl+Enter = copy instead of paste).
    function modIs(event, id) {
        const accel = keyOf(id);
        const p = accel === "desligado" ? null : parseAccel(accel);
        return !!p && p.mods !== 0 && heldMods(event) === p.mods;
    }

    // "<Control><Shift>e" → "Ctrl Shift E" (sep " ") or "Ctrl+Shift+E" (sep "+"): what hints print,
    // so a rebound key shows up right.
    function label(id, sep) {
        const accel = keyOf(id);
        if (accel === "desligado")
            return "—";
        const p = parseAccel(accel);
        if (!p)
            return accel;
        const names = {"return": "↵", "delete": "Del", space: "Espaço", escape: "Esc", backspace: "⌫", tab: "Tab", semicolon: ";",
            colon: ":", period: ".", comma: ",", slash: "/"};
        const parts = [];
        if (p.mods & 1) parts.push("Ctrl");
        if (p.mods & 4) parts.push("Shift");
        if (p.mods & 2) parts.push("Alt");
        if (p.mods & 8) parts.push("Super");
        parts.push(names[p.key] || (p.key.length === 1 ? p.key.toUpperCase() : p.key[0].toUpperCase() + p.key.slice(1)));
        return parts.join(sep || " ");
    }

    function parse(text) {
        const out = {};
        for (const line of (text || "").split("\n")) {
            const bare = line.split("#")[0];
            const i = bare.indexOf("=");
            if (i > 0)
                out[bare.slice(0, i).trim()] = bare.slice(i + 1).trim();
        }
        values = out;
    }

    FileView {
        id: file
        path: Sys.configDir + "/corretor.conf"
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: conf.parse(text())
        onLoadFailed: conf.values = {}
    }
}
