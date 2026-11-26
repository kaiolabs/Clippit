import QtQuick
import ".."

// A Lucide icon from the extension's icons/ folder (already colored SVGs).
Image {
    property string name: ""
    property string tone: "g"
    property int size: 16
    width: size
    height: size
    implicitWidth: size
    implicitHeight: size
    sourceSize.width: size * 2
    sourceSize.height: size * 2
    source: name ? Theme.icon(name, tone) : ""
    fillMode: Image.PreserveAspectFit
    smooth: true
}
