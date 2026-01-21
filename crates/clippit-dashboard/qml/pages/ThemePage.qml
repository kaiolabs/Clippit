import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs

ScrollView {
    id: root
    
    property var configModel
    property var themeModel
    
    ColumnLayout {
        width: root.width - 40
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: 30
        spacing: 25
        
        Text {
            text: "Temas e Aparência"
            font.pixelSize: 24
            font.bold: true
            color: "#333"
        }
        
        GroupBox {
            Layout.fillWidth: true
            title: "Tema Atual"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 15
                
                ComboBox {
                    id: themeCombo
                    Layout.fillWidth: true
                    model: ["Dark", "Light"]
                    currentIndex: configModel.theme === "dark" ? 0 : 1
                    onCurrentTextChanged: {
                        configModel.theme = currentText.toLowerCase()
                        themeModel.loadTheme(currentText)
                    }
                }
                
                // Preview
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 150
                    color: "#f5f5f5"
                    radius: 8
                    border.color: "#ddd"
                    border.width: 1
                    
                    Rectangle {
                        anchors.centerIn: parent
                        width: parent.width * 0.8
                        height: parent.height * 0.8
                        color: themeModel.background
                        radius: 12
                        
                        ColumnLayout {
                            anchors.centerIn: parent
                            spacing: 5
                            
                            Repeater {
                                model: 3
                                
                                Rectangle {
                                    width: 250
                                    height: 35
                                    color: index === 1 ? themeModel.selection : "transparent"
                                    radius: 4
                                    
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Item " + (index + 1)
                                        color: themeModel.foreground
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        GroupBox {
            Layout.fillWidth: true
            title: "Fonte"
            
            RowLayout {
                anchors.fill: parent
                spacing: 15
                
                Text {
                    text: "Família:"
                    Layout.preferredWidth: 80
                }
                
                ComboBox {
                    Layout.fillWidth: true
                    model: ["Nunito", "Inter", "Roboto", "Ubuntu", "System"]
                    currentIndex: {
                        let fonts = ["nunito", "inter", "roboto", "ubuntu", "system"]
                        return fonts.indexOf(configModel.fontFamily.toLowerCase())
                    }
                    onCurrentTextChanged: {
                        configModel.fontFamily = currentText
                    }
                }
                
                Text {
                    text: "Tamanho:"
                    Layout.preferredWidth: 70
                }
                
                SpinBox {
                    from: 10
                    to: 24
                    value: configModel.fontSize
                    onValueChanged: configModel.fontSize = value
                }
            }
        }
        
        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            
            Item { Layout.fillWidth: true }
            
            Button {
                text: "Resetar Padrões"
                onClicked: configModel.resetToDefaults()
            }
            
            Button {
                text: "Salvar"
                highlighted: true
                onClicked: {
                    if (configModel.saveConfig()) {
                        themeModel.saveCustomTheme()
                        savedNotification.visible = true
                        savedTimer.restart()
                    }
                }
            }
        }
        
        Rectangle {
            id: savedNotification
            Layout.fillWidth: true
            height: 40
            color: "#4caf50"
            radius: 6
            visible: false
            
            Text {
                anchors.centerIn: parent
                text: "✓ Tema salvo com sucesso!"
                color: "white"
                font.bold: true
            }
            
            Timer {
                id: savedTimer
                interval: 3000
                onTriggered: savedNotification.visible = false
            }
        }
        
        Item { Layout.fillHeight: true }
    }
}
