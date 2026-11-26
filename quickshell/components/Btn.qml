import QtQuick
import ".."

// Text button: primary (blue) or plain.
Rectangle {
    id: btn
    property alias text: label.text
    property bool primary: false
    signal clicked
    implicitWidth: label.implicitWidth + 24
    implicitHeight: label.implicitHeight + 10
    radius: Theme.r2
    color: primary ? (mouse.containsMouse ? Theme.accentHover : Theme.accent) : (mouse.containsMouse ? Qt.rgba(1, 1, 1, 0.14) : Theme.hover)
    Txt {
        id: label
        anchors.centerIn: parent
        size: "small"
        strong: btn.primary
        color: btn.primary ? "#ffffff" : Theme.text
    }
    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: btn.clicked()
    }
}
