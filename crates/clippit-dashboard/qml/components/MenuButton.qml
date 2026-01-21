import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    width: parent.width
    height: 45
    color: selected ? "#3d3d3d" : (hovered ? "#353535" : "transparent")
    radius: 6
    
    property string text: ""
    property string icon: ""
    property bool selected: false
    property bool hovered: mouseArea.containsMouse
    
    signal clicked()
    
    Behavior on color { ColorAnimation { duration: 150 } }
    
    Row {
        anchors.left: parent.left
        anchors.leftMargin: 15
        anchors.verticalCenter: parent.verticalCenter
        spacing: 12
        
        Text {
            text: root.icon
            font.pixelSize: 20
            anchors.verticalCenter: parent.verticalCenter
        }
        
        Text {
            text: root.text
            color: root.selected ? "#ffffff" : "#cccccc"
            font.pixelSize: 14
            font.bold: root.selected
            anchors.verticalCenter: parent.verticalCenter
        }
    }
    
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
