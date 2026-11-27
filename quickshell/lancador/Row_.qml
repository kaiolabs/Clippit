import QtQuick
import QtQuick.Layouts
import Quickshell
import "../components"
import ".."
import "../lib/busca.js" as Busca

// One launcher row: icon, title + dimmed subtitle on the same line, kind.
Rectangle {
    id: row
    required property var modelData
    required property int index
    required property var launcher
    readonly property var it: modelData
    readonly property bool cb: launcher.isHistory
    readonly property string swatch: it.data && Busca.HEX_COLOR.test((it.data.texto || "").trim()) ? it.data.texto.trim() : ""
    implicitHeight: 40
    radius: Theme.r3
    color: index === launcher.sel ? Theme.hover : "transparent"

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 10
        anchors.rightMargin: 10
        spacing: 12
        Item {
            Layout.preferredWidth: 24
            Layout.preferredHeight: 24
            Rectangle {
                visible: row.swatch !== ""
                anchors.centerIn: parent
                width: 18
                height: 18
                radius: Theme.r1
                color: row.swatch || "transparent"
                border.color: Qt.rgba(1, 1, 1, 0.15)
            }
            Text {
                visible: !row.swatch && !!row.it.emoji
                anchors.centerIn: parent
                text: row.it.emoji || ""
                font.pointSize: 15
            }
            Image {
                visible: !row.swatch && !row.it.emoji && !!row.it.appIcon
                anchors.fill: parent
                source: row.it.appIcon || ""
                sourceSize.width: 48
                sourceSize.height: 48
                fillMode: Image.PreserveAspectFit
            }
            Icon {
                visible: !row.swatch && !row.it.emoji && !row.it.appIcon && row.cb && !!(row.it.data && row.it.data.tipo)
                anchors.centerIn: parent
                size: 16
                name: {
                    const h = row.it.data || {};
                    return h.fixado ? "pin" : h.tipo === "image" ? "image" : /^https?:\/\//.test(h.texto || "") ? "link" : "type";
                }
                tone: (row.it.data || {}).fixado ? "acc" : "g"
            }
            IconTile {
                visible: !row.swatch && !row.it.emoji && !row.it.appIcon && !(row.cb && row.it.data && row.it.data.tipo)
                anchors.centerIn: parent
                name: Busca.lucideFor(row.it.id, row.it.icon)
            }
        }
        Text {
            Layout.fillWidth: true
            elide: Text.ElideRight
            textFormat: Text.StyledText
            font.family: Theme.font
            font.pointSize: Theme.body
            color: Theme.text
            text: Busca.esc(row.it.title) + (!row.cb && row.it.subtitle
                ? "&nbsp;&nbsp;&nbsp;<font color=\"" + Theme.text2 + "\" size=\"2\">" + Busca.esc(row.it.subtitle) + "</font>" : "")
        }
        Txt {
            visible: !row.cb && text !== ""
            text: Busca.KIND[row.it.section] || ""
            size: "tiny"
        }
    }
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: if (row.launcher.sel !== row.index)
            row.launcher.sel = row.index
        onClicked: row.launcher.activate(row.it, false)
    }
}
