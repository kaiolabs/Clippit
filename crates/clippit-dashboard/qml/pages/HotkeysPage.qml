import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ScrollView {
    id: root
    
    property var configModel
    
    ColumnLayout {
        width: root.width - 40
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: 30
        spacing: 25
        
        Text {
            text: "Atalhos de Teclado"
            font.pixelSize: 24
            font.bold: true
            color: "#333"
        }
        
        GroupBox {
            Layout.fillWidth: true
            title: "Atalho Principal"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 15
                
                Text {
                    text: "Pressione a combinação de teclas para mostrar o histórico:"
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
                
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    
                    ComboBox {
                        id: modifierCombo
                        Layout.preferredWidth: 150
                        model: ["super", "ctrl", "alt", "shift", "ctrl+shift", "ctrl+alt", "super+shift"]
                        currentIndex: {
                            let mod = configModel.hotkeyModifier.toLowerCase()
                            return model.indexOf(mod)
                        }
                        onCurrentTextChanged: {
                            configModel.hotkeyModifier = currentText
                        }
                    }
                    
                    Text {
                        text: "+"
                        font.pixelSize: 18
                        font.bold: true
                    }
                    
                    TextField {
                        id: keyField
                        Layout.preferredWidth: 80
                        text: configModel.hotkeyKey
                        placeholderText: "tecla"
                        maximumLength: 1
                        onTextChanged: {
                            if (text.length > 0) {
                                configModel.hotkeyKey = text.toLowerCase()
                            }
                        }
                        
                        Keys.onPressed: (event) => {
                            text = event.text
                            event.accepted = true
                        }
                    }
                    
                    Button {
                        text: "Testar"
                        onClicked: {
                            testNotification.text = "Pressione " + modifierCombo.currentText + " + " + keyField.text
                            testNotification.visible = true
                            testTimer.restart()
                        }
                    }
                }
                
                Rectangle {
                    id: testNotification
                    Layout.fillWidth: true
                    height: 40
                    color: "#2196f3"
                    radius: 6
                    visible: false
                    
                    property alias text: notifText.text
                    
                    Text {
                        id: notifText
                        anchors.centerIn: parent
                        color: "white"
                        font.bold: true
                    }
                    
                    Timer {
                        id: testTimer
                        interval: 3000
                        onTriggered: testNotification.visible = false
                    }
                }
                
                Rectangle {
                    Layout.fillWidth: true
                    height: 80
                    color: "#fff3cd"
                    radius: 6
                    border.color: "#ffc107"
                    border.width: 1
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 15
                        spacing: 10
                        
                        Text {
                            text: "⚠️"
                            font.pixelSize: 24
                        }
                        
                        Text {
                            Layout.fillWidth: true
                            text: "Após alterar o atalho, reinicie o daemon:\nsystemctl --user restart clippit"
                            wrapMode: Text.WordWrap
                            font.pixelSize: 12
                        }
                    }
                }
            }
        }
        
        GroupBox {
            Layout.fillWidth: true
            title: "Sugestões de Atalhos"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 10
                
                Text {
                    text: "Atalhos sem conflitos comuns:"
                    font.bold: true
                }
                
                Repeater {
                    model: [
                        { mod: "ctrl+shift", key: "v", desc: "Ctrl+Shift+V - Seguro e intuitivo" },
                        { mod: "alt", key: "v", desc: "Alt+V - Simples e rápido" },
                        { mod: "ctrl", key: "`", desc: "Ctrl+` - Estilo terminal" },
                        { mod: "super+shift", key: "c", desc: "Super+Shift+C - Clipboard" }
                    ]
                    
                    Rectangle {
                        Layout.fillWidth: true
                        height: 35
                        color: "#f5f5f5"
                        radius: 4
                        
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 10
                            
                            Text {
                                text: modelData.desc
                                Layout.fillWidth: true
                            }
                            
                            Button {
                                text: "Usar"
                                flat: true
                                onClicked: {
                                    configModel.hotkeyModifier = modelData.mod
                                    configModel.hotkeyKey = modelData.key
                                }
                            }
                        }
                    }
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
                text: "✓ Atalho salvo! Reinicie o daemon para aplicar."
                color: "white"
                font.bold: true
            }
            
            Timer {
                id: savedTimer
                interval: 4000
                onTriggered: savedNotification.visible = false
            }
        }
        
        Item { Layout.fillHeight: true }
    }
}
