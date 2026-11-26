import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import "../components"
import "../services"
import ".."
import "../lib/busca.js" as Busca
import "../lib/compat.js" as Compat

// The launcher window (and the clipboard panel next to it).
Scope {
    id: root
    required property var lancador
    readonly property var l: lancador
    readonly property var mon: l.monitor || {x: 0, y: 0, width: 1920, height: 1080, name: ""}
    readonly property var scr: Quickshell.screens.find(s => s.name === mon.name) || Quickshell.screens[0]
    readonly property int panelW: Math.min(1040, mon.width - 40)
    readonly property int panelH: Math.min(640, mon.height - 60)
    readonly property int panelX: Math.round((mon.width - panelW) / 2)
    readonly property int panelY: Math.round((mon.height - panelH) * 0.38)
    readonly property int sideW: 360
    readonly property int sideX: {
        let x = panelX + panelW + 12;
        if (x + sideW > mon.width - 8)
            x = panelX - 12 - sideW;
        if (x < 8)
            x = mon.width - sideW - 8;
        return x;
    }

    Connections {
        target: l
        function onSetEntry(text, cursor) {
            entry.text = text;
            entry.cursorPosition = cursor < 0 ? text.length : cursor;
            l.onQuery(text, entry.cursorPosition);
        }
        function onInsertEntry(text) {
            const pos = entry.cursorPosition;
            entry.insert(pos, text);
        }
        function onFocusEntry() {
            entry.forceActiveFocus();
        }
        function onViewChanged() {
            if (l.view === "ia")
                l.enterChat();
        }
    }

    HyprlandFocusGrab {
        active: Compositor.kind === "hyprland" && win.visible && l.grabbed
        windows: l.side ? [win, sideWin] : [win]
        onCleared: l.release()
    }

    PanelWindow {
        id: win
        visible: l.open && !l.hiddenForShot
        screen: root.scr
        color: "transparent"
        exclusionMode: ExclusionMode.Ignore
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.namespace: "clippit"
        WlrLayershell.keyboardFocus: l.grabbed ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.OnDemand
        anchors.top: true
        anchors.left: true
        margins.left: root.panelX
        margins.top: root.panelY
        implicitWidth: root.panelW
        implicitHeight: root.panelH

        Item {
            anchors.fill: parent
            opacity: l.grabbed ? 1 : 0.78

            Glass {
                anchors.fill: parent
            }

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Search
                RowLayout {
                    Layout.fillWidth: true
                    Layout.topMargin: 14
                    Layout.bottomMargin: 12
                    Layout.leftMargin: Theme.pad
                    Layout.rightMargin: Theme.pad
                    spacing: 10
                    IconBtn {
                        visible: l.view !== "root" || l.isHistory
                        icon: "chevron-left"
                        normalColor: Theme.surface
                        onClicked: l.goBack()
                    }
                    TextField {
                        id: entry
                        Layout.fillWidth: true
                        background: null
                        padding: 2
                        color: Theme.text
                        selectionColor: Qt.rgba(76 / 255, 157 / 255, 1, 0.45)
                        selectedTextColor: "#ffffff"
                        font.family: Theme.font
                        font.pointSize: Theme.input
                        placeholderText: l.hint
                        placeholderTextColor: Theme.text3
                        onTextEdited: l.onQuery(text, cursorPosition)
                        onCursorPositionChanged: if (l.cursorPos !== cursorPosition) {
                            l.cursorPos = cursorPosition;
                            l.updateMention();
                        }
                        onActiveFocusChanged: if (activeFocus && l.open && !l.grabbed)
                            l.wake()
                        Keys.onPressed: event => root.onKey(event)
                    }
                    Txt {
                        visible: text !== ""
                        text: l.modeLabel
                        size: "tiny"
                    }
                }

                // Spell check of what you typed (AI screen) and dictation.
                RowLayout {
                    visible: l.fixState !== ""
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.pad
                    Layout.rightMargin: Theme.pad
                    Layout.bottomMargin: 10
                    spacing: 10
                    Ring {
                        visible: l.fixState === "checking"
                    }
                    Icon {
                        visible: l.fixState !== "checking"
                        name: l.fixState === "error" ? "triangle-alert" : l.fixState === "ok" ? "check" : l.fixState === "rec" ? "mic" : "spell-check"
                        tone: l.fixState === "error" ? "err" : l.fixState === "ok" ? "ok" : "g"
                        size: 14
                    }
                    Txt {
                        Layout.fillWidth: true
                        size: "small"
                        elide: Text.ElideRight
                        textFormat: l.fixState === "fix" ? Text.StyledText : Text.PlainText
                        text: l.fixMsg
                    }
                    Btn {
                        visible: l.fixState === "fix"
                        text: "Corrigir  Alt+S"
                        primary: true
                        onClicked: l.applyFix()
                    }
                }

                // "@": attach an app, file or screenshot.
                ColumnLayout {
                    visible: !!l.mention && (l.mention.items.length > 0 || l.mention.q.length >= 2)
                    Layout.fillWidth: true
                    Layout.leftMargin: 8
                    Layout.rightMargin: 8
                    Layout.bottomMargin: 8
                    spacing: 1
                    Txt {
                        visible: !!l.mention && !l.mention.items.length
                        text: "Procurando apps e arquivos…"
                        size: "small"
                        Layout.leftMargin: 10
                    }
                    Repeater {
                        model: l.mention ? l.mention.items : []
                        delegate: Rectangle {
                            id: mrow
                            required property var modelData
                            required property int index
                            Layout.fillWidth: true
                            implicitHeight: 34
                            radius: Theme.r3
                            color: index === l.mention.sel ? Theme.hover : "transparent"
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 10
                                anchors.rightMargin: 10
                                spacing: 12
                                Image {
                                    visible: !!mrow.modelData.appIcon
                                    source: mrow.modelData.appIcon || ""
                                    sourceSize.width: 40
                                    sourceSize.height: 40
                                    Layout.preferredWidth: 20
                                    Layout.preferredHeight: 20
                                }
                                Icon {
                                    visible: !mrow.modelData.appIcon
                                    name: mrow.modelData.lucide || "file"
                                    size: 18
                                }
                                Txt {
                                    text: mrow.modelData.nome
                                }
                                Txt {
                                    Layout.fillWidth: true
                                    text: mrow.modelData.sub || ""
                                    size: "small"
                                    elide: Text.ElideRight
                                }
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    l.mention = Object.assign({}, l.mention, {sel: mrow.index});
                                    l.mentionPick();
                                }
                            }
                        }
                    }
                }

                // Emoji categories
                Flow {
                    visible: l.view === "emoji"
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.pad
                    Layout.rightMargin: Theme.pad
                    Layout.bottomMargin: 10
                    spacing: 6
                    Repeater {
                        model: Busca.EMOJI_CATS
                        delegate: Rectangle {
                            id: chip
                            required property string modelData
                            readonly property bool on: !l.query.trim() && modelData === (l.emojiCat || "Sorrisos e emoções")
                            implicitWidth: chipText.implicitWidth + 20
                            implicitHeight: chipText.implicitHeight + 8
                            radius: Theme.r2
                            color: on ? Theme.accentSoft : Theme.surface
                            Txt {
                                id: chipText
                                anchors.centerIn: parent
                                text: chip.modelData
                                size: "small"
                                strong: chip.on
                                color: chip.on ? Theme.accent : Theme.text2
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    l.emojiCat = chip.modelData;
                                    l.emojiSel = 0;
                                    l.setEntry("", 0);
                                }
                            }
                        }
                    }
                }

                // Body
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true

                    // List (+ history preview)
                    RowLayout {
                        anchors.fill: parent
                        spacing: 0
                        visible: l.view !== "ia" && l.view !== "emoji" && !(l.view === "texto" && l.tx)

                        ListView {
                            id: list
                            visible: !(l.isHistory && l.zoom)
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            Layout.leftMargin: 8
                            Layout.rightMargin: 8
                            Layout.bottomMargin: 6
                            clip: true
                            spacing: 1
                            model: l.shown
                            currentIndex: l.sel
                            highlightMoveDuration: 0
                            boundsBehavior: Flickable.StopAtBounds
                            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
                            section.property: l.isHistory ? "" : "section"
                            section.delegate: Txt {
                                required property string section
                                text: section
                                size: "tiny"
                                strong: true
                                leftPadding: 10
                                topPadding: 10
                                bottomPadding: 5
                            }
                            delegate: Row_ {
                                launcher: root.lancador
                                width: list.width
                            }
                            EmptyState {
                                anchors.centerIn: parent
                                visible: l.shown.length === 0
                                icon: "search"
                                title: "Nada encontrado"
                                sub: "Tente outra palavra."
                            }
                        }

                        Rectangle {
                            visible: l.isHistory && !l.zoom
                            Layout.fillHeight: true
                            width: 1
                            color: Theme.line
                        }

                        Detalhe {
                            visible: l.isHistory
                            Layout.fillHeight: true
                            Layout.preferredWidth: l.zoom ? root.panelW : Math.round(root.panelW * 0.6)
                            item: l.shown[l.sel] ? l.shown[l.sel].data : null
                            zoom: l.zoom
                        }
                    }

                    Chat {
                        anchors.fill: parent
                        visible: l.view === "ia"
                        launcher: root.lancador
                    }

                    // Text action result (Alt+A)
                    Flickable {
                        anchors.fill: parent
                        anchors.leftMargin: Theme.pad
                        anchors.rightMargin: Theme.pad
                        visible: l.view === "texto" && !!l.tx
                        contentHeight: txCol.implicitHeight + 24
                        clip: true
                        ColumnLayout {
                            id: txCol
                            width: parent.width
                            spacing: 14
                            RowLayout {
                                spacing: 8
                                Ring {
                                    visible: !!l.tx && l.tx.pending
                                }
                                Txt {
                                    text: l.tx ? l.tx.title + (l.tx.pending ? "…" : "") : ""
                                    size: "small"
                                    strong: true
                                    color: Theme.text
                                }
                            }
                            Text {
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                textFormat: Text.RichText
                                color: Theme.text
                                font.family: Theme.font
                                font.pointSize: Theme.body
                                text: !l.tx ? "" : l.tx.error ? Busca.esc("Não consegui: " + l.tx.error)
                                    : Busca.mdToHtml(l.tx.pending ? l.tx.live : l.tx.text)
                            }
                        }
                    }

                    EmojiGrade {
                        id: grade
                        anchors.fill: parent
                        visible: l.view === "emoji"
                        launcher: root.lancador
                        query: l.query
                    }
                }

                Footer {
                    Layout.fillWidth: true
                    where: root.footer().where
                    icon: root.footer().icon
                    actions: root.footer().actions
                }
            }

            MouseArea {
                anchors.fill: parent
                visible: !l.grabbed
                onPressed: mouse => {
                    l.wake();
                    mouse.accepted = false;
                }
            }
        }
    }

    // ---- Side panel: the clipboard (Alt+C) ---------------------------------
    PanelWindow {
        id: sideWin
        visible: l.open && l.side && !l.hiddenForShot
        screen: root.scr
        color: "transparent"
        exclusionMode: ExclusionMode.Ignore
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.namespace: "clippit"
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        anchors.top: true
        anchors.left: true
        margins.left: root.sideX
        margins.top: root.panelY
        implicitWidth: root.sideW
        implicitHeight: root.panelH

        Item {
            anchors.fill: parent
            opacity: l.grabbed ? 1 : 0.78
            Glass {
                anchors.fill: parent
            }
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                RowLayout {
                    Layout.fillWidth: true
                    Layout.margins: 14
                    Layout.leftMargin: Theme.pad
                    Layout.rightMargin: Theme.pad
                    spacing: 10
                    Icon {
                        name: "clipboard-list"
                        size: 14
                    }
                    Txt {
                        Layout.fillWidth: true
                        text: "Área de transferência"
                        strong: true
                    }
                    Keycap {
                        text: "Alt C"
                    }
                }
                Txt {
                    visible: !!l.sideTx
                    Layout.fillWidth: true
                    Layout.leftMargin: 18
                    Layout.rightMargin: 18
                    Layout.bottomMargin: 6
                    size: "small"
                    elide: Text.ElideRight
                    text: l.sideTx ? "Transformar: " + ((l.sideTx.tipo === "image" ? l.sideTx.ocr : l.sideTx.texto) || "").replace(/\s+/g, " ").slice(0, 50) : ""
                }
                ListView {
                    id: sideList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.leftMargin: 8
                    Layout.rightMargin: 8
                    clip: true
                    spacing: 1
                    model: l.sideTx ? Busca.TEXT_TRANSFORMS : l.sideItems
                    currentIndex: l.sideSel
                    onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
                    boundsBehavior: Flickable.StopAtBounds
                    delegate: Rectangle {
                        id: srow
                        required property var modelData
                        required property int index
                        readonly property bool isImg: !l.sideTx && modelData.tipo === "image" && !!modelData.imagem
                        width: sideList.width
                        implicitHeight: Math.max(40, scol.implicitHeight + 14)
                        radius: Theme.r3
                        color: index === l.sideSel ? Theme.hover : "transparent"
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 12
                            Image {
                                visible: srow.isImg
                                source: srow.isImg ? "file://" + srow.modelData.imagem : ""
                                sourceSize.width: 72
                                sourceSize.height: 72
                                fillMode: Image.PreserveAspectCrop
                                Layout.preferredWidth: 36
                                Layout.preferredHeight: 36
                            }
                            Icon {
                                visible: !srow.isImg
                                size: 16
                                name: l.sideTx ? (Busca.ICON_MAP[srow.modelData.icon] || "wand-sparkles")
                                    : srow.modelData.fixado ? "pin" : /^https?:\/\//.test(srow.modelData.texto || "") ? "link" : "type"
                                tone: !l.sideTx && srow.modelData.fixado ? "acc" : "g"
                            }
                            ColumnLayout {
                                id: scol
                                Layout.fillWidth: true
                                spacing: 0
                                Txt {
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                    text: {
                                        const h = srow.modelData;
                                        if (l.sideTx)
                                            return h.title;
                                        const ocr = (h.ocr || "").replace(/\s+/g, " ").trim();
                                        const t = srow.isImg ? (ocr || "Imagem") : (h.texto || "").replace(/\s+/g, " ").trim();
                                        return t.slice(0, 160) || "(vazio)";
                                    }
                                }
                                Txt {
                                    visible: text !== ""
                                    size: "small"
                                    text: {
                                        const h = srow.modelData;
                                        if (l.sideTx)
                                            return h.ai ? "IA" : "";
                                        const ocr = (h.ocr || "").trim();
                                        return (h.fixado ? "Fixado · " : "") + (srow.isImg ? (l.view === "ia" ? "Imagem · anexa para a IA ver"
                                            : ocr ? "Imagem · cola o texto lido" : "Imagem · copia a imagem")
                                            : /^https?:\/\//.test(h.texto || "") ? "Link" : (h.texto || "").length + " caracteres");
                                    }
                                }
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: mouse => {
                                l.wake();
                                l.sideSel = srow.index;
                                if (l.sideTx)
                                    l.sideApplyTx(Busca.TEXT_TRANSFORMS[srow.index]);
                                else
                                    l.sideUse((mouse.modifiers & Qt.ControlModifier) !== 0);
                            }
                        }
                    }
                    EmptyState {
                        anchors.centerIn: parent
                        visible: !l.sideTx && !l.sideLoading && l.sideItems.length === 0
                        icon: "clipboard-list"
                        title: "Nada copiado ainda"
                        sub: "O que você copiar aparece aqui."
                    }
                    Txt {
                        visible: l.sideLoading
                        text: "Carregando…"
                        size: "small"
                        padding: 12
                    }
                }
                Footer {
                    Layout.fillWidth: true
                    where: l.sideFoot
                    icon: "clipboard-list"
                    actions: l.sideFoot ? [] : l.sideTx ? [{label: "Colar no campo", key: "↵"}, {label: "Voltar", key: "Tab"}]
                        : [{label: "Colar", key: "↵"}, {label: "Transformar", key: "Tab"}, {label: "Fixar", key: "Ctrl P"}]
                }
            }
        }
    }

    // ---- Footer per screen --------------------------------------------------
    function footer() {
        const it = l.shown[l.sel];
        if (l.view === "ia") {
            const lastA = l.chat.slice().reverse().find(m => m.role === "assistant" && !m.pending);
            const asking = lastA && lastA.confirm && !lastA.confirmState;
            return {where: "IA local", icon: "sparkles", actions: l.busyChat ? [{label: "Parar", key: "Esc", run: () => l.stopAsk()}]
                : asking ? [{label: "Rodar", key: "↵"}, {label: "Cancelar", key: "Esc"}]
                : [{label: "Enviar", key: "↵"}, lastA && lastA.sources && lastA.sources.length ? {label: "Abrir fonte", key: "Ctrl ↵"} : null,
                    {label: "Nova", key: "Ctrl N"}, {label: "Conversas", key: "Ctrl H"}, {label: "Copiados", key: "Alt C"}]};
        }
        if (l.view === "texto" && l.tx)
            return {where: "Texto selecionado", icon: "wand-sparkles", actions: l.tx.pending ? [{label: "Escrevendo…"}]
                : [{label: "Colar no lugar", key: "↵"}, {label: "Copiar", key: "Ctrl ↵"}, {label: "Voltar", key: "⌫"}]};
        if (l.view === "emoji") {
            const cur = emojiGridCurrent();
            if (!cur)
                return {where: "Emoji", icon: "smile", actions: []};
            const pinned = l.emojiPrefs.fixados.indexOf(cur.e) >= 0;
            return {where: cur.e + "  " + cur.desc, icon: "smile", actions: [{label: "Colar", key: "↵"}, {label: "Copiar", key: "Ctrl ↵"},
                {label: pinned ? "Desafixar" : "Fixar", key: "Ctrl P"}]};
        }
        if (l.isHistory && l.cbTx)
            return {where: "Transformar", icon: "clipboard-list", actions: [{label: "Colar", key: "↵"}, {label: "Copiar", key: "Ctrl ↵"},
                {label: "Voltar", key: "Tab"}]};
        if (l.isHistory)
            return {where: "Histórico", icon: "clipboard-list", actions: [{label: "Colar", key: "↵"}, {label: "Copiar", key: "Ctrl ↵"},
                {label: "Transformar", key: "Tab"}, {label: "Fixar", key: "Ctrl P"}, {label: "Apagar", key: "Ctrl Del"},
                {label: "Ampliar", key: "Ctrl Espaço"}]};
        if (l.view === "conversas")
            return {where: "Conversas", icon: "messages-square", actions: it ? [{label: "Abrir", key: "↵"}, {label: "Apagar", key: "Ctrl Del"}] : []};
        if (it)
            return {where: l.view === "texto" ? "Texto selecionado" : "Clippit", icon: l.view === "texto" ? "wand-sparkles" : "search",
                actions: [{label: it.hint || "Abrir", key: "↵"}, it.alt ? {label: "Ação extra", key: "Ctrl ↵"} : null,
                    l.view === "texto" ? {label: "Voltar", key: "⌫"} : null]};
        return {where: "Clippit", icon: "search", actions: []};
    }

    readonly property var emojiFlat: grade.flat
    readonly property var emojiRows: grade.rows
    function emojiGridCurrent() {
        return emojiFlat[l.emojiSel] || null;
    }

    // ---- Keys (the same as the GNOME extension) -----------------------------
    function onKey(event) {
        const k = event.key;
        if (Quickshell.env("CLIPPIT_DEBUG"))
            console.log("clippit: tecla " + k + " mods " + event.modifiers);
        const ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
        const alt = (event.modifiers & Qt.AltModifier) !== 0;
        const shift = (event.modifiers & Qt.ShiftModifier) !== 0;
        const enter = k === Qt.Key_Return || k === Qt.Key_Enter;
        const stop = () => {
            event.accepted = true;
        };
        if (alt && k === Qt.Key_V) {
            if (l.view !== "ia")
                l.enterView("ia");
            l.toggleDictation();
            return stop();
        }
        if (alt && k === Qt.Key_C) {
            if (l.isHistory)
                l.close();
            else
                l.toggleSide();
            return stop();
        }
        if (l.mention) {
            if (k === Qt.Key_Down || k === Qt.Key_Up) {
                l.mentionMove(k === Qt.Key_Down ? 1 : -1);
                return stop();
            }
            if ((enter || k === Qt.Key_Tab) && l.mention.items.length) {
                l.mentionPick();
                return stop();
            }
            if (k === Qt.Key_Escape) {
                l.mention = null;
                return stop();
            }
        }
        if (l.side) {
            const moves = {[Qt.Key_Down]: 1, [Qt.Key_Up]: -1, [Qt.Key_PageDown]: 5, [Qt.Key_PageUp]: -5};
            if (moves[k] !== undefined) {
                l.sideMove(moves[k]);
                return stop();
            }
            if (enter) {
                if (l.sideTx)
                    l.sideApplyTx(Busca.TEXT_TRANSFORMS[l.sideSel]);
                else
                    l.sideUse(ctrl);
                return stop();
            }
            if (k === Qt.Key_Tab) {
                l.sideTab();
                return stop();
            }
            if (ctrl && k === Qt.Key_P && !l.sideTx) {
                const h = l.sideItems[l.sideSel];
                if (h)
                    l.togglePin(h);
                return stop();
            }
        }
        if (l.fix && ((alt && (k === Qt.Key_S || k === Qt.Key_O)) || k === Qt.Key_Tab || k === Qt.Key_Backtab) && l.applyFix())
            return stop();
        if (l.isHistory && k === Qt.Key_Tab) {
            const h = l.shown[l.sel] ? l.shown[l.sel].data : null;
            l.cbTx = l.cbTx ? null : h && h.tipo ? h : null;
            l.sel = 0;
            l.refresh(false);
            return stop();
        }
        if (l.isHistory && l.cbTx && k === Qt.Key_Backspace && !entry.text) {
            l.cbTx = null;
            l.refresh(false);
            return stop();
        }
        if (l.isHistory && ctrl && k === Qt.Key_P) {
            const h = l.cbTx || (l.shown[l.sel] ? l.shown[l.sel].data : null);
            if (h && h.tipo)
                l.togglePin(h);
            return stop();
        }
        if (l.isHistory && ctrl && k === Qt.Key_Delete) {
            l.deleteHistory(l.shown[l.sel]);
            return stop();
        }
        if (l.isHistory && ctrl && k === Qt.Key_Space) {
            l.zoom = !l.zoom;
            return stop();
        }
        if (l.view === "texto" && l.tx) {
            if (enter) {
                l.txKey(true, ctrl);
                return stop();
            }
            if (k === Qt.Key_Backspace && !entry.text) {
                l.tx = null;
                l.refresh(false);
                return stop();
            }
        }
        if (k === Qt.Key_Backspace && !entry.text && l.view !== "root") {
            l.enterView("root");
            return stop();
        }
        if (l.view === "ia" && ctrl && k === Qt.Key_H) {
            l.convs = null;
            l.enterView("conversas");
            return stop();
        }
        if (l.view === "conversas" && ctrl && k === Qt.Key_Delete) {
            l.deleteConv(l.shown[l.sel]);
            return stop();
        }
        if (l.view === "ia" && ctrl && k === Qt.Key_N) {
            l.newChat();
            return stop();
        }
        if (k === Qt.Key_Escape) {
            // AI answering: the first Esc only stops the answer.
            if (l.view === "ia" && l.busyChat) {
                l.stopAsk();
                return stop();
            }
            l.close();
            return stop();
        }
        if (l.view === "emoji" && emojiKey(k, ctrl, shift, enter))
            return stop();
        if (l.view === "ia" && (k === Qt.Key_Down || k === Qt.Key_Up) && !entry.text.trim() && l.pendingChoice()) {
            l.moveChoice(l.pendingChoice(), k === Qt.Key_Down ? 1 : -1);
            return stop();
        }
        if (k === Qt.Key_Down || k === Qt.Key_Up || k === Qt.Key_PageDown || k === Qt.Key_PageUp) {
            const step = k === Qt.Key_Down ? 1 : k === Qt.Key_Up ? -1 : k === Qt.Key_PageDown ? 5 : -5;
            if (l.shown.length)
                l.sel = Math.max(0, Math.min(l.shown.length - 1, l.sel + step));
            return stop();
        }
        if (enter) {
            if (l.view === "ia" && !entry.text.trim() && l.pendingChoice()) {
                const m = l.pendingChoice();
                l.answerChoice(m, m.choiceSel || 0);
                return stop();
            }
            if (l.view === "ia") {
                l.chatEnter(ctrl);
                return stop();
            }
            l.activate(l.shown[l.sel], ctrl);
            return stop();
        }
    }

    function emojiKey(k, ctrl, shift, enter) {
        const flat = emojiFlat;
        const cur = flat[l.emojiSel];
        const moveRow = d => {
            if (!cur)
                return;
            const r = Math.max(0, Math.min(emojiRows.length - 1, cur.row + d));
            const row = emojiRows[r];
            l.emojiSel = row[Math.min(cur.col, row.length - 1)];
        };
        switch (k) {
        case Qt.Key_Left:
            l.emojiSel = Math.max(0, l.emojiSel - 1);
            return true;
        case Qt.Key_Right:
            l.emojiSel = Math.min(flat.length - 1, l.emojiSel + 1);
            return true;
        case Qt.Key_Up:
            moveRow(-1);
            return true;
        case Qt.Key_Down:
            moveRow(1);
            return true;
        case Qt.Key_Tab:
        case Qt.Key_Backtab:
            l.nextEmojiCat(shift || k === Qt.Key_Backtab ? -1 : 1);
            return true;
        case Qt.Key_P:
            if (!ctrl || !cur)
                return false;
            l.emojiPin(cur);
            return true;
        }
        if (enter) {
            l.emojiUse(cur, ctrl);
            return true;
        }
        return false;
    }
}
