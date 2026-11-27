import QtQuick
import QtQuick.Layouts
import ".."

// Standard footer: where you are on the left, the actions and their keys on
// the right. actions: [{label, key, run?}] — the first is the main one (Enter).
Item {
    id: foot
    property string where: ""
    property string icon: ""
    property var actions: []
    readonly property string primary: (actions.filter(a => a && a.label)[0] || {}).label || ""
    implicitHeight: 44
    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.pad
        anchors.rightMargin: 12
        anchors.topMargin: 10
        anchors.bottomMargin: 12
        spacing: 8
        Icon {
            visible: foot.where !== "" && foot.icon !== ""
            name: foot.icon
            size: 14
        }
        Txt {
            visible: foot.where !== ""
            text: foot.where
            size: "small"
            elide: Text.ElideRight
            Layout.maximumWidth: 360
        }
        Item {
            Layout.fillWidth: true
        }
        Repeater {
            model: foot.actions.filter(a => a && a.label)
            delegate: RowLayout {
                id: act
                required property var modelData
                required property int index
                spacing: 8
                Rectangle {
                    visible: act.index > 0
                    width: 1
                    height: 14
                    color: Theme.line
                    Layout.leftMargin: 4
                    Layout.rightMargin: 4
                }
                Rectangle {
                    implicitWidth: inner.implicitWidth + (act.modelData.run ? 12 : 0)
                    implicitHeight: inner.implicitHeight + 4
                    radius: Theme.r2
                    color: act.modelData.run && hov.containsMouse ? Theme.hover : "transparent"
                    RowLayout {
                        id: inner
                        anchors.centerIn: parent
                        spacing: 8
                        Txt {
                            text: act.modelData.label
                            size: "small"
                            strong: act.index === 0
                            color: act.index === 0 ? Theme.text : Theme.text2
                        }
                        Keycap {
                            visible: !!act.modelData.key
                            text: act.modelData.key || ""
                        }
                    }
                    MouseArea {
                        id: hov
                        anchors.fill: parent
                        enabled: !!act.modelData.run
                        hoverEnabled: true
                        onClicked: act.modelData.run()
                    }
                }
            }
        }
    }
}
