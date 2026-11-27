import QtQuick
import ".."

// A Lucide icon from the extension's icons/ folder (already colored SVGs).
Item {
    property string name: ""
    property string tone: "g"
    property int size: 16
    implicitWidth: size
    implicitHeight: size
    width: size
    height: size
    Image {
        anchors.fill: parent
        sourceSize.width: parent.size * 2
        sourceSize.height: parent.size * 2
        source: parent.name ? Theme.icon(parent.name, parent.tone) : ""
        fillMode: Image.PreserveAspectFit
        smooth: true
    }
}
