import QtQuick
import ".."

// Text in one of the 4 sizes of the theme.
Text {
    property string size: "body"
    property bool strong: false
    font.family: Theme.font
    font.pointSize: size === "input" ? Theme.input : size === "small" ? Theme.small : size === "tiny" ? Theme.tiny : Theme.body
    font.weight: strong ? Font.DemiBold : Font.Normal
    color: size === "small" ? Theme.text2 : size === "tiny" ? Theme.text3 : Theme.text
    textFormat: Text.PlainText
    verticalAlignment: Text.AlignVCenter
}
