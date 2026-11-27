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
        sugestoes: "<Alt>o", aplicar: "<Alt>s", ajuste: "<Alt>f",
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

    function keyOf(id) {
        return value("atalho_" + id, shortcuts[id]) || shortcuts[id];
    }

    // "<Control><Shift>e" → "Ctrl Shift E": what to print in hints, so a rebound key shows up right.
    function label(id, sep) {
        const accel = keyOf(id);
        if (accel === "desligado")
            return "";
        const names = {alt: "Alt", control: "Ctrl", ctrl: "Ctrl", primary: "Ctrl", shift: "Shift", "super": "Super", meta: "Super"};
        const parts = [];
        const re = /<([^>]+)>/g;
        let m;
        while ((m = re.exec(accel)) !== null)
            parts.push(names[m[1].toLowerCase()] || m[1]);
        const key = accel.replace(/<[^>]+>/g, "");
        parts.push(key.length === 1 ? key.toUpperCase() : key === "space" ? "Espaço" : key);
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
