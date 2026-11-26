import QtQuick
import ".."

// Small icon button (close, edit, folder…).
Rectangle {
    id: b
    property string icon: "square-x"
    property color hoverColor: Theme.hover
    property color normalColor: "transparent"
    signal clicked
    width: 28
    height: 28
    radius: Theme.r2
    color: mouse.containsMouse ? hoverColor : normalColor
    Icon {
        anchors.centerIn: parent
        name: b.icon
        size: 16
    }
    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: b.clicked()
    }
}
