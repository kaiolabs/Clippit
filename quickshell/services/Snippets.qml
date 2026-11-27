pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import "../lib/busca.js" as Busca

// Ready-made texts called by ;trigger (~/.config/clippit/snippets.json).
Singleton {
    id: sn

    readonly property string path: Sys.configDir + "/snippets.json"
    property var items: []

    function save(list) {
        items = list;
        return Sys.write(path, JSON.stringify(list, null, 2));
    }

    // {data} {hora} {area} (what is copied)
    function expand(text) {
        const now = Busca.today();
        let out = text.split("{data}").join(now.data).split("{hora}").join(now.hora);
        if (out.indexOf("{area}") < 0)
            return Promise.resolve(out);
        return Sys.clipboard(false).then(cb => out.split("{area}").join(cb));
    }

    FileView {
        path: sn.path
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            try {
                const list = JSON.parse(text());
                sn.items = Array.isArray(list) ? list.filter(x => x.gatilho && x.texto) : [];
            } catch (e) {
                sn.items = [];
            }
        }
        onLoadFailed: sn.items = []
    }
}
