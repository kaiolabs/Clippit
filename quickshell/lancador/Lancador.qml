import QtQuick
import Quickshell
import "../services"
import "../lib/busca.js" as Busca
import "../lib/compat.js" as Compat

// Launcher (Alt+Space), the same as the GNOME extension: a search bar in the
// middle of the screen that opens apps, switches windows, does math and
// conversions, searches the Clippit history and files, runs system commands,
// searches the web and asks the local AI. Screens: root | historico | ia |
// conversas | texto | emoji. This file is the logic; LancadorJanela.qml draws it.
Scope {
    id: l

    required property var corretor
    required property var assistente

    readonly property int launchRows: 40
    readonly property int keepTextSeconds: 300
    readonly property string usageFile: Sys.dataDir + "/lancador.json"
    readonly property string quicklinksFile: Sys.configDir + "/atalhos-web.json"
    readonly property string convDir: Sys.dataDir + "/conversas"
    readonly property string statusFile: Sys.runtimeDir + "/clippit-correct/agente-status"

    // ---- State --------------------------------------------------------------
    property bool open: false
    property bool grabbed: false
    property string view: "root"
    property string query: ""
    property int cursorPos: -1 // the entry's cursor (for @ and dictation)
    property var shown: []
    property int sel: 0
    property bool zoom: false
    property bool consumed: false
    property var saved: null
    property var prevWin: null
    property var monitor: null
    property var usage: ({})
    property var quicklinks: Busca.WEB
    property var asyncHistory: []
    property var asyncFiles: []
    property var winList: []
    property var ctl: ({dark: false, dnd: null, dndKind: "", bt: null, brightness: false})
    property int searchGen: 0

    // History screen
    property var cbTx: null
    // Text actions (Alt+A)
    property string selText: ""
    property string selFrom: ""
    property var tx: null
    // Emoji
    property var emojiData: null
    property bool emojiLoading: false
    property var emojiPrefs: ({fixados: ["❤️", "👋", "👍", "🚀", "🎉", "🙏"], recentes: []})
    property string emojiCat: ""
    property int emojiSel: 0
    // AI chat
    property var chat: []
    property string chatId: ""
    property double chatTime: 0
    property int askGen: 0
    property var attach: []
    property var mention: null
    property int mentionGen: 0
    property var fix: null
    property string fixState: "" // checking | ok | error | fix | rec | ""
    property string fixMsg: ""
    property var rec: null
    property var convs: null
    // Side panel (Alt+C)
    property bool side: false
    property var sideItems: []
    property bool sideLoading: false
    property int sideSel: 0
    property var sideTx: null
    property string sideFoot: ""
    property double sideToggled: 0

    signal setEntry(string text, int cursor)
    signal insertEntry(string text)
    signal focusEntry

    readonly property bool isHistory: view === "historico" || /^cb(\s|$)/i.test(query.trim())
    readonly property bool busyChat: chat.some(m => m.pending)
    readonly property var apps: DesktopEntries.applications.values.filter(a => !a.noDisplay)

    Component.onCompleted: {
        Sys.readJson(usageFile, {}).then(u => l.usage = u);
        Sys.readJson(Sys.dataDir + "/emoji.json", null).then(p => {
            if (p)
                l.emojiPrefs = p;
        });
        bindGlobal();
        checkReminders();
        watchGames();
    }

    Connections {
        target: Conf
        function onValuesChanged() {
            Atalhos.clear(l);
            l.bindGlobal();
            l.syncKeys();
        }
    }

    function bindGlobal() {
        Atalhos.set(l, Conf.keyOf("lancador"), true, () => {
            if (l.open && !l.grabbed)
                l.wake();
            else if (l.open)
                l.close();
            else
                l.openLauncher("root");
        }, true);
        Atalhos.set(l, Conf.keyOf("ia"), true, () => l.toggleView("ia"), true);
        Atalhos.set(l, Conf.keyOf("acoes"), true, () => l.openTextActions(false), true);
        Atalhos.set(l, Conf.keyOf("emoji"), true, () => l.toggleView("emoji"), true);
        Atalhos.set(l, Conf.keyOf("snippets"), true, () => l.toggleSnippets(), true);
        Atalhos.set(l, Conf.value("atalho_historico", "<Alt>c"), true, () => l.toggleView("historico"), true);
    }

    // Keys that only exist while the launcher is open come from the window
    // itself (it has the keyboard); nothing to bind here.
    function syncKeys() {
    }

    // ---- Open, close, release -----------------------------------------------

    function toggleView(v) {
        if (v === "ia" && !Conf.aiOn("chat")) {
            Conf.aiOffNotice("O chat com a IA");
            return;
        }
        if (open && !grabbed) {
            wake();
            if (v === "historico" && view !== "historico")
                toggleSide();
            else if (v !== view && v !== "root")
                enterView(v);
            return;
        }
        if (v === "historico" && open && view !== "historico") {
            toggleSide();
            return;
        }
        if (open && view === v) {
            close();
            return;
        }
        if (!open)
            openLauncher(v);
        else if (v !== "root")
            enterView(v);
    }

    // The shortcut for snippets: opens the launcher already on ";" (the trigger list).
    function toggleSnippets() {
        const showing = open && view === "root" && query.startsWith(";");
        if (open && !grabbed) {
            wake();
            if (showing)
                return;
        } else if (showing) {
            close();
            return;
        }
        if (!open) {
            openLauncher("root", ";");
            return;
        }
        if (view !== "root")
            enterView("root");
        setEntry(";", -1);
    }

    function openLauncher(v, text) {
        if (open)
            return;
        const s = saved;
        saved = null;
        const fresh = s && (Date.now() - s.time) / 1000 < keepTextSeconds;
        sel = 0;
        view = "root";
        zoom = false;
        consumed = false;
        asyncHistory = [];
        asyncFiles = [];
        cbTx = null;
        tx = null;
        side = false;
        Compat.co(function* () {
            l.prevWin = yield Compositor.focused();
            const mons = yield Compositor.monitors();
            const cur = yield Compositor.cursorPos();
            let mon = l.prevWin ? mons[l.prevWin.monitor] : null;
            if (!mon && cur)
                mon = mons.find(m => cur.x >= m.x && cur.x < m.x + m.width && cur.y >= m.y && cur.y < m.y + m.height);
            l.monitor = mon || mons.find(m => m.focused) || mons[0];
            l.open = true;
            l.grabbed = true;
            l.corretor.setSuspended(true);
            l.loadQuicklinks();
            l.loadControls();
            Compositor.windows().then(ws => {
                l.winList = ws;
                l.refresh(true);
            });
            if (text) {
                l.setEntry(text, -1);
            } else if (fresh) {
                if (s.view !== "root")
                    l.enterView(s.view);
                l.setEntry(s.text, -1);
            } else {
                l.setEntry("", 0);
                l.refresh(false);
            }
            if (v !== "root" && l.view !== v)
                l.enterView(v);
            l.focusEntry();
        });
    }

    function close() {
        if (open && !consumed && (query || (view === "ia" && chat.length)))
            saved = {time: Date.now(), view, text: query};
        else if (consumed)
            saved = null;
        if (rec)
            stopDictation(false);
        fix = null;
        fixState = "";
        mention = null;
        closeSide();
        if (open) {
            open = false;
            grabbed = false;
            corretor.setSuspended(false);
        }
        searchGen++;
        askGen++;
    }

    // Click outside: lets go of the keyboard without closing (it stays on
    // screen, half transparent). A click on it, or the shortcut, wakes it.
    function release() {
        if (!grabbed)
            return;
        grabbed = false;
        corretor.setSuspended(false);
    }

    function wake() {
        if (!open || grabbed)
            return;
        Compositor.focused().then(w => {
            if (w)
                l.prevWin = w;
        });
        grabbed = true;
        corretor.setSuspended(true);
        focusEntry();
    }

    function enterView(v) {
        view = v;
        zoom = false;
        if (v === "emoji") {
            emojiSel = 0;
            loadEmoji();
        }
        sel = 0;
        if (query)
            setEntry("", 0);
        else
            onQuery("", 0);
    }

    function goBack() {
        if (view === "texto" && tx) {
            tx = null;
            refresh(false);
        } else if (cbTx) {
            cbTx = null;
            refresh(false);
        } else {
            enterView("root");
        }
        focusEntry();
    }

    readonly property string hint: ({
        root: "Buscar apps, comandos e mais…", historico: "Filtrar o que você copiou…",
        ia: "Pergunte algo… (@ anexa app, arquivo ou print · Alt+V fala)", conversas: "Procurar nas conversas…",
        texto: "O que fazer com o texto? Escolha abaixo ou escreva…", emoji: "Buscar emoji e símbolos…",
    })[view] || ""

    readonly property string modeLabel: isHistory ? "Histórico" : ({ia: "IA local", conversas: "Conversas", texto: "Texto selecionado"})[view] || ""

    // ---- Usage (most used first) --------------------------------------------

    function bump(key) {
        const u = Object.assign({}, usage);
        u[key] = (u[key] || 0) + 1;
        usage = u;
        Sys.writeJson(usageFile, u);
    }

    function usageBoost(key) {
        return Math.min(30, (usage[key] || 0) * 3);
    }

    function loadQuicklinks() {
        Sys.readJson(quicklinksFile, []).then(own => {
            l.quicklinks = Busca.WEB.concat((own || []).map(x => ({key: Busca.fold(x.atalho), name: x.nome, url: x.url,
                icon: "web-browser-symbolic"})));
        });
    }

    // ---- Search -------------------------------------------------------------

    function onQuery(text, cursor) {
        query = text;
        cursorPos = cursor;
        // ":" in the root search opens the emoji picker (the rest is the filter).
        if (view === "root" && (text.startsWith(":") || /^emoji\s/i.test(text))) {
            const rest = text.startsWith(":") ? text.slice(1) : text.replace(/^emoji\s+/i, "");
            enterView("emoji");
            setEntry(rest, -1);
            return;
        }
        updateMention();
        scheduleFix();
        sel = 0;
        emojiSel = 0;
        refresh(false);
        debounce.restart();
    }

    Timer {
        id: debounce
        interval: 220
        onTriggered: l.searchAsync(l.query.trim())
    }

    function searchAsync(q) {
        if (view === "ia" || view === "emoji" || !open)
            return;
        const g = ++searchGen;
        const cbMode = isHistory;
        const hq = view === "historico" ? q : q.replace(/^cb\s*/i, "");
        if (cbMode || hq.length >= 3)
            Sys.run(Sys.tool("clippit-busca").concat(["historico", hq]), null, 5).then(o => {
                if (g === l.searchGen && l.open) {
                    l.asyncHistory = JSON.parse(o);
                    l.refresh(true);
                }
            }).catch(e => console.log("clippit: busca falhou: " + e.message));
        if (!cbMode && view === "root" && q.length >= 3 && !q.startsWith("?"))
            Sys.run(Sys.tool("clippit-busca").concat(["arquivos", q]), null, 6).then(o => {
                if (g === l.searchGen && l.open) {
                    l.asyncFiles = JSON.parse(o);
                    l.refresh(true);
                }
            }).catch(e => console.log("clippit: busca falhou: " + e.message));
    }

    function refresh(keepSel) {
        if (!open)
            return;
        const q = query.trim();
        const old = shown[sel];
        const items = collect(q);
        let s = sel;
        if (keepSel && old) {
            const i = items.findIndex(it => it.id === old.id);
            s = i >= 0 ? i : 0;
        }
        shown = items;
        sel = Math.min(s, Math.max(0, items.length - 1));
    }

    function collect(q) {
        const items = [];
        const ql = Busca.fold(q);
        if ((view === "ia" || q.startsWith("?")) && !Conf.aiOn("chat"))
            return [{id: "ia-off", icon: "starred-symbolic", score: 1000, title: "A IA está desligada",
                subtitle: "Configurações do Clippit → Geral → Inteligência artificial", keepOpen: true, run: () => {}}];
        if (view === "ia" || q.startsWith("?")) {
            const question = view === "ia" ? q : q.slice(1).trim();
            items.push({id: "ia", section: "IA local", icon: "starred-symbolic", score: 1000,
                title: question ? "Perguntar à IA: " + question : "Pergunte algo à IA (roda no seu computador)",
                subtitle: "Enter envia · a resposta aparece aqui", hint: "Perguntar", keepOpen: true,
                run: () => {
                    l.enterView("ia");
                    if (question)
                        l.send(question, {});
                }});
            return items;
        }
        if (view === "conversas")
            return convItems(q);
        if (view === "emoji")
            return [];
        if (view === "texto")
            return textActionItems(q);
        if (q.startsWith(";"))
            return snippetItems(q);
        if (isHistory && cbTx)
            return cbTxItems(q);
        if (isHistory) {
            for (const h of asyncHistory)
                items.push(historyItem(h, 500));
            if (!items.length)
                items.push({id: "cb-vazio", title: "Nada no histórico com esse texto", subtitle: "", icon: "edit-paste-symbolic", score: 0,
                    run: () => {}});
            return items;
        }
        if (!q) {
            const used = apps.map(a => ({a, n: usage["app:" + a.id] || 0})).filter(x => x.n > 0)
                .sort((x, y) => y.n - x.n).slice(0, 5);
            for (const u of used)
                items.push(appItem(u.a, 100, "Mais usados"));
            const cmds = commands();
            for (const id of ["cmd:historico", "cmd:ia", "cmd:game", "cmd:clippit", "cmd:lock"]) {
                const cmd = cmds.find(c => c.id === id);
                if (cmd)
                    items.push(Object.assign({}, cmd, {section: "Sugestões", score: 0}));
            }
            return items;
        }
        quickNumberItems(q).forEach(it => items.push(it));
        const conv = Busca.convert(q);
        const calc = conv ? null : Busca.calculate(q);
        if (conv || calc !== null) {
            const value = conv ? conv.value : calc;
            const copy = (Busca.formatNum(value) || "").replace(/\./g, "");
            items.push({id: "calc", section: "Calculadora", icon: "accessories-calculator-symbolic", score: 900,
                title: "= " + Busca.formatNum(value) + (conv ? " " + conv.unit : ""), subtitle: q + " · Enter copia o resultado",
                hint: "Copiar", run: () => {
                    Sys.copy(copy);
                    Sys.notify("Clippit", "Copiado: " + copy);
                }});
        }
        const pm = q.match(/^(\S+)\s+(.+)$/);
        const prefixed = pm && quicklinks.find(w => w.key === Busca.fold(pm[1]));
        if (prefixed)
            items.push(webItem(prefixed, pm[2], 950));
        for (const a of apps) {
            const s = Math.max(Busca.matchScore(q, a.name), Busca.matchScore(q, a.genericName || "") * 0.7,
                Math.max.apply(null, (a.keywords || []).map(k => Busca.matchScore(q, k) * 0.6).concat([0])),
                Busca.matchScore(q, (a.command || [])[0] || "") * 0.6);
            if (s >= 40)
                items.push(appItem(a, s + 200 + usageBoost("app:" + a.id), "Apps"));
        }
        for (const w of winList) {
            const s = Math.max(Busca.matchScore(q, w.title || ""), Busca.matchScore(q, appName(w.app)) * 0.9);
            if (s >= 40)
                items.push({id: "win:" + w.id, section: "Janelas abertas", appIcon: appIconFor(w.app), icon: "focus-windows-symbolic",
                    score: s + 150, title: w.title || appName(w.app) || "Janela", subtitle: appName(w.app), hint: "Ir para",
                    run: () => Compositor.activate(w)});
        }
        for (const cmd of commands()) {
            const s = Math.max(Busca.matchScore(q, cmd.title), Math.max.apply(null, cmd.words.map(w => Busca.matchScore(q, w) * 0.9)));
            if (s >= 40)
                items.push(Object.assign({}, cmd, {section: "Comandos", score: s + 180 + usageBoost(cmd.key)}));
        }
        asyncHistory.slice(0, 3).forEach(h => items.push(historyItem(h, 120)));
        asyncFiles.slice(0, 5).forEach(f => items.push({
            id: "file:" + f.uri, section: "Arquivos", icon: "text-x-generic-symbolic", score: 100 + Busca.matchScore(q, f.nome) * 0.5,
            title: f.nome, subtitle: f.pasta, hint: "Abrir",
            run: () => Sys.openUri(f.uri),
            alt: () => Sys.run(["dbus-send", "--session", "--dest=org.freedesktop.FileManager1", "--type=method_call",
                "/org/freedesktop/FileManager1", "org.freedesktop.FileManager1.ShowItems", "array:string:" + f.uri, "string:"]).catch(() => {}),
        }));
        if (/^(https?:\/\/)?[\w-]+(\.[\w-]+)+(\/\S*)?$/.test(q) && !/^\d+([.,]\d+)*$/.test(q)) {
            const url = q.startsWith("http") ? q : "https://" + q;
            items.push({id: "url", section: "Web", icon: "web-browser-symbolic", score: 850, title: "Abrir " + url,
                subtitle: "No navegador padrão", hint: "Abrir", run: () => Sys.openUri(url)});
        }
        items.sort((x, y) => y.score - x.score);
        const best = items.slice(0, launchRows);
        const order = [...new Set(best.map(it => it.section))];
        const top = Compat.flatMap(order, sec => best.filter(it => it.section === sec));
        if (!prefixed)
            top.push(webItem(quicklinks[0], q, 1));
        if (ql.split(/\s+/).length >= 3 || /\?$/.test(q))
            top.push({id: "ia-fallback", section: "Web", icon: "starred-symbolic", score: 0, title: "Perguntar à IA: " + q,
                subtitle: "Resposta do modelo local, aqui mesmo", hint: "Perguntar", keepOpen: true,
                run: () => {
                    l.enterView("ia");
                    l.send(q, {});
                }});
        return top;
    }

    function appName(appId) {
        const e = appId ? DesktopEntries.heuristicLookup(appId) : null;
        return e ? e.name : appId || "";
    }

    function appIconFor(appId) {
        const e = appId ? DesktopEntries.heuristicLookup(appId) : null;
        return e && e.icon ? Quickshell.iconPath(e.icon, true) : "";
    }

    function isInstanceOf(entry, winApp) {
        const app = (winApp || "").toLowerCase();
        if (!app)
            return false;
        const id = entry.id.replace(/\.desktop$/, "").toLowerCase();
        return app === id || app === (entry.startupClass || "").toLowerCase() || DesktopEntries.heuristicLookup(winApp) === entry;
    }

    // Enter on an app that is already running focuses its window instead of opening a second one.
    function openApp(entry) {
        Compositor.windows()
            .then(all => all.find(w => isInstanceOf(entry, w.app)))
            .catch(() => null)
            .then(w => w ? Compositor.activate(w) : Sys.launchApp(entry));
    }

    function appItem(a, score, section) {
        return {id: "app:" + a.id, key: "app:" + a.id, section, appIcon: a.icon ? Quickshell.iconPath(a.icon, true) : "",
            icon: "application-x-executable-symbolic", score, title: a.name, subtitle: a.comment || "", hint: "Abrir",
            run: () => l.openApp(a), alt: () => Sys.launchApp(a)};
    }

    function historyItem(h, score) {
        const img = h.tipo === "image";
        const ocr = (h.ocr || "").replace(/\s+/g, " ").trim();
        const text = img ? "Imagem" + (ocr ? " · " + ocr.slice(0, 70) : "") : (h.texto || "").replace(/\s+/g, " ");
        return {id: "cb:" + h.id, data: h, section: "Histórico do Clippit",
            icon: h.fixado ? "view-pin-symbolic" : img ? "image-x-generic-symbolic" : /^https?:\/\//.test(h.texto || "") ? "web-browser-symbolic" : "edit-paste-symbolic",
            score, title: text.slice(0, 90) || "(vazio)", subtitle: "Histórico", hint: "Colar",
            run: () => l.pasteHistory(h.id, true), alt: () => l.pasteHistory(h.id, false)};
    }

    function webItem(w, q, score) {
        return {id: "web:" + w.key, key: "web:" + w.key, section: "Web", icon: w.icon || "web-browser-symbolic", score,
            title: "Pesquisar “" + q + "” no " + w.name, subtitle: "Atalho: " + w.key + " + espaço", hint: "Pesquisar",
            run: () => Sys.openUri(w.url.replace("{q}", encodeURIComponent(q)))};
    }

    function quickNumberItems(q) {
        const items = [];
        let m = q.match(/^timer\s+(\d+(?:[.,]\d+)?)\s*(min|m|minutos?)?\s*(.*)$/i);
        if (m) {
            const min = parseFloat(m[1].replace(",", "."));
            const label = (m[3] || "").trim() || "Timer de " + m[1] + " min";
            items.push({id: "timer", section: "Comandos", icon: "alarm-symbolic", score: 990, title: "Timer de " + m[1] + " min",
                subtitle: (m[3] || "").trim() || "Avisa com notificação e som", hint: "Marcar",
                run: () => {
                    l.addReminder(min, label);
                    Sys.notify("Clippit", "Timer de " + m[1] + " min marcado.");
                }});
        }
        m = q.match(/^vol(?:ume)?\s+(\d{1,3})\s*%?$/i);
        if (m) {
            const v = Math.min(150, +m[1]);
            items.push({id: "volume", section: "Comandos", icon: "audio-volume-high-symbolic", score: 990, title: "Volume em " + v + "%",
                subtitle: "Saída de som padrão", hint: "Aplicar",
                run: () => Sys.detach(["wpctl", "set-volume", "-l", "1.5", "@DEFAULT_AUDIO_SINK@", v + "%"])});
        }
        m = q.match(/^brilho\s+(\d{1,3})\s*%?$/i);
        if (m) {
            const v = Math.max(1, Math.min(100, +m[1]));
            items.push({id: "brilho", section: "Comandos", icon: "display-brightness-symbolic", score: 990, title: "Brilho em " + v + "%",
                subtitle: "Tela do notebook", hint: "Aplicar", run: () => Sys.detach(["brightnessctl", "set", v + "%"])});
        }
        return items;
    }

    // ---- System controls ----------------------------------------------------
    // Read once per opening (the search runs on every letter).

    function loadControls() {
        Compat.co(function* () {
            const c = {dark: false, dnd: null, dndKind: "", bt: null, brightness: false};
            try {
                c.dark = (yield Sys.run(["gsettings", "get", "org.gnome.desktop.interface", "color-scheme"], null, 3)).indexOf("dark") >= 0;
            } catch (e) {}
            if (yield Sys.has("swaync-client")) {
                c.dndKind = "swaync";
                c.dnd = (yield Sys.run(["swaync-client", "-D"], null, 3).catch(() => "")).trim() === "true";
            } else if (yield Sys.has("makoctl")) {
                c.dndKind = "mako";
                c.dnd = (yield Sys.run(["makoctl", "mode"], null, 3).catch(() => "")).indexOf("do-not-disturb") >= 0;
            } else if (yield Sys.has("dunstctl")) {
                c.dndKind = "dunst";
                c.dnd = (yield Sys.run(["dunstctl", "is-paused"], null, 3).catch(() => "")).trim() === "true";
            }
            const bt = yield Sys.run(["bluetoothctl", "show"], null, 3).catch(() => "");
            c.bt = /Powered:\s*yes/.test(bt) ? false : /Powered:\s*no/.test(bt) ? true : null; // true = off
            c.brightness = yield Sys.has("brightnessctl");
            l.ctl = c;
        });
    }

    function controlCommands() {
        const c = ctl;
        const sh = cmd => () => Sys.detach(["sh", "-c", cmd]);
        const list = [
            {id: "ctl:wifi", key: "ctl:wifi", icon: "network-wireless-symbolic", title: "Wi-Fi (liga/desliga)", subtitle: "Sistema",
                words: ["wifi", "wi-fi", "internet", "rede sem fio"], hint: "Alternar",
                run: sh("if [ \"$(nmcli radio wifi)\" = enabled ]; then nmcli radio wifi off; else nmcli radio wifi on; fi")},
            {id: "ctl:bt", key: "ctl:bt", icon: "bluetooth-active-symbolic",
                title: c.bt === null ? "Bluetooth (liga/desliga)" : c.bt ? "Ligar Bluetooth" : "Desligar Bluetooth",
                subtitle: c.bt === null ? "Sistema" : c.bt ? "Está desligado" : "Está ligado", words: ["bluetooth", "bt", "fone"],
                hint: "Alternar", run: sh("if bluetoothctl show | grep -q 'Powered: yes'; then bluetoothctl power off; else rfkill unblock bluetooth; bluetoothctl power on; fi")},
            {id: "ctl:dark", key: "ctl:dark", icon: "weather-clear-night-symbolic", title: c.dark ? "Usar tema claro" : "Usar modo escuro",
                subtitle: c.dark ? "Modo escuro está ligado" : "Modo escuro está desligado", words: ["modo escuro", "tema", "dark", "claro"],
                hint: "Alternar", run: () => Sys.detach(["gsettings", "set", "org.gnome.desktop.interface", "color-scheme", c.dark ? "default" : "prefer-dark"])},
        ];
        if (c.dndKind) {
            const toggle = {swaync: "swaync-client -d", mako: "makoctl mode -t do-not-disturb", dunst: "dunstctl set-paused toggle"}[c.dndKind];
            list.push({id: "ctl:dnd", key: "ctl:dnd", icon: "notifications-disabled-symbolic",
                title: c.dnd ? "Desligar o Não perturbe" : "Ligar o Não perturbe",
                subtitle: c.dnd ? "Notificações escondidas" : "Notificações aparecendo",
                words: ["nao perturbe", "notificacoes", "silencio", "foco"], hint: "Alternar", run: sh(toggle)});
        }
        list.push(
            {id: "ctl:mute", key: "ctl:mute", icon: "audio-volume-muted-symbolic", title: "Som mudo (liga/desliga)", subtitle: "Saída de som",
                words: ["mudo", "mute", "silenciar", "som"], hint: "Alternar", run: sh("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle")},
            {id: "ctl:volup", key: "ctl:volup", icon: "audio-volume-high-symbolic", title: "Aumentar o volume", subtitle: "+10% · ou \"volume 50\"",
                words: ["volume", "mais alto", "aumentar som"], hint: "Aplicar", run: sh("wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 10%+")},
            {id: "ctl:voldown", key: "ctl:voldown", icon: "audio-volume-low-symbolic", title: "Diminuir o volume", subtitle: "−10%",
                words: ["volume", "mais baixo", "abaixar som"], hint: "Aplicar", run: sh("wpctl set-volume @DEFAULT_AUDIO_SINK@ 10%-")});
        if (c.brightness)
            list.push(
                {id: "ctl:brup", key: "ctl:brup", icon: "display-brightness-symbolic", title: "Aumentar o brilho", subtitle: "ou \"brilho 70\"",
                    words: ["brilho", "tela mais clara"], hint: "Aplicar", run: sh("brightnessctl set 10%+")},
                {id: "ctl:brdown", key: "ctl:brdown", icon: "display-brightness-symbolic", title: "Diminuir o brilho", subtitle: "Tela",
                    words: ["brilho", "tela mais escura"], hint: "Aplicar", run: sh("brightnessctl set 10%-")});
        return list;
    }

    // ---- Windows (the one in focus before the launcher) ---------------------

    function windowCommands() {
        const w = prevWin;
        if (!w)
            return [];
        const name = w.title || "janela";
        const mk = (op, title, icon, words) => ({id: "win:" + op, key: "win:" + op, icon, title, subtitle: name,
            words: ["janela", "window"].concat(words), hint: "Aplicar", run: () => Compositor.winOp(w, op)});
        return [
            mk("left", "Janela na metade esquerda", "view-dual-symbolic", ["esquerda", "left", "lado"]),
            mk("right", "Janela na metade direita", "view-dual-symbolic", ["direita", "right", "lado"]),
            mk("max", "Maximizar janela", "window-maximize-symbolic", ["maximizar", "tela toda"]),
            mk("restore", "Restaurar tamanho da janela", "window-restore-symbolic", ["restaurar", "desmaximizar"]),
            mk("center", "Centralizar janela", "focus-windows-symbolic", ["centralizar", "meio"]),
            mk("monitor", "Mandar janela para o outro monitor", "video-display-symbolic", ["monitor", "tela", "outra tela"]),
            mk("full", "Tela cheia (liga/desliga)", "view-fullscreen-symbolic", ["tela cheia", "fullscreen"]),
            mk("min", "Minimizar janela", "window-minimize-symbolic", ["minimizar", "esconder"]),
            mk("close", "Fechar janela", "window-close-symbolic", ["fechar", "close"]),
        ];
    }

    function aiFilter(items) {
        const need = {"cmd:ia": "chat", "cmd:conversas": "chat", "cmd:acoes": "acoes"};
        return items.filter(it => !need[it.id] || Conf.aiOn(need[it.id]));
    }

    function commands() {
        const spawn = argv => () => Sys.detach(argv);
        return aiFilter([
            {id: "cmd:historico", key: "cmd:historico", icon: "edit-paste-symbolic", title: "Histórico da área de transferência",
                subtitle: "Tudo o que você copiou, com prévia", words: ["historico", "clipboard", "copiados", "area de transferencia", "cb"],
                hint: "Abrir", keepOpen: true, run: () => l.enterView("historico")},
            {id: "cmd:ia", key: "cmd:ia", icon: "starred-symbolic", title: "Perguntar à IA",
                subtitle: "Vê o sistema, pesquisa, cotações, clima · Alt+I", words: ["ia", "ai", "chat", "pergunta", "assistente"],
                hint: "Abrir", keepOpen: true, run: () => l.enterView("ia")},
            {id: "cmd:conversas", key: "cmd:conversas", icon: "user-available-symbolic", title: "Conversas anteriores com a IA",
                subtitle: "Retomar uma conversa · Ctrl+H na IA", words: ["conversas", "historico ia", "chats"],
                hint: "Abrir", keepOpen: true, run: () => {
                    l.convs = null;
                    l.enterView("conversas");
                }},
        ].concat(controlCommands(), windowCommands(), [
            {id: "cmd:snippets", key: "cmd:snippets", icon: "insert-text-symbolic", title: "Snippets (textos prontos)",
                subtitle: "Digite ; para ver · ;gatilho = texto cria um", words: ["snippet", "texto pronto", "atalho de texto"],
                hint: "Abrir", keepOpen: true, run: () => l.setEntry(";", -1)},
            {id: "cmd:emoji", key: "cmd:emoji", icon: "face-smile-symbolic", title: "Emoji e símbolos",
                subtitle: "Digite : e o nome (ex.: :coração)", words: ["emoji", "emoticon", "simbolo"],
                hint: "Abrir", keepOpen: true, run: () => l.enterView("emoji")},
            {id: "cmd:acoes", key: "cmd:acoes", icon: "edit-select-all-symbolic", title: "Ações no texto selecionado",
                subtitle: "Resumir, traduzir, corrigir… · Alt+A", words: ["resumir", "traduzir", "texto selecionado"],
                hint: "Abrir", keepOpen: true, run: () => l.openTextActions(true)},
            {id: "cmd:lock", key: "cmd:lock", icon: "system-lock-screen-symbolic", title: "Bloquear tela", subtitle: "Sistema",
                words: ["travar", "lock", "bloquear"], hint: "Executar", run: spawn(["loginctl", "lock-session"])},
            {id: "cmd:suspend", key: "cmd:suspend", icon: "weather-clear-night-symbolic", title: "Suspender", subtitle: "Sistema",
                words: ["dormir", "sleep", "suspend"], hint: "Executar", run: spawn(["systemctl", "suspend"])},
            {id: "cmd:reboot", key: "cmd:reboot", icon: "system-reboot-symbolic", title: "Reiniciar…", subtitle: "Pede confirmação",
                words: ["restart", "reboot", "reiniciar"], hint: "Executar",
                confirm: "Reiniciar o computador? Enter de novo para confirmar", run: spawn(["systemctl", "reboot"])},
            {id: "cmd:poweroff", key: "cmd:poweroff", icon: "system-shutdown-symbolic", title: "Desligar…", subtitle: "Pede confirmação",
                words: ["shutdown", "desligar", "power off"], hint: "Executar",
                confirm: "Desligar o computador? Enter de novo para confirmar", run: spawn(["systemctl", "poweroff"])},
            {id: "cmd:logout", key: "cmd:logout", icon: "system-log-out-symbolic", title: "Sair da sessão…", subtitle: "Pede confirmação",
                words: ["logout", "sair", "encerrar sessão"], hint: "Executar",
                confirm: "Sair da sessão? Enter de novo para confirmar", run: () => Compositor.logout()},
            {id: "cmd:trash", key: "cmd:trash", icon: "user-trash-full-symbolic", title: "Esvaziar lixeira",
                subtitle: "Apaga de vez os arquivos da lixeira (pede Enter duas vezes)", words: ["lixeira", "trash", "limpar lixo"],
                hint: "Executar", confirm: "Esvaziar a lixeira? Não dá para desfazer. Enter de novo para confirmar",
                run: () => {
                    Sys.detach(["gio", "trash", "--empty"]);
                    Sys.notify("Clippit", "Lixeira esvaziada.");
                }},
            {id: "cmd:game", key: "cmd:game", icon: "input-gaming-symbolic", title: "Modo jogo (liga/desliga)",
                subtitle: "Tira a IA da placa de vídeo · " + Conf.label("modo_jogo", "+"), words: ["jogo", "game", "liberar placa", "vram"],
                hint: "Executar", run: () => l.assistente.toggleGameMode()},
            {id: "cmd:clippit", key: "cmd:clippit", icon: "preferences-system-symbolic", title: "Configurações do Clippit",
                subtitle: "Corretor, assistente, aprendizado", words: ["clippit", "painel", "dashboard"], hint: "Abrir",
                run: () => Sys.launch(Sys.tool("clippit-dashboard"))},
            {id: "cmd:corretor", key: "cmd:corretor", icon: "tools-check-spelling-symbolic", title: "Rodar a prova do corretor",
                subtitle: "Mede a qualidade (uns 50 s)", words: ["prova", "teste corretor", "qualidade"], hint: "Executar",
                run: () => {
                    Sys.notify("Clippit", "Rodando a prova do corretor…");
                    Sys.run(Sys.tool("clippit-correct").concat(["--prova"]), null, 600).then(out =>
                        Sys.notify("Clippit — prova do corretor", out.split("\n").slice(1, 3).join(" ").trim()));
                }},
        ]));
    }

    // ---- Actions ------------------------------------------------------------

    function activate(item, alt) {
        if (!item)
            return;
        if (item.confirm && !item.confirmed) {
            // Actions that cannot be undone ask for a second Enter.
            const list = shown.slice();
            const i = list.indexOf(item);
            list[i] = Object.assign({}, item, {confirmed: true, title: item.confirm});
            shown = list;
            return;
        }
        if (item.key)
            bump(item.key);
        const fn = alt && item.alt ? item.alt : item.run;
        if (!item.keepOpen) {
            consumed = true;
            close();
        }
        try {
            fn();
        } catch (e) {
            console.log("clippit: ação falhou: " + e.message);
            Sys.notify("Clippit", "Não consegui: " + e.message);
        }
    }

    function pasteHistory(id, paste) {
        Sys.run(Sys.tool("clippit-busca").concat(["copiar", String(id)]), null, 5).then(() => {
            if (!paste) {
                Sys.notify("Clippit", "Copiado para a área de transferência.");
                return;
            }
            const w = l.prevWin;
            Compat.co(function* () {
                if (w)
                    yield Compositor.activate(w);
                yield Compat.sleep(250);
                yield Paste.keys("ctrl+v");
            });
        }).catch(e => Sys.notify("Clippit", "Não consegui copiar: " + e.message));
    }

    function pasteText(text) {
        consumed = true;
        if (open)
            close();
        Paste.into(prevWin, text);
    }

    function deleteHistory(item) {
        const h = item && item.data;
        if (!h)
            return;
        Sys.run(Sys.tool("clippit-busca").concat(["apagar", String(h.id)]), null, 5).then(() => {
            l.asyncHistory = l.asyncHistory.filter(x => x.id !== h.id);
            l.refresh(true);
        }).catch(e => Sys.notify("Clippit", "Não consegui apagar: " + e.message));
    }

    function togglePin(h) {
        Sys.run(Sys.tool("clippit-busca").concat([h.fixado ? "desafixar" : "fixar", String(h.id)]), null, 5).then(() => {
            l.cbTx = null;
            if (l.side) {
                l.closeSide();
                l.openSide();
            }
            if (l.open && l.isHistory) {
                const q = l.query.trim();
                return Sys.run(Sys.tool("clippit-busca").concat(["historico", l.view === "historico" ? q : q.replace(/^cb\s*/i, "")]), null, 5)
                    .then(o => {
                        l.asyncHistory = JSON.parse(o);
                        l.refresh(true);
                    });
            }
        }).catch(e => Sys.notify("Clippit", "Não consegui fixar: " + e.message));
    }

    // ---- History: transform and pin -----------------------------------------

    function cbTxItems(q) {
        const h = cbTx;
        const text = h.tipo === "image" ? (h.ocr || "").trim() : h.texto || "";
        const items = Busca.TEXT_TRANSFORMS.map(tf => ({tf, s: q ? Busca.matchScore(q, tf.title) : 100})).filter(x => x.s >= 40)
            .map(x => ({id: "cbtx:" + x.tf.id, data: h, icon: x.tf.icon, score: x.s, title: x.tf.title,
                subtitle: x.tf.ai ? "IA local" : "", hint: "Colar", keepOpen: true,
                run: () => l.applyCbTransform(x.tf, text, false), alt: () => l.applyCbTransform(x.tf, text, true)}));
        items.push({id: "cbtx:pin", data: h, icon: "view-pin-symbolic", score: 0,
            title: h.fixado ? "Desafixar do topo" : "Fixar no topo (não some com o tempo)", subtitle: "Ctrl+P", hint: "Aplicar",
            keepOpen: true, run: () => l.togglePin(h)});
        return items;
    }

    function applyCbTransform(tf, text, copyOnly) {
        if (!text) {
            Sys.notify("Clippit", "Esse item não tem texto para transformar.");
            return;
        }
        cbTx = null;
        if (tf.ai) {
            selText = text;
            selFrom = "copiado";
            enterView("texto");
            runTx(tf.ai, tf.title, "");
            return;
        }
        let out;
        try {
            out = tf.fn(text);
        } catch (e) {
            Sys.notify("Clippit", "Não deu para " + tf.title.toLowerCase() + ": " + e.message);
            refresh(false);
            return;
        }
        if (copyOnly) {
            Sys.copy(out);
            consumed = true;
            close();
            Sys.notify("Clippit", "Copiado.");
        } else {
            pasteText(out);
        }
    }

    function runTransform(tf, text) {
        if (tf.ai)
            return Sys.run(Sys.tool("clippit-correct").concat(["--transformar"]), JSON.stringify({acao: tf.ai, texto: text}), 240)
                .then(o => o.trim());
        return new Promise(resolve => resolve(tf.fn(text)));
    }

    // ---- Alt+A: actions on the selected text --------------------------------

    function openTextActions(fromLauncher) {
        if (!Conf.aiOn("acoes")) {
            Conf.aiOffNotice("As ações no texto");
            return;
        }
        Compat.co(function* () {
            let text = (yield Sys.clipboard(true)).trim();
            let from = "selecionado";
            if (!text || fromLauncher) {
                const copied = (yield Sys.clipboard(false)).trim();
                if (!text || (fromLauncher && copied)) {
                    text = copied;
                    from = "copiado";
                }
            }
            if (!text) {
                Sys.notify("Clippit", "Selecione (ou copie) um texto e aperte Alt+A.");
                return;
            }
            if (l.open && !fromLauncher)
                l.close();
            l.selText = text;
            l.selFrom = from;
            l.tx = null;
            if (!l.open)
                l.openLauncher("texto");
            else
                l.enterView("texto");
        });
    }

    function textActionItems(q) {
        if (!selText)
            return [{id: "tx-vazio", title: "Selecione um texto em qualquer app e aperte Alt+A", subtitle: "",
                icon: "edit-select-all-symbolic", score: 0, keepOpen: true, run: () => {}}];
        const preview = selText.replace(/\s+/g, " ").slice(0, 70);
        const section = "Texto " + selFrom + ": “" + preview + (selText.length > 70 ? "…" : "") + "”";
        const items = Busca.TEXT_ACTIONS.map(a => ({a, s: q ? Math.max.apply(null, [Busca.matchScore(q, a.title)]
            .concat(a.words.map(w => Busca.matchScore(q, w) * 0.8))) : 100})).filter(x => x.s >= 40)
            .map(x => ({id: "tx:" + x.a.acao, section, icon: x.a.icon, score: x.s, title: x.a.title, subtitle: "", hint: "Fazer",
                keepOpen: true, run: () => l.runTx(x.a.acao, x.a.title, "")}));
        if (q)
            items.unshift({id: "tx:livre", section, icon: "document-edit-symbolic", score: 1000, title: "Fazer: " + q,
                subtitle: "Pedido livre sobre o texto", hint: "Fazer", keepOpen: true, run: () => l.runTx("", q, q)});
        return items;
    }

    function runTx(acao, title, instrucao) {
        if (acao === "perguntar") {
            const text = selText;
            enterView("ia");
            attachToken(null, {tipo: "texto", nome: "seleção", texto: text});
            return;
        }
        const t = {title, pending: true, text: "", live: "", error: ""};
        tx = t;
        setEntry("", 0);
        Sys.write(statusFile, "");
        txPoll.target = t;
        txPoll.start();
        Sys.run(Sys.tool("clippit-correct").concat(["--transformar"]), JSON.stringify({acao, texto: selText, instrucao}), 240)
            .then(out => t.text = out.trim()).catch(e => t.error = e.message).then(() => {
                t.pending = false;
                txPoll.stop();
                if (l.tx === t)
                    l.tx = Object.assign({}, t);
            });
    }

    Timer {
        id: txPoll
        property var target: null
        interval: 200
        repeat: true
        onTriggered: Sys.read(l.statusFile).then(raw => {
            const st = JSON.parse(raw.trim() || "{}");
            if (l.tx === txPoll.target && (st.texto || "") !== l.tx.live)
                l.tx = Object.assign({}, l.tx, {live: st.texto || ""});
        }).catch(() => {})
    }

    function txKey(enter, ctrl) {
        if (!tx || tx.pending || !tx.text)
            return;
        if (ctrl) {
            Sys.copy(tx.text);
            consumed = true;
            close();
            Sys.notify("Clippit", "Copiado.");
        } else {
            pasteText(tx.text);
        }
    }

    // ---- Snippets (;trigger) ------------------------------------------------

    function snippetItems(q) {
        const items = [];
        const make = q.match(/^;([^\s=]+)\s*=\s*([\s\S]+)$/);
        if (make) {
            const gat = ";" + make[1];
            items.push({id: "snip:novo", section: "Snippets", icon: "list-add-symbolic", score: 1000, title: "Criar snippet " + gat,
                subtitle: make[2].slice(0, 80), hint: "Criar", keepOpen: true,
                run: () => {
                    const list = Snippets.items.filter(x => x.gatilho !== gat);
                    list.push({gatilho: gat, nome: make[1], texto: make[2].split("\\n").join("\n")});
                    Snippets.save(list);
                    Sys.notify("Clippit", "Snippet " + gat + " criado. Digite " + gat + " e espaço em qualquer campo.");
                    l.setEntry(";", -1);
                }});
            return items;
        }
        const term = q.slice(1);
        for (const sn of Snippets.items) {
            const sc = term ? Math.max(Busca.matchScore(term, sn.gatilho.slice(1)), Busca.matchScore(term, sn.nome || "") * 0.9,
                Busca.matchScore(term, sn.texto) * 0.5) : 100;
            if (sc < 40)
                continue;
            items.push({id: "snip:" + sn.gatilho, section: "Snippets", icon: "insert-text-symbolic", score: sc,
                title: sn.gatilho + "  " + (sn.nome || ""), subtitle: sn.texto.replace(/\s+/g, " ").slice(0, 90), hint: "Colar",
                run: () => Snippets.expand(sn.texto).then(t => l.pasteText(t)),
                alt: () => Snippets.expand(sn.texto).then(t => {
                    Sys.copy(t);
                    Sys.notify("Clippit", "Snippet copiado.");
                })});
        }
        items.sort((a, b) => b.score - a.score);
        items.push({id: "snip:editar", section: "Snippets", icon: "document-edit-symbolic", score: 0, title: "Editar snippets",
            subtitle: "Abre o arquivo · ou digite ;gatilho = texto", hint: "Abrir",
            run: () => Sys.exists(Snippets.path).then(ok => (ok ? Promise.resolve() : Snippets.save([
                {gatilho: ";data", nome: "Data de hoje", texto: "{data}"},
                {gatilho: ";obg", nome: "Agradecimento", texto: "Muito obrigado! Qualquer coisa, estou à disposição."}])))
                .then(() => Sys.openUri("file://" + Snippets.path))});
        return items;
    }

    // ---- Emoji grid ---------------------------------------------------------

    property bool emojiTried: false

    function loadEmoji() {
        if (emojiData || emojiLoading)
            return;
        const cache = Sys.cacheDir + "/emoji-v2.json";
        Sys.readJson(cache, null).then(list => {
            if (list) {
                l.emojiData = list.map(x => ({e: x.e, desc: x.d, cat: x.c, words: x.w, all: Busca.fold(x.w.join(" | "))}));
                return;
            }
            if (l.emojiTried) {
                console.log("clippit: não consegui carregar os emoji");
                return;
            }
            l.emojiTried = true;
            l.emojiLoading = true;
            return Sys.run(Sys.tool("clippit-busca").concat(["emoji"]), null, 30).then(() => {
                l.emojiLoading = false;
                l.loadEmoji();
            });
        }).catch(e => {
            l.emojiLoading = false;
            console.log("clippit: emoji: " + e.message);
        });
    }

    function emojiSections(q) {
        const data = emojiData;
        if (!data)
            return null;
        const byE = new Map(data.map(x => [x.e, x]));
        const term = Busca.fold(q.trim());
        if (term) {
            const first = term.split(/\s+/)[0];
            const found = data.filter(x => x.all.indexOf(first) >= 0).map(x => ({x, s: Busca.emojiScore(q.trim(), x)}))
                .filter(y => y.s >= 45).sort((a, b) => b.s - a.s).slice(0, 96).map(y => y.x);
            return [{title: found.length ? "Resultados" : "Nenhum emoji com esse nome", items: found}];
        }
        const pick = list => list.map(e => byE.get(e) || {e, desc: e, words: []});
        const sections = [];
        const cat = emojiCat || "Sorrisos e emoções";
        if (cat === "Fixados" || !emojiCat) {
            sections.push({title: "Fixados", items: pick(emojiPrefs.fixados)});
            if (emojiPrefs.recentes.length)
                sections.push({title: "Recentes", items: pick(emojiPrefs.recentes.slice(0, 16))});
        }
        if (cat !== "Fixados")
            sections.push({title: cat, items: data.filter(x => x.cat === cat)});
        return sections;
    }

    function emojiUse(x, copyOnly) {
        if (!x)
            return;
        const p = {fixados: emojiPrefs.fixados, recentes: [x.e].concat(emojiPrefs.recentes.filter(e => e !== x.e)).slice(0, 24)};
        emojiPrefs = p;
        Sys.writeJson(Sys.dataDir + "/emoji.json", p);
        if (copyOnly) {
            Sys.copy(x.e);
            consumed = true;
            close();
            Sys.notify("Clippit", x.e + " copiado.");
        } else {
            pasteText(x.e);
        }
    }

    function emojiPin(x) {
        const f = emojiPrefs.fixados;
        const p = {fixados: f.indexOf(x.e) >= 0 ? f.filter(e => e !== x.e) : f.concat([x.e]), recentes: emojiPrefs.recentes};
        emojiPrefs = p;
        Sys.writeJson(Sys.dataDir + "/emoji.json", p);
    }

    function nextEmojiCat(dir) {
        const cats = Busca.EMOJI_CATS;
        const i = cats.indexOf(emojiCat || "Sorrisos e emoções");
        emojiCat = cats[(i + (dir < 0 ? cats.length - 1 : 1)) % cats.length];
        emojiSel = 0;
        setEntry("", 0);
    }

    // ---- Reminders and timers -----------------------------------------------

    readonly property string remindFile: Sys.dataDir + "/lembretes.json"

    function addReminder(minutes, texto) {
        Sys.readJson(remindFile, []).then(items => {
            items = items.filter(x => !x.feito);
            items.push({id: Date.now(), quando: Date.now() / 1000 + minutes * 60, texto, timer: true});
            Sys.writeJson(l.remindFile, items);
        });
    }

    function checkReminders() {
        Sys.readJson(remindFile, []).then(items => {
            const now = Date.now() / 1000;
            const due = items.filter(x => !x.feito && x.quando <= now);
            if (!due.length)
                return;
            for (const x of due) {
                x.feito = true;
                l.alarm(x.timer ? "Timer acabou" : "Lembrete", x.texto);
            }
            Sys.writeJson(l.remindFile, items.filter(x => !x.feito || now - x.quando < 86400));
        });
    }

    // A notification that stays until you close it, with a sound.
    function alarm(title, body) {
        Sys.notify(title, body, "critical");
        Sys.detach(["sh", "-c", "canberra-gtk-play -i alarm-clock-elapsed 2>/dev/null && exit 0; "
            + "for d in $(echo \"${XDG_DATA_DIRS:-/usr/local/share:/usr/share}\" | tr : ' '); do "
            + "f=\"$d/sounds/freedesktop/stereo/alarm-clock-elapsed.oga\"; "
            + "[ -f \"$f\" ] && { pw-play \"$f\" 2>/dev/null || paplay \"$f\" 2>/dev/null; exit 0; }; done"]);
    }

    Timer {
        interval: 15000
        running: true
        repeat: true
        onTriggered: l.checkReminders()
    }

    // ---- Games always on the dedicated GPU -----------------------------------
    // When a new game shortcut shows up (Steam, Lutris…), clippit-jogos-nvidia
    // makes it ask for the NVIDIA card too.

    property string appsStamp: ""

    function watchGames() {
        Sys.run(["sh", "-c", "ls -l --time-style=+%s \"${XDG_DATA_HOME:-$HOME/.local/share}/applications\" 2>/dev/null | md5sum"], null, 5)
            .then(stamp => {
                if (stamp === l.appsStamp)
                    return;
                l.appsStamp = stamp;
                Sys.run(Sys.tool("clippit-jogos-nvidia"), null, 60).then(out => {
                    if (out.startsWith("Na NVIDIA agora"))
                        console.log("clippit: " + out.trim());
                }).catch(() => {});
            }).catch(() => {});
    }

    Timer {
        interval: 60000
        running: true
        repeat: true
        onTriggered: l.watchGames()
    }

    // ---- Saved conversations (Ctrl+H) ---------------------------------------

    function saveChat() {
        const msgs = chat.filter(m => !m.pending && (m.text || m.role === "user"));
        if (!msgs.some(m => m.role === "user"))
            return;
        if (!chatId)
            chatId = String(Date.now());
        const data = {
            id: chatId, titulo: msgs.find(m => m.role === "user").text.slice(0, 90), atualizado: Date.now(),
            mensagens: msgs.map(m => ({role: m.role, text: m.text, steps: m.steps, sources: m.sources, outputs: m.outputs,
                attachments: m.attachments, confirm: m.confirm, choice: m.choice,
                choiceAnswered: m.choice ? (m.choiceAnswered !== undefined ? m.choiceAnswered : "texto") : undefined,
                confirmState: m.confirmState || (m.confirm ? "no" : undefined)})),
        };
        Sys.writeJson(convDir + "/" + chatId + ".json", data);
    }

    function loadConvs() {
        const script = "import json,glob,os,sys\nout=[]\nfor f in glob.glob(os.path.join(sys.argv[1],'*.json')):\n" +
            "    try:\n        c=json.load(open(f,encoding='utf-8'))\n        if isinstance(c,dict) and isinstance(c.get('mensagens'),list):\n            c['arquivo']=f\n            out.append(c)\n" +
            "    except (OSError, ValueError): pass\nprint(json.dumps(out))";
        return Sys.run(["python3", "-c", script, convDir], null, 10).then(raw => JSON.parse(raw).sort((a, b) => (b.atualizado || 0) - (a.atualizado || 0)))
            .catch(() => []);
    }

    function convItems(q) {
        if (convs === null) {
            convs = [];
            loadConvs().then(list => {
                l.convs = list;
                l.refresh(false);
            });
        }
        const items = convs.map(c => ({c, s: q ? Math.max.apply(null, [Busca.matchScore(q, c.titulo || "")]
            .concat(c.mensagens.map(m => Busca.matchScore(q, m.text || "") * 0.8))) : 1})).filter(x => x.s > 0)
            .map(x => {
                const n = x.c.mensagens.filter(m => m.role === "user").length;
                return {id: "conv:" + x.c.id, section: "Conversas", icon: "user-available-symbolic", score: x.c.atualizado,
                    title: x.c.titulo, subtitle: Busca.when(x.c.atualizado) + " · " + n + " " + (n === 1 ? "pergunta" : "perguntas"),
                    hint: "Abrir", keepOpen: true, data: {conv: x.c},
                    run: () => {
                        l.chat = x.c.mensagens.map(m => Object.assign({}, m));
                        l.chatId = x.c.id;
                        l.chatTime = Date.now();
                        l.enterView("ia");
                    }};
            });
        if (!items.length)
            items.push({id: "conv-vazio", title: q ? "Nenhuma conversa com esse texto" : "Nenhuma conversa salva ainda", subtitle: "",
                icon: "user-available-symbolic", score: 0, run: () => {}, keepOpen: true});
        return items;
    }

    function deleteConv(it) {
        const c = it && it.data && it.data.conv;
        if (!c)
            return;
        if (c.arquivo)
            Sys.remove(c.arquivo);
        convs = convs.filter(x => x.id !== c.id);
        if (chatId === c.id) {
            chat = [];
            chatId = "";
        }
        refresh(false);
    }

    // The doctor's diagnosis left for you to see (and run its fix with "Rodar").
    function doctorInbox() {
        const path = Sys.dataDir + "/doutor/sugestao.json";
        if (busyChat)
            return;
        Sys.readJson(path, null).then(sg => {
            if (!sg)
                return;
            Sys.remove(path);
            l.chat = l.chat.concat([{role: "user", text: "Diagnóstico do Clippit"},
                {role: "assistant", text: sg.texto || "Achei um problema.", confirm: sg.comando || null, steps: [], sources: []}]);
            l.saveChat();
        });
    }

    // ---- @: attach an app, a file or a screenshot to the question ------------

    function updateMention() {
        if (view !== "ia") {
            mention = null;
            return;
        }
        const pos = cursorPos < 0 ? query.length : cursorPos;
        const m = query.slice(0, pos).match(/(^|\s)@([^\s@]*)$/);
        if (!m) {
            mention = null;
            return;
        }
        const q = m[2];
        const start = pos - q.length - 1;
        const g = ++mentionGen;
        const list = q ? apps.map(a => ({a, s: Busca.matchScore(q, a.name)})).filter(x => x.s > 0).sort((x, y) => y.s - x.s).slice(0, 5).map(x => x.a)
            : apps.map(a => ({a, n: usage["app:" + a.id] || 0})).sort((x, y) => y.n - x.n).slice(0, 5).map(x => x.a);
        const items = list.map(a => ({tipo: "app", nome: a.name, id: a.id, appIcon: a.icon ? Quickshell.iconPath(a.icon, true) : "", sub: "App"}));
        if (!q || ["print", "tela", "captura", "screenshot", "imagem"].some(w => w.startsWith(Busca.fold(q))))
            items.unshift({tipo: "print", nome: "print", lucide: "camera", sub: "Captura a janela de trás e a IA olha a imagem"});
        mention = {start, end: pos, q, items, sel: 0};
        if (q.length >= 2)
            Sys.run(Sys.tool("clippit-busca").concat(["arquivos", q]), null, 6).then(out => {
                if (g !== l.mentionGen || !l.mention)
                    return;
                const more = JSON.parse(out).slice(0, 5).map(f => ({tipo: "arquivo", nome: f.nome, uri: f.uri,
                    caminho: decodeURIComponent(f.uri.replace(/^file:\/\//, "")), lucide: "file", sub: f.pasta}));
                l.mention = Object.assign({}, l.mention, {items: l.mention.items.concat(more)});
            }).catch(() => {});
    }

    function mentionMove(d) {
        if (!mention || !mention.items.length)
            return;
        mention = Object.assign({}, mention, {sel: Math.max(0, Math.min(mention.items.length - 1, mention.sel + d))});
    }

    function mentionPick() {
        const mt = mention;
        const it = mt && mt.items[mt.sel];
        if (!it)
            return;
        if (it.tipo === "print") {
            attachPrint(mt);
            return;
        }
        attachToken(mt, {tipo: it.tipo, nome: it.nome, id: it.id, caminho: it.caminho, uri: it.uri});
    }

    function attachToken(mt, att) {
        const text = query;
        const before = text.slice(0, mt ? mt.start : text.length) + (mt || !text || /\s$/.test(text) ? "" : " ") + "@" + att.nome + " ";
        attach = attach.filter(a => a.nome !== att.nome).concat([att]);
        mention = null;
        setEntry(before + (mt ? text.slice(mt.end).replace(/^\s+/, "") : ""), before.length);
        mention = null;
    }

    function nextName(base) {
        const used = new Set(attach.map(a => a.nome));
        let n = 1;
        while (used.has(n === 1 ? base : base + n))
            n++;
        return n === 1 ? base : base + n;
    }

    // Hides the launcher for a moment, shoots the window behind and attaches it.
    property bool hiddenForShot: false

    function attachPrint(mt) {
        const path = Sys.workDir + "/anexos/print-" + Date.now() + ".png";
        const w = prevWin;
        const mon = monitor;
        Compat.co(function* () {
            l.hiddenForShot = true;
            yield Compat.sleep(180);
            try {
                yield Compositor.shoot(w && !w.minimized ? w : mon, path);
            } catch (e) {
                console.log("clippit: print falhou: " + e.message);
            }
            l.hiddenForShot = false;
            if (!l.open || !(yield Sys.exists(path)))
                return;
            l.attachToken(mt, {tipo: "imagem", nome: l.nextName("print"), caminho: path});
        });
    }

    // ---- Dictation (Alt+V) --------------------------------------------------

    function toggleDictation() {
        if (rec) {
            stopDictation(true);
            return;
        }
        const path = Sys.workDir + "/anexos/voz-" + Date.now() + ".wav";
        Sys.mkdir(Sys.workDir + "/anexos").then(() => {
            const p = recComponent.createObject(l, {command: ["pw-record", "--rate", "16000", "--channels", "1", "--format", "s16", path]});
            l.rec = {proc: p, path, t0: Date.now()};
            p.running = true;
            l.setFixState("rec", "Ouvindo… fale e aperte Alt+V para terminar");
            recTick.start();
        });
    }

    Component {
        id: recComponent
        RecProcess {}
    }

    Timer {
        id: recTick
        interval: 500
        repeat: true
        onTriggered: {
            if (!l.rec) {
                stop();
                return;
            }
            const secs = Math.floor((Date.now() - l.rec.t0) / 1000);
            l.fixMsg = "Ouvindo… " + secs + " s · Alt+V para terminar";
            if (secs >= 90)
                l.stopDictation(true); // safety limit
        }
    }

    function stopDictation(transcribe) {
        const r = rec;
        if (!r)
            return;
        rec = null;
        recTick.stop();
        r.proc.signal(2); // SIGINT: pw-record closes the file properly
        Compat.co(function* () {
            yield Compat.sleep(400);
            r.proc.destroy();
            if (!transcribe || !l.open) {
                Sys.remove(r.path);
                return;
            }
            l.setFixState("checking", "Transcrevendo…");
            try {
                const text = (yield Sys.run(Sys.tool("clippit-correct").concat(["--transcrever", r.path]), null, 180)).trim();
                if (!l.open)
                    return;
                if (text) {
                    const cur = l.query;
                    const pos = l.cursorPos < 0 ? cur.length : l.cursorPos;
                    l.insertEntry((pos > 0 && !/\s$/.test(cur.slice(0, pos)) ? " " : "") + text);
                    l.setFixState("", "");
                } else {
                    l.setFixState("error", "Não entendi nada. Tente de novo (Alt+V).");
                }
            } catch (e) {
                if (l.open)
                    l.setFixState("error", "Ditado: " + e.message.replace(/^.*?:\s*/, ""));
            }
            Sys.remove(r.path);
        });
    }

    // ---- Spell check of the launcher's own field (AI screen) ----------------

    function scheduleFix() {
        const text = query;
        if (fix && fix.from !== text) {
            fix = null;
            setFixState("", "");
        }
        if (fixState === "checking" && fixChecking !== text)
            setFixState("", "");
        fixTimer.stop();
        if (view !== "ia" || text.trim().split(/\s+/).length < 2)
            return;
        fixTimer.text = text;
        fixTimer.restart();
    }

    property string fixChecking: ""

    Timer {
        id: fixTimer
        property string text: ""
        interval: 700
        onTriggered: l.checkFix(text)
    }

    Timer {
        id: fixOk
        interval: 1500
        onTriggered: if (l.fixState === "ok" && !l.fix)
            l.setFixState("", "")
    }

    function setFixState(s, msg) {
        if (rec && s !== "rec" && s !== "checking" && s !== "error" && s !== "")
            return;
        fixState = s;
        fixMsg = msg || "";
        fixChecking = s === "checking" ? query : "";
        if (s === "ok")
            fixOk.restart();
    }

    function checkFix(text) {
        if (busyChat)
            return Promise.resolve();
        setFixState("checking", "Verificando a escrita…");
        return corretor.runCorrector(text).then(out => {
            out = out.trim();
            if (!l.open || l.query !== text)
                return;
            if (!out || out === text.trim()) {
                l.setFixState("ok", "Sem erros.");
                return;
            }
            l.fix = {from: text, to: out};
            const before = new Set(text.split(/\s+/));
            l.setFixState("fix", out.split(/(\s+)/).map(w => /\s/.test(w) || before.has(w) ? Busca.esc(w)
                : "<u><font color=\"#f2f2f2\">" + Busca.esc(w) + "</font></u>").join(""));
        }).catch(e => {
            console.log("clippit: corretor falhou: " + e.message);
            if (l.open && l.query === text)
                l.setFixState("error", "Não consegui verificar a escrita agora.");
        });
    }

    function applyFix() {
        if (!fix || query !== fix.from)
            return false;
        const to = fix.to;
        fix = null;
        setFixState("", "");
        setEntry(to, -1);
        focusEntry();
        return true;
    }

    // ---- Side panel (Alt+C): the clipboard next to the launcher --------------

    function toggleSide() {
        // The shortcut may come twice (key + D-Bus of clippit-historico).
        if (Date.now() - sideToggled < 400)
            return;
        sideToggled = Date.now();
        if (side)
            closeSide();
        else
            openSide();
    }

    function closeSide() {
        sideTx = null;
        side = false;
    }

    function openSide() {
        if (!open || side)
            return;
        side = true;
        sideLoading = true;
        sideItems = [];
        sideSel = 0;
        sideFoot = "";
        Sys.run(Sys.tool("clippit-busca").concat(["historico", ""]), null, 5).then(o => JSON.parse(o)).catch(e => {
            console.log("clippit: histórico falhou: " + e.message);
            return [];
        }).then(items => {
            if (!l.side)
                return;
            l.sideItems = items;
            l.sideSel = 0;
            l.sideLoading = false;
        });
    }

    function sideMove(d) {
        const n = sideTx ? Busca.TEXT_TRANSFORMS.length : sideItems.length;
        if (n)
            sideSel = Math.max(0, Math.min(n - 1, sideSel + d));
    }

    // Enter: into the launcher field (image: the text read, if any).
    // Ctrl+Enter (or an image without text): copies to the clipboard.
    function sideUse(copy) {
        const h = sideItems[sideSel];
        if (!h)
            return;
        if (h.tipo === "image" && h.imagem && view === "ia" && !copy) {
            closeSide();
            attachToken(null, {tipo: "imagem", nome: nextName("imagem"), caminho: h.imagem});
            focusEntry();
            return;
        }
        const text = h.tipo === "image" ? (h.ocr || "").trim() : h.texto;
        if (copy || !text) {
            Sys.run(Sys.tool("clippit-busca").concat(["copiar", String(h.id)]), null, 5)
                .then(() => l.sideFoot = h.tipo === "image" ? "Imagem copiada ✓" : "Copiado ✓")
                .catch(e => console.log("clippit: copiar falhou: " + e.message));
            return;
        }
        insertEntry(text);
        closeSide();
        focusEntry();
    }

    function sideApplyTx(tf) {
        const h = sideTx;
        const text = h && h.tipo === "image" ? (h.ocr || "").trim() : h ? h.texto || "" : "";
        if (!tf || !text)
            return;
        sideFoot = tf.title + "…";
        runTransform(tf, text).then(out => {
            if (!l.open)
                return;
            l.insertEntry(out);
            l.closeSide();
            l.focusEntry();
        }).catch(e => l.sideFoot = "Não deu: " + e.message);
    }

    function sideTab() {
        const h = sideItems[sideSel];
        sideTx = sideTx ? null : h || null;
        sideSel = sideTx ? 0 : Math.max(0, sideItems.indexOf(h));
    }

    // ---- AI chat ------------------------------------------------------------

    function pendingChoice() {
        const lastA = chat.slice().reverse().find(x => x.role === "assistant" && !x.pending);
        return lastA && lastA.choice && lastA.choiceAnswered === undefined && lastA === Compat.last(chat) ? lastA : null;
    }

    function updateMsg(m, patch) {
        const i = chat.indexOf(m);
        if (i < 0)
            return m;
        const n = Object.assign({}, m, patch);
        const list = chat.slice();
        list[i] = n;
        chat = list;
        return n;
    }

    function moveChoice(m, d) {
        updateMsg(m, {choiceSel: Math.max(0, Math.min(m.choice.opcoes.length - 1, (m.choiceSel || 0) + d))});
    }

    function answerChoice(m, i) {
        if (m.choiceAnswered !== undefined || busyChat)
            return;
        updateMsg(m, {choiceAnswered: i});
        send(m.choice.opcoes[i], {choiceIndex: i});
    }

    function confirm(m, run) {
        if (m.confirmState || busyChat)
            return;
        const n = updateMsg(m, {confirmState: run ? "ok" : "no"});
        if (!run)
            return;
        // Repeats the question that asked for the command, now authorized.
        const i = chat.indexOf(n);
        const asked = chat.slice(0, i).reverse().find(x => x.role === "user");
        send(asked ? asked.text : "", {confirmed: m.confirm, upTo: chat.indexOf(asked)});
    }

    readonly property int confirmGuardMs: 600

    // What the model may open: web pages and mail, or a file inside the home folder.
    // Its input includes web pages and OCR'd screens, so nothing else (smb://, custom handlers…).
    function safeUri(uri) {
        const u = String(uri || "");
        if (Busca.isWebUrl(u) || /^mailto:/i.test(u))
            return true;
        const home = Quickshell.env("HOME");
        return !!home && u.indexOf("file://" + home + "/") === 0 && u.indexOf("/../") < 0;
    }

    // Opens the app/site the AI asked for and gets out of the way.
    function runActions(actions) {
        let opened = false;
        for (const a of actions || []) {
            try {
                if (a.tipo === "app") {
                    const e = DesktopEntries.byId(a.id) || DesktopEntries.heuristicLookup(a.id);
                    if (e) {
                        Sys.launchApp(e);
                        opened = true;
                    }
                } else if (a.tipo === "janela" && a.id) {
                    const w = winList.find(x => String(x.id) === a.id);
                    if (w) {
                        prevWin = w;
                        Compositor.activate(w);
                        opened = true;
                    }
                } else if (a.tipo === "escrever" && a.texto) {
                    // Never typed into the window behind on the model's word alone.
                    Sys.copy(a.texto);
                    Sys.notify("Clippit", "Texto copiado: cole com Ctrl+V onde quiser.");
                    opened = true;
                } else if (a.tipo === "uri" && safeUri(a.uri)) {
                    const known = a.app && DesktopEntries.byId(a.app);
                    Sys.openUri(a.uri, known ? a.app : "");
                    opened = true;
                } else if (a.tipo === "uri") {
                    console.log("clippit: endereço recusado: " + String(a.uri).slice(0, 60));
                }
            } catch (e) {
                console.log("clippit: não abriu " + JSON.stringify(a) + ": " + e.message);
            }
        }
        return opened;
    }

    function newChat() {
        chat = [];
        chatId = "";
    }

    // A new conversation after 30 min without use (the old one stays in Ctrl+H).
    function enterChat() {
        if (chatTime && (Date.now() - chatTime) / 1000 > 1800 && !busyChat) {
            chat = [];
            chatId = "";
            chatTime = 0;
        }
        doctorInbox();
    }

    function send(question, opts) {
        opts = opts || {};
        if (busyChat)
            return;
        // Writing something instead of "Rodar" cancels the pending command.
        chat = chat.map(m => {
            const p = {};
            if (m.confirm && !m.confirmState && !opts.confirmed)
                p.confirmState = "no";
            if (m.choice && m.choiceAnswered === undefined)
                p.choiceAnswered = opts.choiceIndex !== undefined ? opts.choiceIndex : "texto";
            return Object.keys(p).length ? Object.assign({}, m, p) : m;
        });
        debounce.stop();
        const upTo = opts.confirmed ? Math.max(0, opts.upTo || 0) : chat.length;
        const history = chat.slice(0, upTo).filter(m => !m.pending && m.text).map(m => ({role: m.role, content: m.choice
            ? m.text + "\n" + m.choice.opcoes.map((o, i) => Busca.LETTERS[i] + ") " + o).join("\n") : m.text}));
        const asked = opts.confirmed ? chat[opts.upTo] : null;
        const attachments = opts.confirmed ? ((asked && asked.attachments) || []) : attach.filter(a => question.indexOf("@" + a.nome) >= 0);
        let list = chat.slice();
        if (!opts.confirmed) {
            attach = [];
            list.push({role: "user", text: question, attachments});
        }
        const replyId = "r" + Date.now();
        list.push({role: "assistant", pending: true, rid: replyId, live: {passos: [], pensando: true}, elapsed: 0});
        chat = list;
        chatTime = Date.now();
        if (!opts.confirmed)
            setEntry("", 0);
        const g = ++askGen;
        const started = Date.now();
        Sys.write(statusFile, "");
        livePoll.rid = replyId;
        livePoll.gen = g;
        livePoll.started = started;
        livePoll.start();
        const findReply = () => l.chat.find(m => m.rid === replyId);
        Sys.run(Sys.tool("clippit-correct").concat(["--perguntar"]), JSON.stringify({pergunta: question, historico: history,
            confirmado: opts.confirmed || null, anexos: attachments}), 240).then(raw => {
            raw = raw.trim();
            const r = findReply();
            if (!r || r.stopped)
                return;
            let data = {resposta: raw, passos: [], fontes: []};
            try {
                data = JSON.parse(raw);
            } catch (e) {}
            return Sys.readJson(l.statusFile, {}).then(last => {
                l.updateMsg(findReply(), {pending: false, text: data.resposta, steps: (last && last.passos) || [],
                    sources: data.fontes || [], outputs: data.saidas || [], confirm: data.confirmar || null,
                    choice: data.escolha || null, choiceSel: 0, answeredAt: Date.now()});
                l.saveChat();
                if (data.acoes && data.acoes.length && l.runActions(data.acoes)) {
                    l.chatTime = Date.now();
                    if (l.open)
                        l.close();
                }
            });
        }).catch(e => {
            const r = findReply();
            if (!r || r.stopped)
                return;
            l.updateMsg(r, {pending: false, text: "A IA não respondeu: " + e.message, steps: []});
            l.saveChat();
        }).then(() => {
            if (livePoll.rid === replyId)
                livePoll.stop();
            l.chatTime = Date.now();
        });
    }

    Timer {
        id: livePoll
        property string rid: ""
        property int gen: 0
        property double started: 0
        interval: 250
        repeat: true
        onTriggered: {
            const r = l.chat.find(m => m.rid === rid);
            if (gen !== l.askGen || !r || !r.pending) {
                stop();
                return;
            }
            Sys.read(l.statusFile).then(raw => {
                const cur = l.chat.find(m => m.rid === livePoll.rid);
                if (!cur || !cur.pending)
                    return;
                let live = cur.live;
                try {
                    if (raw.trim())
                        live = JSON.parse(raw.trim());
                } catch (e) {}
                l.updateMsg(cur, {live, elapsed: (Date.now() - livePoll.started) / 1000});
            }).catch(() => {});
        }
    }

    // Stop: tells the server (it stops at the next piece) and ends the reply now.
    function stopAsk() {
        const r = chat.find(m => m.pending);
        if (!r)
            return;
        Sys.write(Sys.runtimeDir + "/clippit-correct/cancelar", "1");
        askGen++;
        const live = r.live || {};
        const done = (live.passos || []).map(p => p.estado === "rodando" ? Object.assign({}, p, {estado: "erro", resumo: "parado"}) : p);
        updateMsg(r, {pending: false, stopped: true, text: live.texto ? live.texto + "\n\n*Parado.*" : "*Parado.*", steps: done});
        saveChat();
    }

    function chatEnter(ctrl) {
        const q = query.trim();
        const lastA = chat.slice().reverse().find(m => m.role === "assistant" && !m.pending);
        if (q) {
            send(q, {});
        } else if (lastA && lastA.confirm && !lastA.confirmState) {
            // An Enter pressed out of habit right as the command shows up must not run it.
            if (Date.now() - (lastA.answeredAt || 0) > confirmGuardMs)
                confirm(lastA, true);
        } else if (lastA && ctrl && lastA.sources && lastA.sources.length && Busca.isWebUrl(lastA.sources[0])) {
            Sys.openUri(lastA.sources[0]);
            close();
        } else if (lastA && lastA.text) {
            consumed = true;
            Sys.copy(lastA.text);
            close();
            Sys.notify("Clippit", "Resposta copiada.");
        }
    }

    // ---- D-Bus (com.clippit.Lancador, through clippit-wl) --------------------

    Connections {
        target: Ime
        function onCall(id, method, args) {
            l.dbus(id, method, args);
        }
    }

    function findWindows(alvo, all) {
        const a = (alvo || "").trim().toLowerCase();
        if (!a || ["atual", "ativa", "essa", "esta", "foco"].indexOf(a) >= 0)
            return prevWin ? [all.find(w => w.id === prevWin.id) || prevWin] : [];
        const byId = all.filter(w => String(w.id).toLowerCase() === a);
        if (byId.length)
            return byId;
        const tests = [n => n === a, n => n.startsWith(a), n => n.indexOf(a) >= 0];
        for (const t of tests) {
            const hit = all.filter(w => t(appName(w.app).toLowerCase()) || t((w.app || "").toLowerCase()));
            if (hit.length)
                return hit;
        }
        return all.filter(w => (w.title || "").toLowerCase().indexOf(a) >= 0);
    }

    function dbus(id, method, args) {
        const answer = v => Ime.reply(id, v);
        if (method === "Abrir") {
            const tela = args[0];
            toggleView(["historico", "ia"].indexOf(tela) >= 0 ? tela : "root");
            answer("");
        } else if (method === "Janelas") {
            Compositor.windows().then(ws => answer(JSON.stringify(ws.map(w => ({
                id: String(w.id), titulo: w.title || "", app: appName(w.app), area: w.workspace, monitor: (w.monitor || 0) + 1,
                atual: !!prevWin && w.id === prevWin.id, maximizada: !!w.maximized, minimizada: !!w.minimized, pid: w.pid,
            }))))).catch(() => answer("[]"));
        } else if (method === "Janela") {
            const acao = args[0], alvo = args[1];
            const ops = {focar: "focus", fechar: "close", maximizar: "max", restaurar: "restore", minimizar: "min",
                esquerda: "left", direita: "right", centralizar: "center", outro_monitor: "monitor", tela_cheia: "full"};
            const area = /^area_(\d+)$/.exec(acao || "");
            if (!ops[acao] && !area) {
                answer(JSON.stringify({erro: "ação desconhecida: " + acao}));
                return;
            }
            Compositor.windows().then(all => {
                const wins = findWindows(alvo, all);
                if (!wins.length) {
                    answer(JSON.stringify({erro: "nenhuma janela encontrada para \"" + alvo + "\""}));
                    return;
                }
                const names = wins.map(w => appName(w.app) + " — " + (w.title || ""));
                if (acao === "focar") {
                    answer(JSON.stringify({ok: true, focar: String(wins[0].id), janelas: names.slice(0, 1)}));
                    return;
                }
                if (wins.length > 1 && ["fechar", "tela_cheia", "minimizar"].indexOf(acao) >= 0) {
                    answer(JSON.stringify({erro: "ambíguo: " + wins.length + " janelas casam com \"" + alvo + "\"; peça ao usuário qual", janelas: names}));
                    return;
                }
                wins.forEach(w => Compositor.winOp(w, area ? acao : ops[acao]));
                answer(JSON.stringify({ok: true, janelas: names}));
            }).catch(e => answer(JSON.stringify({erro: e.message})));
        } else if (method === "Print") {
            const onde = args[0], caminho = args[1];
            if (caminho.indexOf(Sys.runtimeDir + "/") !== 0 || caminho.indexOf("..") >= 0) {
                answer("erro: o print só pode ser salvo em " + Sys.runtimeDir);
                return;
            }
            Compat.co(function* () {
                const w = l.prevWin || (yield Compositor.focused());
                const mons = yield Compositor.monitors();
                const rect = onde === "janela" && w && !w.minimized ? w : mons[w ? w.monitor : 0] || mons[0];
                l.hiddenForShot = true;
                yield Compat.sleep(180);
                try {
                    yield Compositor.shoot(rect, caminho);
                } finally {
                    l.hiddenForShot = false;
                }
                return onde === "janela" && w ? appName(w.app) + " — " + (w.title || "") : "tela";
            }).then(answer, e => answer("erro: " + e.message));
        } else if (method === "AreaHash") {
            // Only the fingerprint of what is copied (never the text).
            Sys.clipboard(false).then(t => t ? Sys.sha256(t) : "").then(answer, () => answer(""));
        } else {
            answer("");
        }
    }

    LancadorJanela {
        lancador: l
    }
}
