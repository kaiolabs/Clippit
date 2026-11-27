pragma Singleton

import QtQuick
import Quickshell

// Shortcuts that come and go (Esc only with a panel open, Alt+S only with
// suggestions…), like grab_accelerator in GNOME Shell. The compositor binds
// the key to `clippit-shell key <accel>`, which lands in fire().
Singleton {
    id: keys

    // accel → [{owner, fn}] (the last one registered wins)
    property var handlers: ({})

    // global: Alt+Space and the other always-on shortcuts. With
    // "atalhos_globais = false" the compositor config binds them to
    // `clippit-shell <função>` instead, and they are not bound here.
    function set(owner, accel, on, fn, global) {
        if (!accel || accel === "desligado")
            return;
        const list = (handlers[accel] || []).filter(h => h.owner !== owner);
        if (on)
            list.push({owner, fn});
        const map = Object.assign({}, handlers);
        if (list.length)
            map[accel] = list;
        else
            delete map[accel];
        handlers = map;
        Compositor.setKey(accel, list.length > 0 && !(global && !Conf.on("atalhos_globais")));
    }

    function clear(owner) {
        for (const accel in handlers)
            set(owner, accel, false);
    }

    function fire(accel) {
        if (Quickshell.env("CLIPPIT_DEBUG"))
            console.log("clippit: atalho " + accel + " → " + (handlers[accel] || []).length + " função(ões)");
        const list = handlers[accel] || [];
        const h = list[list.length - 1];
        if (h)
            h.fn();
    }
}
