pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland

// What GNOME Shell did for the extension, per compositor: windows (list,
// focus, move, tile), the cursor position, screenshots and global shortcuts
// that can change while running. Hyprland and Sway have full support; other
// compositors get what foreign-toplevel offers (focus, close, maximize…).
Singleton {
    id: comp

    readonly property string kind: Quickshell.env("HYPRLAND_INSTANCE_SIGNATURE") ? "hyprland"
        : Quickshell.env("SWAYSOCK") ? "sway" : Quickshell.env("NIRI_SOCKET") ? "niri" : "other"
    // Hyprland 0.53+ reads a Lua config (hl.*); older ones, hyprlang.
    property bool hyprLua: true
    property var binds: ({})

    Component.onCompleted: {
        if (kind === "hyprland")
            Sys.run(["hyprctl", "eval", "return 1"], null, 3).then(out => comp.hyprLua = !/unknown request/i.test(out),
                () => comp.hyprLua = false);
    }

    // Reloading the config (rebuild, edit, `hyprctl reload`) wipes the binds
    // made at runtime: put them back.
    Connections {
        target: comp.kind === "hyprland" ? Hyprland : null
        function onRawEvent(event) {
            if (event.name === "configreloaded")
                rebindTimer.restart();
        }
    }

    Timer {
        id: rebindTimer
        interval: 500
        onTriggered: comp.rebind()
    }

    function rebind() {
        const keys = Object.keys(binds);
        binds = {};
        keys.forEach(k => setKey(k, true));
    }

    // ---- Global shortcuts -------------------------------------------------
    // GTK accelerator ("<Control><Shift>e") → compositor syntax.
    function parseAccel(accel) {
        const mods = [];
        const re = /<([^>]+)>/g;
        let m;
        while ((m = re.exec(accel)) !== null)
            mods.push(m[1].toLowerCase());
        const key = accel.replace(/<[^>]+>/g, "");
        return {mods, key};
    }

    function hyprAccel(accel) {
        const a = parseAccel(accel);
        const names = {alt: "ALT", control: "CTRL", ctrl: "CTRL", primary: "CTRL", shift: "SHIFT", "super": "SUPER", meta: "SUPER"};
        return a.mods.map(x => names[x] || x.toUpperCase()).concat([a.key]).join(" + ");
    }

    function swayAccel(accel) {
        const a = parseAccel(accel);
        const names = {alt: "Mod1", control: "Ctrl", ctrl: "Ctrl", primary: "Ctrl", shift: "Shift", "super": "Mod4", meta: "Mod4"};
        return a.mods.map(x => names[x] || x).concat([a.key]).join("+");
    }

    function luaStr(s) {
        return "\"" + String(s).replace(/\\/g, "\\\\").replace(/"/g, "\\\"") + "\"";
    }

    // Binds `accel` to `clippit-shell key <accel>` (the shell calls the handler).
    function setKey(accel, on) {
        if (!accel || accel === "desligado" || !!binds[accel] === on || Quickshell.env("CLIPPIT_SEM_ATALHOS"))
            return;
        const b = Object.assign({}, binds);
        if (on)
            b[accel] = true;
        else
            delete b[accel];
        binds = b;
        const cmd = "clippit-shell key " + Sys.quote(accel);
        if (kind === "hyprland") {
            const h = hyprAccel(accel);
            if (hyprLua)
                Sys.run(["hyprctl", "eval", on ? "hl.bind(" + luaStr(h) + ", hl.dsp.exec_cmd(" + luaStr(cmd) + "))"
                    : "hl.unbind(" + luaStr(h) + ")"], null, 3).catch(() => {});
            else {
                const legacy = h.replace(/ \+ /g, " ").split(" ");
                const key = legacy.pop();
                const mods = legacy.join(" ");
                Sys.run(on ? ["hyprctl", "keyword", "bind", mods + "," + key + ",exec," + cmd]
                    : ["hyprctl", "keyword", "unbind", mods + "," + key], null, 3).catch(() => {});
            }
        } else if (kind === "sway") {
            // One argument after "--": swaymsg would read --no-repeat as its own option.
            Sys.run(["swaymsg", "--", on ? "bindsym --no-repeat " + swayAccel(accel) + " exec \"" + cmd + "\""
                : "unbindsym " + swayAccel(accel)], null, 3).catch(() => {});
        }
    }

    function clearKeys() {
        for (const k in binds)
            setKey(k, false);
    }

    // ---- Windows ------------------------------------------------------------
    // Window: {id, title, app, x, y, width, height, monitor, workspace, floating,
    //          fullscreen, maximized, minimized, pid}

    function fromHypr(c, monitors) {
        const mon = monitors.findIndex(m => m.id === c.monitor);
        return {
            id: c.address, title: c.title, app: c["class"], x: c.at[0], y: c.at[1], width: c.size[0], height: c.size[1],
            monitor: Math.max(0, mon), workspace: c.workspace.id, floating: c.floating, fullscreen: c.fullscreen > 0,
            maximized: c.fullscreen === 1, minimized: String(c.workspace.name).indexOf("special:minimized") === 0, pid: c.pid,
        };
    }

    function swayViews(node, out, ws, output) {
        if (node.type === "output")
            output = node.name;
        if (node.type === "workspace")
            ws = node.num;
        if ((node.type === "con" || node.type === "floating_con") && (node.app_id || node.window_properties) && !node.nodes.length) {
            out.push({
                id: String(node.id), title: node.name || "", app: node.app_id || (node.window_properties || {})["class"] || "",
                x: node.rect.x, y: node.rect.y, width: node.rect.width, height: node.rect.height, output, workspace: ws,
                floating: node.type === "floating_con", fullscreen: node.fullscreen_mode > 0, maximized: false,
                minimized: ws === undefined || ws < 0, pid: node.pid, focused: node.focused,
            });
        }
        (node.nodes || []).concat(node.floating_nodes || []).forEach(n => swayViews(n, out, ws, output));
        return out;
    }

    function monitors() {
        if (kind === "hyprland")
            return Sys.run(["hyprctl", "-j", "monitors"], null, 3).then(t => JSON.parse(t).map(m => ({
                id: m.id, name: m.name, x: m.x, y: m.y, width: Math.round(m.width / m.scale), height: Math.round(m.height / m.scale),
                reserved: m.reserved, focused: m.focused,
            })));
        if (kind === "sway")
            return Sys.run(["swaymsg", "-t", "get_outputs", "-r"], null, 3).then(t => JSON.parse(t).filter(o => o.active).map((o, i) => ({
                id: i, name: o.name, x: o.rect.x, y: o.rect.y, width: o.rect.width, height: o.rect.height, reserved: [0, 0, 0, 0],
                focused: o.focused,
            })));
        return Promise.resolve(Quickshell.screens.map((s, i) => ({id: i, name: s.name, x: s.x, y: s.y, width: s.width, height: s.height,
            reserved: [0, 0, 0, 0], focused: i === 0})));
    }

    function windows() {
        if (kind === "hyprland")
            return Promise.all([Sys.run(["hyprctl", "-j", "clients"], null, 3), Sys.run(["hyprctl", "-j", "monitors"], null, 3)])
                .then(r => {
                    const mons = JSON.parse(r[1]);
                    return JSON.parse(r[0]).filter(c => c.mapped).map(c => fromHypr(c, mons));
                });
        if (kind === "sway")
            return Promise.all([Sys.run(["swaymsg", "-t", "get_tree", "-r"], null, 3), monitors()]).then(r => {
                const mons = r[1];
                return swayViews(JSON.parse(r[0]), []).map(w => Object.assign(w, {monitor: Math.max(0, mons.findIndex(m => m.name === w.output))}));
            });
        return Promise.resolve(ToplevelManager.toplevels.values.map((t, i) => ({
            id: "tl" + i, title: t.title, app: t.appId, monitor: 0, workspace: 1, maximized: t.maximized, minimized: t.minimized,
            fullscreen: t.fullscreen, toplevel: t,
        })));
    }

    function focused() {
        if (kind === "hyprland")
            return Promise.all([Sys.run(["hyprctl", "-j", "activewindow"], null, 3), Sys.run(["hyprctl", "-j", "monitors"], null, 3)])
                .then(r => {
                    const c = JSON.parse(r[0]);
                    return c && c.address ? fromHypr(c, JSON.parse(r[1])) : null;
                }).catch(() => null);
        if (kind === "sway")
            return windows().then(ws => ws.find(w => w.focused) || null).catch(() => null);
        const t = ToplevelManager.activeToplevel;
        return Promise.resolve(t ? {id: "active", title: t.title, app: t.appId, monitor: 0, toplevel: t} : null);
    }

    function cursorPos() {
        if (kind === "hyprland")
            return Sys.run(["hyprctl", "-j", "cursorpos"], null, 3).then(t => JSON.parse(t)).catch(() => null);
        return Promise.resolve(null);
    }

    function toplevelFor(w) {
        if (w.toplevel)
            return w.toplevel;
        const list = ToplevelManager.toplevels.values;
        return list.find(t => t.appId === w.app && t.title === w.title) || list.find(t => t.appId === w.app) || null;
    }

    function hypr(lua, legacy) {
        return hyprLua ? Sys.run(["hyprctl", "dispatch", lua], null, 3) : Sys.run(["hyprctl", "dispatch"].concat(legacy), null, 3);
    }

    function sway(w, cmd) {
        return Sys.run(["swaymsg", "[con_id=" + w.id + "] " + cmd], null, 3);
    }

    function activate(w) {
        if (kind === "hyprland")
            return hypr("hl.dsp.focus({window=" + luaStr("address:" + w.id) + "})", ["focuswindow", "address:" + w.id]);
        if (kind === "sway")
            return sway(w, "focus");
        const t = toplevelFor(w);
        if (t)
            t.activate();
        return Promise.resolve();
    }

    // op: left right max restore center monitor full min close focus area_N
    function winOp(w, op) {
        if (!w)
            return Promise.resolve();
        return monitors().then(mons => {
            const mon = mons[w.monitor] || mons[0];
            const r = mon.reserved || [0, 0, 0, 0];
            const wa = {x: mon.x + r[0], y: mon.y + r[1], width: mon.width - r[0] - r[2], height: mon.height - r[1] - r[3]};
            const half = Math.floor(wa.width / 2);
            const area = /^area_(\d+)$/.exec(op);
            if (kind === "hyprland") {
                const win = "window=" + luaStr("address:" + w.id);
                const addr = "address:" + w.id;
                const place = (x, y, width, height) => hypr(
                    "hl.dsp.window.float({" + win + ", action=\"enable\"})", ["setfloating", addr])
                    .then(() => hypr("hl.dsp.window.resize({" + win + ", x=" + width + ", y=" + height + ", exact=true})",
                        ["resizewindowpixel", "exact " + width + " " + height + "," + addr]))
                    .then(() => hypr("hl.dsp.window.move({" + win + ", x=" + x + ", y=" + y + ", exact=true})",
                        ["movewindowpixel", "exact " + x + " " + y + "," + addr]));
                if (area)
                    return hypr("hl.dsp.window.move({" + win + ", workspace=" + area[1] + ", silent=true})",
                        ["movetoworkspacesilent", area[1] + "," + addr]);
                switch (op) {
                case "left":
                    return place(wa.x, wa.y, half, wa.height);
                case "right":
                    return place(wa.x + half, wa.y, half, wa.height);
                case "center":
                    return place(wa.x + Math.round((wa.width - w.width) / 2), wa.y + Math.round((wa.height - w.height) / 2), w.width, w.height);
                case "max":
                    return hypr("hl.dsp.window.fullscreen({" + win + ", mode=\"maximized\", action=\"set\"})", ["fullscreenstate", "1 -1," + addr]);
                case "restore":
                    return hypr("hl.dsp.window.fullscreen({" + win + ", mode=\"maximized\", action=\"unset\"})", ["fullscreenstate", "0 -1," + addr])
                        .then(() => w.floating ? null : hypr("hl.dsp.window.float({" + win + ", action=\"disable\"})", ["settiled", addr]));
                case "full":
                    return hypr("hl.dsp.window.fullscreen({" + win + ", mode=\"fullscreen\"})", ["fullscreen", "0," + addr]);
                case "monitor":
                    return hypr("hl.dsp.window.move({" + win + ", monitor=\"+1\"})", ["movewindow", "mon:+1," + addr]);
                case "min":
                    return hypr("hl.dsp.window.move({" + win + ", workspace=\"special:minimized\", silent=true})",
                        ["movetoworkspacesilent", "special:minimized," + addr]);
                case "close":
                    return hypr("hl.dsp.window.close({" + win + "})", ["closewindow", addr]);
                case "focus":
                    return activate(w);
                }
                return null;
            }
            if (kind === "sway") {
                if (area)
                    return sway(w, "move container to workspace number " + area[1]);
                const place = (x, y, width, height) => sway(w, "floating enable, resize set " + width + " " + height + ", move position " + x + " " + y);
                switch (op) {
                case "left":
                    return place(wa.x, wa.y, half, wa.height);
                case "right":
                    return place(wa.x + half, wa.y, half, wa.height);
                case "center":
                    return sway(w, "floating enable, move position center");
                case "max":
                    return place(wa.x, wa.y, wa.width, wa.height);
                case "restore":
                    return sway(w, "fullscreen disable, floating disable");
                case "full":
                    return sway(w, "fullscreen toggle");
                case "monitor":
                    return sway(w, "move container to output right");
                case "min":
                    return sway(w, "move scratchpad");
                case "close":
                    return sway(w, "kill");
                case "focus":
                    return sway(w, "focus");
                }
                return null;
            }
            const t = toplevelFor(w);
            if (!t)
                return null;
            switch (op) {
            case "max":
                t.maximized = true;
                break;
            case "restore":
                t.maximized = false;
                t.fullscreen = false;
                break;
            case "full":
                t.fullscreen = !t.fullscreen;
                break;
            case "min":
                t.minimized = true;
                break;
            case "close":
                t.close();
                break;
            case "focus":
                t.activate();
                break;
            }
            return null;
        });
    }

    // Exits the session (each compositor has its own way).
    function logout() {
        if (kind === "hyprland")
            return hypr("hl.dsp.exit()", ["exit"]);
        if (kind === "sway")
            return Sys.run(["swaymsg", "exit"], null, 3);
        return Sys.sh("loginctl terminate-session \"$XDG_SESSION_ID\"");
    }

    // Screenshot of a rectangle (grim). rect = {x, y, width, height}.
    function shoot(rect, path) {
        const argv = ["grim"];
        if (rect)
            argv.push("-g", Math.round(rect.x) + "," + Math.round(rect.y) + " " + Math.round(rect.width) + "x" + Math.round(rect.height));
        argv.push(path);
        return Sys.mkdir(path.replace(/\/[^/]*$/, "")).then(() => Sys.run(argv, null, 15));
    }
}
