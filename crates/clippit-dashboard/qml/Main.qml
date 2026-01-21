import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.clippit.dashboard 1.0

ApplicationWindow {
    id: root
    width: 900
    height: 600
    visible: true
    title: "Clippit - Configurações"
    
    color: "#f5f5f5"
    
    // Models
    ConfigModel {
        id: configModel
        Component.onCompleted: loadConfig()
    }
    
    ThemeModel {
        id: themeModel
        Component.onCompleted: loadTheme("Dark")
    }
    
    RowLayout {
        anchors.fill: parent
        spacing: 0
        
        // Sidebar
        Rectangle {
            Layout.preferredWidth: 220
            Layout.fillHeight: true
            color: "#2d2d2d"
            
            Column {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 5
                
                // Header
                Rectangle {
                    width: parent.width
                    height: 80
                    color: "transparent"
                    
                    Column {
                        anchors.centerIn: parent
                        spacing: 5
                        
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: "📋"
                            font.pixelSize: 32
                        }
                        
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: "Clippit"
                            color: "white"
                            font.pixelSize: 18
                            font.bold: true
                        }
                    }
                }
                
                Rectangle {
                    width: parent.width
                    height: 1
                    color: "#444444"
                }
                
                // Menu Items
                Repeater {
                    model: [
                        { icon: "⚙️", text: "Geral", page: 0 },
                        { icon: "⌨️", text: "Atalhos", page: 1 },
                        { icon: "🎨", text: "Temas", page: 2 },
                        { icon: "🔐", text: "Privacidade", page: 3 }
                    ]
                    
                    MenuButton {
                        text: modelData.text
                        icon: modelData.icon
                        selected: stackLayout.currentIndex === modelData.page
                        onClicked: stackLayout.currentIndex = modelData.page
                    }
                }
                
                Item { Layout.fillHeight: true }
                
                Rectangle {
                    width: parent.width
                    height: 1
                    color: "#444444"
                }
                
                // Footer buttons
                MenuButton {
                    text: "Sobre"
                    icon: "ℹ️"
                    onClicked: aboutDialog.open()
                }
            }
        }
        
        // Main Content Area
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#ffffff"
            
            StackLayout {
                id: stackLayout
                anchors.fill: parent
                currentIndex: 0
                
                GeneralPage {
                    configModel: configModel
                }
                
                HotkeysPage {
                    configModel: configModel
                }
                
                ThemePage {
                    configModel: configModel
                    themeModel: themeModel
                }
                
                PrivacyPage {
                }
            }
        }
    }
    
    // About Dialog
    Dialog {
        id: aboutDialog
        title: "Sobre o Clippit"
        anchors.centerIn: parent
        modal: true
        
        ColumnLayout {
            spacing: 10
            
            Text {
                text: "Clippit v1.0"
                font.pixelSize: 18
                font.bold: true
            }
            
            Text {
                text: "Gerenciador de Clipboard para Linux"
                color: "#666"
            }
            
            Text {
                text: "© 2026 - MIT License"
                color: "#999"
                font.pixelSize: 12
            }
        }
        
        standardButtons: Dialog.Ok
    }
}
