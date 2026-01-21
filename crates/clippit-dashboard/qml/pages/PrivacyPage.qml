import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ScrollView {
    id: root
    
    ColumnLayout {
        width: root.width - 40
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: 30
        spacing: 25
        
        Text {
            text: "Privacidade e Segurança"
            font.pixelSize: 24
            font.bold: true
            color: "#333"
        }
        
        GroupBox {
            Layout.fillWidth: true
            title: "Aplicativos Ignorados"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 15
                
                Text {
                    text: "O Clippit não capturará conteúdo dos seguintes aplicativos:"
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
                
                ListView {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 150
                    clip: true
                    
                    model: ListModel {
                        id: ignoredAppsModel
                        ListElement { name: "keepassxc" }
                        ListElement { name: "bitwarden" }
                        ListElement { name: "1password" }
                    }
                    
                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 40
                        color: index % 2 === 0 ? "#f5f5f5" : "white"
                        
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 10
                            
                            Text {
                                text: "🔒 " + model.name
                                Layout.fillWidth: true
                            }
                            
                            Button {
                                text: "Remover"
                                flat: true
                                onClicked: ignoredAppsModel.remove(index)
                            }
                        }
                    }
                }
                
                RowLayout {
                    Layout.fillWidth: true
                    
                    TextField {
                        id: newAppField
                        Layout.fillWidth: true
                        placeholderText: "nome-do-aplicativo"
                    }
                    
                    Button {
                        text: "Adicionar"
                        enabled: newAppField.text.length > 0
                        onClicked: {
                            ignoredAppsModel.append({ name: newAppField.text })
                            newAppField.text = ""
                        }
                    }
                }
                
                Rectangle {
                    Layout.fillWidth: true
                    height: 60
                    color: "#e3f2fd"
                    radius: 6
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        
                        Text {
                            text: "ℹ️"
                            font.pixelSize: 20
                        }
                        
                        Text {
                            Layout.fillWidth: true
                            text: "Use 'xprop | grep WM_CLASS' no terminal e clique na janela para descobrir o nome do aplicativo."
                            wrapMode: Text.WordWrap
                            font.pixelSize: 11
                        }
                    }
                }
            }
        }
        
        GroupBox {
            Layout.fillWidth: true
            title: "Outras Opções"
            
            ColumnLayout {
                anchors.fill: parent
                spacing: 10
                
                CheckBox {
                    text: "Ignorar automaticamente aplicativos sensíveis conhecidos"
                    checked: true
                }
                
                CheckBox {
                    text: "Limpar histórico ao sair"
                    checked: false
                }
            }
        }
        
        RowLayout {
            Layout.fillWidth: true
            
            Item { Layout.fillWidth: true }
            
            Button {
                text: "Salvar"
                highlighted: true
                onClicked: {
                    // TODO: Save to config
                    savedNotification.visible = true
                    savedTimer.restart()
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
                text: "✓ Configurações de privacidade salvas!"
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
