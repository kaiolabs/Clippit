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

    function set(owner, accel, on, fn) {
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
        Compositor.setKey(accel, list.length > 0);
    }

    function clear(owner) {
        for (const accel in handlers)
            set(owner, accel, false);
    }

    function fire(accel) {
        const list = handlers[accel] || [];
        const h = list[list.length - 1];
        if (h)
            h.fn();
    }
}
