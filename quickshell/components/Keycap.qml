import QtQuick
import ".."

// A key (shortcut) drawn the same everywhere.
Rectangle {
    property alias text: label.text
    property color tint: Theme.text2
    property color fill: Theme.hover
    implicitWidth: label.implicitWidth + 12
    implicitHeight: label.implicitHeight + 2
    radius: Theme.r1
    color: fill
    Txt {
        id: label
        anchors.centerIn: parent
        size: "tiny"
        strong: true
        color: parent.tint
    }
}
