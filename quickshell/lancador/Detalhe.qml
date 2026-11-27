import QtQuick
import QtQuick.Layouts
import "../components"
import ".."
import "../lib/busca.js" as Busca

// Preview of a history item: color, image or text, plus information.
Flickable {
    id: d
    property var item: null
    property bool zoom: false
    readonly property string text_: item ? (item.texto || "").trim() : ""
    readonly property string color_: Busca.HEX_COLOR.test(text_) ? text_ : ""
    readonly property string kind: !item ? "" : color_ ? "Cor" : item.tipo === "image" ? "Imagem" : /^https?:\/\//.test(text_) ? "Link" : "Texto"
    clip: true
    contentHeight: col.implicitHeight + 28
    boundsBehavior: Flickable.StopAtBounds

    ColumnLayout {
        id: col
        x: Theme.pad
        y: 14
        width: d.width - 2 * Theme.pad
        spacing: 14
        visible: !!d.item

        Rectangle {
            visible: d.color_ !== ""
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: 192
            implicitHeight: 150
            radius: 10
            color: "#ffffff"
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 6
                spacing: 6
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 6
                    color: d.color_ || "transparent"
                }
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: d.color_.toUpperCase()
                    color: "#333333"
                    font.family: Theme.font
                    font.pointSize: Theme.tiny
                    font.weight: Font.DemiBold
                }
            }
        }
        Image {
            visible: !d.color_ && !!d.item && d.item.tipo === "image" && !!d.item.imagem
            Layout.fillWidth: true
            Layout.preferredHeight: d.zoom ? 470 : 320
            source: visible ? "file://" + d.item.imagem : ""
            fillMode: Image.PreserveAspectFit
            asynchronous: true
        }
        Txt {
            visible: !d.color_ && !!d.item && d.item.tipo === "image" && (d.item.ocr || "").length > 15 && !d.zoom
            Layout.fillWidth: true
            text: d.item ? (d.item.ocr || "").slice(0, 400) : ""
            size: "small"
            wrapMode: Text.WrapAnywhere
        }
        Txt {
            visible: !d.color_ && !!d.item && d.item.tipo !== "image"
            Layout.fillWidth: true
            text: d.text_.slice(0, d.zoom ? 2000 : 1200)
            wrapMode: Text.WrapAnywhere
        }
        Txt {
            visible: !d.zoom
            text: "Informações"
            size: "tiny"
            strong: true
            Layout.topMargin: 8
        }
        Repeater {
            model: {
                if (!d.item || d.zoom)
                    return [];
                const info = [["Tipo", d.kind], ["Copiado", (d.item.quando || "").replace("T", " ").slice(0, 16)]];
                if (d.kind === "Texto")
                    info.push(["Caracteres", String(Array.from(d.text_).length)]);
                if (d.item.tipo === "image" && d.item.ocr)
                    info.push(["Texto na imagem", Array.from(d.item.ocr).length + " caracteres"]);
                return info;
            }
            delegate: ColumnLayout {
                required property var modelData
                Layout.fillWidth: true
                spacing: 6
                Rectangle {
                    Layout.fillWidth: true
                    height: 1
                    color: Theme.line
                }
                RowLayout {
                    Layout.fillWidth: true
                    Txt {
                        Layout.fillWidth: true
                        text: modelData[0]
                        size: "small"
                    }
                    Txt {
                        text: modelData[1]
                        size: "small"
                        color: Theme.text
                    }
                }
            }
        }
    }
}
