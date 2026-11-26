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
