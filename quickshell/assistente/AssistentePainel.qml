import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell
import Quickshell.Wayland
import Quickshell.Hyprland
import "../components"
import "../services"
import ".."

// The assistant's panel: options (Curta / Normal / Completa) you can edit,
// context, where the reply came from, the adjust field and the context editor.
PanelWindow {
    id: win
    required property var assistente
    readonly property var as_: assistente
    readonly property var w: as_.win
    readonly property var scr: {
        const p = w ? {x: w.x + w.width / 2, y: w.y + w.height / 2} : {x: 0, y: 0};
        return Quickshell.screens.find(s => p.x >= s.x && p.x < s.x + s.width && p.y >= s.y && p.y < s.y + s.height)
            || Quickshell.screens[0];
    }

    visible: as_.open
    screen: scr
    color: "transparent"
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.namespace: "clippit"
    // Takes the keyboard only after a click on a text field (or Alt+F).
    WlrLayershell.keyboardFocus: as_.grabbed || as_.editing !== undefined ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None
    anchors.top: true
    anchors.left: true
    implicitWidth: as_.width_
    implicitHeight: Math.min(box.implicitHeight, scr.height - 40)
    // Centered at the bottom of the conversation window, above it.
    margins.left: {
        const x = w ? w.x - scr.x + w.width / 2 - as_.width_ / 2 : 100;
        return Math.max(12, Math.min(x, scr.width - as_.width_ - 12));
    }
    margins.top: {
        const y = w ? w.y - scr.y + w.height - 80 : scr.height - 120;
        return Math.max(12, y - implicitHeight - 12);
    }

    // With the keyboard on the panel, a click outside closes it.
    HyprlandFocusGrab {
        active: Compositor.kind === "hyprland" && win.visible && as_.grabbed
        windows: [win]
        onCleared: as_.close(true)
    }

    Connections {
        target: as_
        function onFocusEntry() {
            entry.forceActiveFocus();
        }
    }

    Glass {
        anchors.fill: parent
    }

    ColumnLayout {
        id: box
        anchors.fill: parent
        spacing: 0

        // Header
        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 14
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            spacing: 6
            Txt {
                text: "Resposta sugerida"
                strong: true
            }
            Rectangle {
                implicitWidth: ctxLabel.implicitWidth + 12
                implicitHeight: ctxLabel.implicitHeight + 2
                radius: Theme.r1
                color: ctxMouse.containsMouse ? Qt.rgba(76 / 255, 157 / 255, 1, 0.32) : Theme.accentSoft
                Layout.leftMargin: 6
                Txt {
                    id: ctxLabel
                    anchors.centerIn: parent
                    size: "tiny"
                    strong: true
                    color: ctxMouse.containsMouse ? "#ffffff" : Theme.accent
                    text: ((as_.contexts[as_.ctxIndex] || {}).name || "Sem contexto") + " ▾"
                }
                MouseArea {
                    id: ctxMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: as_.showCtxList = !as_.showCtxList
                }
            }
            Item {
                Layout.fillWidth: true
            }
            IconBtn {
                icon: "folder"
                onClicked: as_.openContextDir()
            }
            IconBtn {
                icon: "square-x"
                onClicked: as_.close(true)
            }
        }

        // Context list
        Rectangle {
            visible: as_.showCtxList && as_.editing === undefined
            Layout.fillWidth: true
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            Layout.bottomMargin: 10
            implicitHeight: ctxCol.implicitHeight + 8
            radius: Theme.r3
            color: Theme.surface
            ColumnLayout {
                id: ctxCol
                anchors.fill: parent
                anchors.margins: 4
                spacing: 2
                Repeater {
                    model: as_.contexts
                    delegate: Rectangle {
                        id: ctxRow
                        required property var modelData
                        required property int index
                        readonly property bool on: index === as_.ctxIndex
                        Layout.fillWidth: true
                        implicitHeight: ctxRowLay.implicitHeight + 12
                        radius: Theme.r2
                        color: on ? Theme.accentSoft : rowMouse.containsMouse ? Theme.hover : "transparent"
                        MouseArea {
                            id: rowMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: as_.chooseContext(ctxRow.index)
                        }
                        RowLayout {
                            id: ctxRowLay
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 6
                            spacing: 8
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0
                                Txt {
                                    text: (ctxRow.on ? "✓ " : "") + ctxRow.modelData.name
                                    strong: true
                                    color: ctxRow.on ? Theme.accent : Theme.text
                                }
                                Txt {
                                    size: "tiny"
                                    text: ctxRow.modelData.file ? (ctxRow.modelData.apps.length ? "Escolhido sozinho em: "
                                        + ctxRow.modelData.apps.join(", ") : "Escolha manual") : "Responde sem instruções extras"
                                }
                            }
                            IconBtn {
                                visible: !!ctxRow.modelData.file
                                icon: "pencil"
                                normalColor: Theme.hover
                                onClicked: win.openEditor(ctxRow.modelData)
                            }
                        }
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    Layout.margins: 2
                    Layout.topMargin: 8
                    spacing: 6
                    Btn {
                        text: "+ Novo contexto"
                        onClicked: win.openEditor(null)
                    }
                    Item {
                        Layout.fillWidth: true
                    }
                    Btn {
                        text: "Abrir pasta"
                        onClicked: as_.openContextDir()
                    }
                }
            }
        }

        // Body: status, options, hints
        ColumnLayout {
            visible: as_.editing === undefined
            Layout.fillWidth: true
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            spacing: 8
            RowLayout {
                visible: as_.status !== ""
                spacing: 8
                Ring {
                    visible: as_.busy
                }
                Txt {
                    text: as_.status
                    size: "small"
                }
            }
            // A count: editing a reply reassigns `as_.options`, which would rebuild the
            // delegates and drop the focus of the field being typed in.
            Repeater {
                model: as_.options.length
                delegate: Rectangle {
                    id: opt
                    required property int index
                    readonly property string modelData: as_.options[index]
                    Layout.fillWidth: true
                    implicitHeight: optLay.implicitHeight + 16
                    radius: Theme.r3
                    color: index === as_.sel ? Theme.accentSoft : Theme.surface
                    opacity: as_.busy ? 0.45 : 1
                    MouseArea {
                        anchors.fill: parent
                        onClicked: as_.select(opt.index)
                    }
                    RowLayout {
                        id: optLay
                        anchors.fill: parent
                        anchors.margins: 8
                        anchors.leftMargin: 10
                        spacing: 10
                        ColumnLayout {
                            Layout.alignment: Qt.AlignTop
                            spacing: 2
                            Keycap {
                                visible: as_.options.length > 1
                                text: String(opt.index + 1)
                                Layout.alignment: Qt.AlignHCenter
                            }
                            Txt {
                                visible: as_.options.length >= 3
                                text: ["Curta", "Normal", "Completa"][opt.index] || ""
                                size: "tiny"
                                strong: true
                            }
                        }
                        TextArea {
                            id: optText
                            Layout.fillWidth: true
                            text: opt.modelData
                            wrapMode: TextEdit.Wrap
                            color: Theme.text
                            selectionColor: Qt.rgba(76 / 255, 157 / 255, 1, 0.45)
                            font.family: Theme.font
                            font.pointSize: Theme.body
                            background: null
                            padding: 0
                            onTextChanged: if (activeFocus)
                                as_.setOption(opt.index, text)
                            onActiveFocusChanged: if (activeFocus) {
                                as_.select(opt.index);
                                if (!as_.grabbed)
                                    as_.takeKeyboard();
                                optText.forceActiveFocus();
                            }
                            Keys.onEscapePressed: {
                                as_.releaseKeyboard();
                                optText.focus = false;
                            }
                        }
                    }
                }
            }
            Txt {
                visible: as_.options.length > 0 && !as_.busy
                text: "Escolha com 1, 2 ou 3. Clique no texto para editar; Esc sai da edição."
                size: "small"
                font.italic: true
            }
            Txt {
                visible: as_.note !== ""
                text: as_.note
                size: "small"
                font.italic: true
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        // Actions
        RowLayout {
            visible: as_.editing === undefined
            Layout.leftMargin: Theme.pad
            Layout.topMargin: 10
            spacing: 6
            opacity: as_.busy ? 0.47 : 1
            Btn {
                text: "Inserir"
                primary: true
                onClicked: as_.insert()
            }
            Btn {
                text: "Outra"
                onClicked: as_.regenerate(0.8)
            }
            Btn {
                text: "Copiar"
                onClicked: as_.copy()
            }
        }

        // Where the reply came from
        ColumnLayout {
            visible: as_.editing === undefined
            Layout.fillWidth: true
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            Layout.topMargin: 10
            spacing: 6
            Txt {
                text: (as_.showSources ? "▾" : "▸") + "  De onde veio esta resposta"
                size: "small"
                color: srcMouse.containsMouse ? Theme.text : Theme.text2
                MouseArea {
                    id: srcMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        as_.showSources = !as_.showSources;
                        as_.refreshNotes();
                    }
                }
            }
            Rectangle {
                visible: as_.showSources
                Layout.fillWidth: true
                implicitHeight: srcCol.implicitHeight + 16
                radius: Theme.r3
                color: Theme.code
                ColumnLayout {
                    id: srcCol
                    anchors.fill: parent
                    anchors.margins: 8
                    anchors.leftMargin: 10
                    spacing: 8
                    Txt {
                        Layout.fillWidth: true
                        text: as_.showSources ? as_.sourcesSummary() : ""
                        size: "small"
                        color: Theme.text
                        wrapMode: Text.WrapAnywhere
                    }
                    Flickable {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.min(srcText.implicitHeight, 220)
                        contentHeight: srcText.implicitHeight
                        clip: true
                        Text {
                            id: srcText
                            width: parent.width
                            text: (as_.conversation || "").trim() || "(nada)"
                            textFormat: Text.PlainText
                            font.family: "monospace"
                            font.pointSize: Theme.small
                            color: "#c8c8cc"
                            wrapMode: Text.WrapAnywhere
                        }
                    }
                }
            }
        }

        // Adjust field
        Rectangle {
            visible: as_.editing === undefined
            Layout.fillWidth: true
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            Layout.topMargin: 10
            implicitHeight: 32
            radius: Theme.r3
            color: Theme.surface
            TextField {
                id: entry
                anchors.fill: parent
                leftPadding: 10
                rightPadding: 10
                placeholderText: "Ajuste (\"mais curta\"), regra (\"regra: …\") ou anotação sobre a pessoa (\"nota: …\")"
                placeholderTextColor: Theme.text3
                color: Theme.text
                font.family: Theme.font
                font.pointSize: Theme.body
                background: null
                onActiveFocusChanged: if (activeFocus && !as_.grabbed)
                    as_.takeKeyboard()
                onAccepted: {
                    const t = text;
                    text = "";
                    as_.sendAsk(t);
                }
                Keys.onEscapePressed: as_.close(true)
                Keys.onPressed: event => {
                    if ((event.key === Qt.Key_F) && (event.modifiers & Qt.AltModifier)) {
                        as_.releaseKeyboard();
                        entry.focus = false;
                        event.accepted = true;
                    }
                }
            }
        }

        Footer {
            visible: as_.editing === undefined
            Layout.fillWidth: true
            where: "Assistente"
            icon: "sparkles"
            actions: [{label: "Inserir", key: "↵"}, {label: "Ajustar", key: Conf.label("ajuste")}, {label: "Outra", key: Conf.label("assistente")}, {label: "Fechar", key: "Esc"}]
        }

        // Context editor
        ColumnLayout {
            visible: as_.editing !== undefined
            Layout.fillWidth: true
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            spacing: 6
            Txt {
                text: as_.editing ? "NOME DO CONTEXTO" : "NOME DO NOVO CONTEXTO"
                size: "tiny"
                strong: true
                Layout.topMargin: 4
            }
            EditField {
                id: edName
                placeholder: "Ex.: Trovana – atendimento"
            }
            Txt {
                text: "ESCOLHER SOZINHO NESTES APPS (opcional)"
                size: "tiny"
                strong: true
                Layout.topMargin: 4
            }
            EditField {
                id: edApps
                placeholder: "whatsapp, instagram, slack"
            }
            Txt {
                text: "INSTRUÇÕES, INFORMAÇÕES E REGRAS"
                size: "tiny"
                strong: true
                Layout.topMargin: 4
            }
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: Math.max(180, edText.implicitHeight + 16)
                radius: Theme.r3
                color: Theme.surface
                TextArea {
                    id: edText
                    anchors.fill: parent
                    anchors.margins: 8
                    wrapMode: TextEdit.Wrap
                    color: Theme.text
                    font.family: Theme.font
                    font.pointSize: Theme.body
                    background: null
                    padding: 0
                    placeholderText: "Quem você é, o tom, e os fatos que a IA pode usar (frete, prazos, preços)…"
                    placeholderTextColor: Theme.text3
                    Keys.onEscapePressed: win.closeEditor()
                }
            }
            Txt {
                text: "Enter quebra linha. Uma regra por linha começando com \"- \" funciona bem."
                size: "tiny"
            }
            RowLayout {
                Layout.topMargin: 8
                Layout.bottomMargin: 14
                spacing: 6
                Btn {
                    text: "Salvar"
                    primary: true
                    onClicked: if (!as_.saveEditor(edName.text, edApps.text, edText.text))
                        edName.placeholder = "Dê um nome ao contexto"
                }
                Btn {
                    text: "Voltar"
                    onClicked: win.closeEditor()
                }
            }
        }
    }

    component EditField: Rectangle {
        property alias text: field.text
        property alias placeholder: field.placeholderText
        Layout.fillWidth: true
        implicitHeight: 32
        radius: Theme.r3
        color: Theme.surface
        TextField {
            id: field
            anchors.fill: parent
            leftPadding: 10
            color: Theme.text
            font.family: Theme.font
            font.pointSize: Theme.body
            placeholderTextColor: Theme.text3
            background: null
            Keys.onEscapePressed: win.closeEditor()
        }
    }

    function openEditor(ctx) {
        as_.showCtxList = false;
        as_.editing = ctx;
        edName.text = ctx ? ctx.name : "";
        edApps.text = ctx ? ctx.apps.join(", ") : "";
        edText.text = ctx ? ctx.raw : "";
        Qt.callLater(() => (ctx ? edText : edName.children[0]).forceActiveFocus());
    }

    function closeEditor() {
        as_.editing = undefined;
        as_.releaseKeyboard();
    }
}
