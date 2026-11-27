import QtQuick
import ".."

// Glass icon tile: translucent square, thin border, the drawing in soft white.
Rectangle {
    property string name: "app-window"
    property int size: 24
    width: size
    height: size
    implicitWidth: size
    implicitHeight: size
    radius: size > 30 ? Theme.r3 : Theme.r2
    color: Qt.rgba(1, 1, 1, 0.07)
    border.color: Qt.rgba(1, 1, 1, 0.10)
    border.width: 1
    Icon {
        anchors.centerIn: parent
        name: parent.name
        tone: "w"
        size: Math.round(parent.size * 0.58)
        opacity: 0.85
    }
}
