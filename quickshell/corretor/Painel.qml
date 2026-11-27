import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import "../components"
import "../services"
import ".."
import "../lib/texto.js" as Texto
import "../lib/busca.js" as Busca

// Suggestions panel: one row per suggestion (accept, ignore) and "Corrigir tudo".
PanelWindow {
    id: win
    required property var corretor
    readonly property var sugs: corretor.panelOpen ? corretor.pending() : []
    readonly property var pos: corretor.panelPos
    readonly property var scr: {
        const p = pos || {x: 0, y: 0};
        return Quickshell.screens.find(s => p.x >= s.x && p.x < s.x + s.width && p.y >= s.y && p.y < s.y + s.height)
            || Quickshell.screens[0];
    }
    readonly property int panelWidth: 440

    visible: corretor.panelOpen && sugs.length > 0
    screen: scr
    color: "transparent"
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.namespace: "clippit"
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
    anchors.top: true
    anchors.left: true
    implicitWidth: panelWidth
    implicitHeight: Math.min(box.implicitHeight, 460)
    // Above the cursor if it fits, otherwise below.
    margins.left: Math.max(12, Math.min((pos ? pos.x - scr.x - 20 : 100), scr.width - panelWidth - 12))
    margins.top: {
        const y = pos ? pos.y - scr.y : 100;
        return y - implicitHeight - 14 >= 12 ? y - implicitHeight - 14 : Math.min(y + 24, scr.height - implicitHeight - 12);
    }

    Glass {
        anchors.fill: parent
    }

    ColumnLayout {
        id: box
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 14
            Layout.leftMargin: Theme.pad
            Layout.rightMargin: Theme.pad
            spacing: 10
            Txt {
                text: "Sugestões"
                strong: true
            }
            Keycap {
                text: String(win.sugs.length)
                tint: Theme.accent
                fill: Theme.accentSoft
            }
            Item {
                Layout.fillWidth: true
            }
            Btn {
                text: "Corrigir tudo"
                primary: true
                onClicked: win.corretor.apply(win.corretor.pending())
            }
            IconBtn {
                icon: "square-x"
                onClicked: win.corretor.dismissPanel()
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(contentHeight, 340)
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            clip: true
            spacing: 6
            model: win.sugs
            boundsBehavior: Flickable.StopAtBounds
            delegate: Rectangle {
                id: row
                required property var modelData
                width: list.width
                implicitHeight: rowLay.implicitHeight + 16
                radius: Theme.r3
                color: hov.containsMouse ? Theme.hover : Theme.surface
                MouseArea {
                    id: hov
                    anchors.fill: parent
                    hoverEnabled: true
                }
                RowLayout {
                    id: rowLay
                    anchors.fill: parent
                    anchors.margins: 8
                    anchors.leftMargin: 10
                    spacing: 10
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 3
                        Txt {
                            text: Texto.kindOf(row.modelData)
                            size: "tiny"
                            strong: true
                        }
                        Txt {
                            Layout.fillWidth: true
                            textFormat: Text.StyledText
                            wrapMode: Text.WordWrap
                            color: Theme.text2
                            text: {
                                const s = row.modelData;
                                const parts = [];
                                if (s.before)
                                    parts.push(Busca.esc(s.before) + " ");
                                if (s.from)
                                    parts.push("<s><font color=\"" + Theme.danger + "\">" + Busca.esc(s.from) + "</font></s>");
                                if (s.from && s.to)
                                    parts.push(" ");
                                if (s.to)
                                    parts.push("<b><font color=\"" + Theme.ok + "\">" + Busca.esc(s.to) + "</font></b>");
                                if (s.after)
                                    parts.push(" " + Busca.esc(s.after));
                                return parts.join("");
                            }
                        }
                    }
                    IconBtn {
                        icon: "check"
                        normalColor: Theme.hover
                        hoverColor: Qt.rgba(74 / 255, 222 / 255, 128 / 255, 0.16)
                        onClicked: win.corretor.apply([row.modelData])
                    }
                    IconBtn {
                        icon: "square-x"
                        normalColor: Theme.hover
                        hoverColor: Theme.dangerBg
                        onClicked: win.corretor.ignore(row.modelData)
                    }
                }
            }
        }

        Footer {
            Layout.fillWidth: true
            Layout.topMargin: 4
            where: "Corretor"
            icon: "spell-check"
            actions: [{label: "Corrigir tudo", key: Conf.label("aplicar")}, {label: "Fechar", key: "Esc"}]
        }
    }
}
