import QtQuick
import QtQuick.Layouts
import ".."

// Empty screen (the same drawing everywhere): icon, title and explanation.
ColumnLayout {
    property string icon: "search"
    property string title: ""
    property string sub: ""
    spacing: 8
    IconTile {
        Layout.alignment: Qt.AlignHCenter
        name: parent.icon
        size: 36
    }
    Txt {
        Layout.alignment: Qt.AlignHCenter
        Layout.topMargin: 4
        text: parent.title
        strong: true
        horizontalAlignment: Text.AlignHCenter
    }
    Txt {
        Layout.alignment: Qt.AlignHCenter
        Layout.maximumWidth: 520
        visible: text !== ""
        text: parent.sub
        size: "small"
        wrapMode: Text.WordWrap
        horizontalAlignment: Text.AlignHCenter
    }
}
