import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    height: 65
    color: hovered ? themeModel.selection : "transparent"
    radius: 6
    
    required property int itemIndex
    required property var historyModel
    required property var themeModel
    
    property bool hovered: mouseArea.containsMouse || ListView.isCurrentItem
    
    signal itemClicked(int id)
    
    Behavior on color { ColorAnimation { duration: 150 } }
    
    RowLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 12
        
        // Icon
        Rectangle {
            Layout.preferredWidth: 40
            Layout.preferredHeight: 40
            color: themeModel.selection
            radius: 8
            
            Text {
                anchors.centerIn: parent
                text: {
                    let type = historyModel.getItemType(root.itemIndex)
                    return type === "text" ? "📄" : "🖼️"
                }
                font.pixelSize: 20
            }
        }
        
        // Content
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 4
            
            Text {
                id: contentText
                Layout.fillWidth: true
                text: historyModel.getItemContent(root.itemIndex)
                color: themeModel.foreground
                font.pixelSize: 13
                elide: Text.ElideRight
                maximumLineCount: 2
                wrapMode: Text.Wrap
            }
            
            Text {
                text: historyModel.getItemTimestamp(root.itemIndex)
                color: themeModel.foreground
                opacity: 0.5
                font.pixelSize: 10
            }
        }
        
        // Copy indicator (shows on hover)
        Text {
            text: "📋"
            font.pixelSize: 16
            opacity: root.hovered ? 0.6 : 0
            
            Behavior on opacity { NumberAnimation { duration: 150 } }
        }
    }
    
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        
        onClicked: {
            // Get entry ID (using index for now)
            root.itemClicked(root.itemIndex)
        }
        
        onEntered: {
            root.ListView.view.currentIndex = root.itemIndex
        }
    }
    
    // Selection indicator
    Rectangle {
        width: 3
        height: parent.height * 0.6
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        color: themeModel.foreground
        radius: 2
        visible: root.hovered
        opacity: 0.8
    }
}
