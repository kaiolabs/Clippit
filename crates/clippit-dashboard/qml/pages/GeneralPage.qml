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
        
        // Page Title
        Text {
            text: "Configurações Gerais"
            font.pixelSize: 24
            font.bold: true
            color: "#333"
        }
        
        // History Settings
        GroupBox {
            Layout.fillWidth: true
            title: "Histórico"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 15
                
                RowLayout {
                    Layout.fillWidth: true
                    
                    Text {
                        text: "Máximo de itens:"
                        Layout.preferredWidth: 150
                    }
                    
                    Slider {
                        id: maxHistorySlider
                        Layout.fillWidth: true
                        from: 10
                        to: 500
                        stepSize: 10
                        value: configModel.maxHistoryItems
                        onValueChanged: configModel.maxHistoryItems = value
                    }
                    
                    Text {
                        text: maxHistorySlider.value.toFixed(0)
                        Layout.preferredWidth: 50
                        horizontalAlignment: Text.AlignRight
                        font.bold: true
                    }
                }
                
                RowLayout {
                    Layout.fillWidth: true
                    
                    Text {
                        text: "Intervalo de polling:"
                        Layout.preferredWidth: 150
                    }
                    
                    SpinBox {
                        id: pollIntervalSpin
                        Layout.fillWidth: true
                        from: 100
                        to: 1000
                        stepSize: 50
                        value: configModel.pollIntervalMs
                        onValueChanged: configModel.pollIntervalMs = value
                        
                        textFromValue: function(value) {
                            return value + " ms"
                        }
                    }
                }
            }
        }
        
        // Save/Reset Buttons
        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            
            Item { Layout.fillWidth: true }
            
            Button {
                text: "Resetar Padrões"
                onClicked: {
                    configModel.resetToDefaults()
                }
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
        
        // Success notification
        Rectangle {
            id: savedNotification
            Layout.fillWidth: true
            height: 40
            color: "#4caf50"
            radius: 6
            visible: false
            
            Text {
                anchors.centerIn: parent
                text: "✓ Configurações salvas com sucesso!"
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
