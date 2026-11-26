import QtQuick
import QtQuick.Layouts
import "../components"
import ".."
import "../lib/busca.js" as Busca

// Conversation with the AI: your questions, and the answers with the steps
// (tools that ran), command outputs, "Rodar" confirmations and A/B/C choices.
Item {
    id: chat
    required property var launcher
    readonly property var l: launcher

    EmptyState {
        anchors.centerIn: parent
        visible: l.chat.length === 0
        icon: "sparkles"
        title: "Pergunte qualquer coisa"
        sub: "A IA vê o seu computador, cotações, clima, notícias e a web, e lembra da conversa.\n@ anexa app, arquivo ou print · Alt+V para falar"
    }

    Flickable {
        id: flick
        anchors.fill: parent
        visible: l.chat.length > 0
        clip: true
        contentHeight: col.implicitHeight + 24
        boundsBehavior: Flickable.StopAtBounds
        // Follows the end only if you were already there (scrolling up to read stays put).
        property bool follow: true
        onContentYChanged: follow = contentY + height >= contentHeight - 60
        onContentHeightChanged: if (follow || l.busyChat)
            contentY = Math.max(0, contentHeight - height)

        ColumnLayout {
            id: col
            x: Theme.pad
            y: 6
            width: flick.width - 2 * Theme.pad
            spacing: 14

            Repeater {
                model: l.chat
                delegate: Loader {
                    id: msg
                    required property var modelData
                    Layout.fillWidth: true
                    sourceComponent: modelData.role === "user" ? userBubble : aiBlock
                }
            }
        }
    }

    Component {
        id: userBubble
        Item {
            implicitHeight: bubble.implicitHeight
            Rectangle {
                id: bubble
                anchors.right: parent.right
                width: Math.min(parent.width * 0.8, bubbleText.implicitWidth + 24)
                implicitHeight: bubbleText.implicitHeight + 16
                radius: Theme.r4
                color: Theme.accentSoft
                Text {
                    id: bubbleText
                    anchors.fill: parent
                    anchors.margins: 8
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    wrapMode: Text.WrapAnywhere
                    textFormat: Text.StyledText
                    color: Theme.text
                    font.family: Theme.font
                    font.pointSize: Theme.body
                    text: {
                        const m = parent.parent.parent.modelData;
                        let mk = Busca.esc(m.text);
                        for (const a of m.attachments || [])
                            mk = mk.split(Busca.esc("@" + a.nome)).join("<b><font color=\"" + Theme.accent + "\">" + Busca.esc("@" + a.nome) + "</font></b>");
                        return mk;
                    }
                }
            }
        }
    }

    Component {
        id: aiBlock
        ColumnLayout {
            id: block
            readonly property var m: parent.modelData
            readonly property var live: m.pending ? (m.live || {passos: [], pensando: true}) : null
            readonly property var steps: m.pending ? (live.passos || []) : (m.steps || [])
            spacing: 8

            // Steps: a ring while the tool runs, ✓ and the time when it ends.
            Repeater {
                model: block.steps
                delegate: Rectangle {
                    required property var modelData
                    Layout.fillWidth: true
                    implicitHeight: 30
                    radius: Theme.r3
                    color: Theme.surface
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 10
                        spacing: 10
                        Ring {
                            visible: modelData.estado === "rodando"
                        }
                        Icon {
                            visible: modelData.estado !== "rodando"
                            size: 14
                            name: modelData.estado === "erro" ? "triangle-alert" : modelData.estado === "espera" ? "circle-help" : "check"
                            tone: modelData.estado === "erro" ? "err" : modelData.estado === "espera" ? "acc" : "ok"
                        }
                        Txt {
                            text: modelData.titulo
                            size: "small"
                            strong: true
                            color: Theme.text
                        }
                        Txt {
                            Layout.fillWidth: true
                            text: modelData.detalhe || modelData.resumo || ""
                            size: "small"
                            elide: Text.ElideRight
                        }
                        Txt {
                            visible: modelData.segundos !== undefined
                            text: String(modelData.segundos).replace(".", ",") + " s"
                            size: "tiny"
                        }
                    }
                }
            }
            RowLayout {
                visible: !!block.m.pending && block.live.pensando !== false && !block.live.texto
                spacing: 10
                Layout.leftMargin: 10
                Ring {}
                Txt {
                    size: "small"
                    font.italic: true
                    text: {
                        const secs = Math.floor(block.m.elapsed || 0);
                        return block.steps.length ? "Juntando o que encontrei…" : "Pensando" + (secs > 2 ? " · " + secs + " s" : "…");
                    }
                }
            }

            // Command outputs
            Repeater {
                model: block.m.pending ? [] : (block.m.outputs || [])
                delegate: OutBox {
                    required property var modelData
                    Layout.fillWidth: true
                    out: modelData
                }
            }

            Text {
                Layout.fillWidth: true
                visible: text !== ""
                wrapMode: Text.WordWrap
                textFormat: Text.RichText
                color: Theme.text
                font.family: Theme.font
                font.pointSize: Theme.body
                onLinkActivated: link => Qt.openUrlExternally(link)
                text: {
                    if (block.m.pending)
                        return Busca.mdToHtml(block.live.texto || "");
                    const src = (block.m.sources || []).length ? "<br><br><font color=\"" + Theme.text2 + "\" size=\"2\">Fontes</font><br>"
                        + block.m.sources.map(u => "<a href=\"" + Busca.esc(u) + "\"><font color=\"" + Theme.accent + "\" size=\"2\">"
                            + Busca.esc(u) + "</font></a>").join("<br>") : "";
                    return Busca.mdToHtml(block.m.text || "") + src;
                }
            }

            // Command that changes something: shown, waits for "Rodar" (or Enter).
            ColumnLayout {
                visible: !block.m.pending && !!block.m.confirm
                Layout.fillWidth: true
                spacing: 6
                OutBox {
                    Layout.fillWidth: true
                    out: ({comando: block.m.confirm || "", hideOutput: true})
                }
                Txt {
                    visible: !!block.m.confirmState
                    text: block.m.confirmState === "ok" ? "Autorizado." : "Cancelado."
                    size: "tiny"
                }
                RowLayout {
                    visible: !block.m.confirmState
                    spacing: 8
                    Btn {
                        text: "Rodar  ↵"
                        primary: true
                        onClicked: chat.l.confirm(block.m, true)
                    }
                    Btn {
                        text: "Cancelar"
                        onClicked: chat.l.confirm(block.m, false)
                    }
                }
            }

            // AI question with options: A, B, C… (arrows + Enter, click, or write something else).
            ColumnLayout {
                visible: !block.m.pending && !!block.m.choice
                Layout.fillWidth: true
                spacing: 4
                readonly property bool open: block.m.choiceAnswered === undefined
                Repeater {
                    model: block.m.choice ? block.m.choice.opcoes : []
                    delegate: Rectangle {
                        id: opt
                        required property var modelData
                        required property int index
                        readonly property bool open: block.m.choiceAnswered === undefined
                        readonly property bool chosen: block.m.choiceAnswered === index
                        Layout.fillWidth: true
                        implicitHeight: optRow.implicitHeight + 16
                        radius: Theme.r3
                        color: open && index === (block.m.choiceSel || 0) ? Theme.accentSoft : Theme.surface
                        opacity: open || chosen ? 1 : 0.43
                        RowLayout {
                            id: optRow
                            anchors.fill: parent
                            anchors.margins: 8
                            anchors.leftMargin: 10
                            spacing: 10
                            Keycap {
                                text: opt.chosen ? "✓" : Busca.LETTERS[opt.index]
                                tint: Theme.accent
                                fill: Theme.accentSoft
                            }
                            Txt {
                                Layout.fillWidth: true
                                text: opt.modelData
                                wrapMode: Text.WordWrap
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            enabled: opt.open
                            hoverEnabled: true
                            onEntered: if ((block.m.choiceSel || 0) !== opt.index)
                                chat.l.moveChoice(block.m, opt.index - (block.m.choiceSel || 0))
                            onClicked: chat.l.answerChoice(block.m, opt.index)
                        }
                    }
                }
                Txt {
                    text: parent.open ? "↑↓ escolher · Enter responde · ou escreva outra resposta"
                        : block.m.choiceAnswered === "texto" ? "Respondido com outro texto." : ""
                    visible: text !== ""
                    size: "tiny"
                }
            }
        }
    }

    component OutBox: Rectangle {
        property var out: ({})
        implicitHeight: outCol.implicitHeight + 16
        radius: Theme.r3
        color: Theme.code
        ColumnLayout {
            id: outCol
            anchors.fill: parent
            anchors.margins: 8
            anchors.leftMargin: 10
            spacing: 4
            Text {
                Layout.fillWidth: true
                text: "$ " + (parent.parent.out.comando || "")
                font.family: "monospace"
                font.pointSize: Theme.small
                color: Theme.accent
                wrapMode: Text.WrapAnywhere
            }
            Text {
                visible: !parent.parent.out.hideOutput
                Layout.fillWidth: true
                font.family: "monospace"
                font.pointSize: Theme.small
                color: "#c8c8cc"
                wrapMode: Text.WrapAnywhere
                text: {
                    let lines = (parent.parent.out.saida || "(sem saída)").split("\n");
                    if (lines.length > 14)
                        lines = lines.slice(0, 6).concat(["… " + (lines.length - 12) + " linhas …"], lines.slice(-6));
                    return lines.join("\n");
                }
            }
            Txt {
                visible: !parent.parent.out.hideOutput
                size: "tiny"
                color: parent.parent.out.codigo === 0 ? Theme.text3 : Theme.danger
                text: (parent.parent.out.codigo === 0 ? "concluído" : "saiu com código " + parent.parent.out.codigo) + " · "
                    + String(parent.parent.out.segundos || 0).replace(".", ",") + " s"
            }
        }
    }
}
