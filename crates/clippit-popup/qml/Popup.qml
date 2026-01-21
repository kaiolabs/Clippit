import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Effects
import com.clippit.popup 1.0

Window {
    id: root
    width: 600
    height: 450
    flags: Qt.Window | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint
    color: "transparent"
    
    // Models
    HistoryModel {
        id: historyModel
        Component.onCompleted: loadHistory(50)
    }
    
    ThemeModel {
        id: themeModel
        Component.onCompleted: loadTheme("Dark")
    }
    
    // Main popup container
    Rectangle {
        id: popupContainer
        anchors.fill: parent
        anchors.margins: 20
        color: themeModel.background
        radius: 12
        opacity: 0.98
        
        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowBlur: 0.5
            shadowOpacity: 0.3
            shadowColor: "#000000"
            shadowHorizontalOffset: 0
            shadowVerticalOffset: 4
        }
        
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 15
            spacing: 10
            
            // Header
            RowLayout {
                Layout.fillWidth: true
                
                Text {
                    text: "📋 Histórico do Clipboard"
                    color: themeModel.foreground
                    font.pixelSize: 18
                    font.bold: true
                }
                
                Item { Layout.fillWidth: true }
                
                Text {
                    text: historyModel.getItemCount() + " itens"
                    color: themeModel.foreground
                    opacity: 0.6
                    font.pixelSize: 12
                }
                
                Button {
                    text: "✕"
                    flat: true
                    onClicked: root.close()
                    
                    contentItem: Text {
                        text: parent.text
                        color: themeModel.foreground
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
            
            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: themeModel.border
            }
            
            // Search field
            TextField {
                id: searchField
                Layout.fillWidth: true
                placeholderText: "Buscar no histórico..."
                font.pixelSize: 14
                
                background: Rectangle {
                    color: themeModel.background
                    border.color: themeModel.border
                    border.width: 1
                    radius: 6
                }
                
                color: themeModel.foreground
                
                Keys.onEscapePressed: root.close()
                Keys.onDownPressed: historyList.forceActiveFocus()
            }
            
            // History list
            ListView {
                id: historyList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                focus: true
                spacing: 2
                
                model: historyModel.getItemCount()
                
                delegate: HistoryItem {
                    width: ListView.view.width
                    itemIndex: index
                    historyModel: historyModel
                    themeModel: themeModel
                    
                    onItemClicked: (id) => {
                        if (historyModel.selectItem(id)) {
                            root.close()
                        }
                    }
                }
                
                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                }
                
                Keys.onEscapePressed: root.close()
                Keys.onReturnPressed: {
                    if (currentIndex >= 0) {
                        let item = historyList.itemAtIndex(currentIndex)
                        if (item) {
                            item.clicked()
                        }
                    }
                }
            }
            
            // Footer
            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: themeModel.border
            }
            
            RowLayout {
                Layout.fillWidth: true
                
                Text {
                    text: "💡 Use ↑↓ para navegar, Enter para selecionar, Esc para fechar"
                    color: themeModel.foreground
                    opacity: 0.5
                    font.pixelSize: 10
                }
                
                Item { Layout.fillWidth: true }
            }
        }
    }
    
    // Center window on screen
    Component.onCompleted: {
        x = (Screen.width - width) / 2
        y = (Screen.height - height) / 2
        searchField.forceActiveFocus()
    }
    
    // Close on focus out
    onActiveChanged: {
        if (!active) {
            Qt.callLater(root.close)
        }
    }
}
