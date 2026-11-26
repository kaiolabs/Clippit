pragma Singleton

import QtQuick
import Quickshell

// The single visual standard (same values as the GNOME extension): every
// panel uses only these sizes, colors and corners.
Singleton {
    readonly property color bg: "#1e1e1e"
    readonly property color card: "#262626"
    readonly property color cardHover: "#2c2c2c"
    readonly property color border: "#333333"
    readonly property color borderStrong: "#424242"
    readonly property color text: "#f2f2f2"
    readonly property color text2: "#9a9aa0"
    readonly property color text3: "#6e6e74"
    readonly property color accent: "#4c9dff"
    readonly property color accentHover: "#6aafff"
    readonly property color accentBg: "#1c2f45"
    readonly property color danger: "#ff6b6b"
    readonly property color dangerBg: "#3a1f22"
    readonly property color ok: "#4ade80"

    // Without the blur GNOME Shell gave, the glass needs to hide more of what is behind.
    readonly property color glass: Qt.rgba(26 / 255, 26 / 255, 30 / 255, 0.97)
    readonly property color glassBorder: Qt.rgba(1, 1, 1, 0.10)
    readonly property color surface: Qt.rgba(1, 1, 1, 0.04)
    readonly property color hover: Qt.rgba(1, 1, 1, 0.08)
    readonly property color line: Qt.rgba(1, 1, 1, 0.08)
    readonly property color accentSoft: Qt.rgba(76 / 255, 157 / 255, 1, 0.16)
    readonly property color code: Qt.rgba(0, 0, 0, 0.30)

    readonly property string font: "Inter"
    readonly property real input: 12
    readonly property real body: 10
    readonly property real small: 9
    readonly property real tiny: 8

    readonly property int r1: 4
    readonly property int r2: 6
    readonly property int r3: 8
    readonly property int r4: 12
    readonly property int pad: 18

    // The extension's icons (in the Nix package a link puts them at the same relative place).
    readonly property string iconDir: "file://" + Quickshell.shellPath("../gnome-extension/clippit-corretor@clippit/icons/")

    function icon(name, color) {
        return iconDir + name + "-" + (color || "g") + ".svg";
    }
}
