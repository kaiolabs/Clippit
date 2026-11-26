import QtQuick
import QtQuick.Layouts
import "../components"
import ".."

// Emoji picker grid (":" or "Emoji e símbolos"): sections, tiles sized to the
// window width, arrows/Tab/Enter from the launcher's key handler.
Flickable {
    id: g
    required property var launcher
    property string query: ""
    readonly property var l: launcher
    readonly property var sections: l.emojiData ? l.emojiSections(query) : null
    readonly property int gap: 8
    readonly property int cols: Math.max(6, Math.floor((width - 2 * Theme.pad + gap) / (72 + gap)))
    readonly property int tile: Math.floor((width - 2 * Theme.pad - gap * (cols - 1)) / cols)
    // Flat list (index → emoji, row, col) and rows (row → indexes), for the arrows.
    readonly property var layout: {
        const flat = [], rows = [], secs = [];
        for (const sec of sections || []) {
            const secRows = [];
            for (let i = 0; i < sec.items.length; i += cols) {
                const idx = [];
                for (const x of sec.items.slice(i, i + cols)) {
                    idx.push(flat.length);
                    flat.push(Object.assign({}, x, {row: rows.length, col: idx.length - 1}));
                }
                rows.push(idx);
                secRows.push(idx);
            }
            secs.push({title: sec.title, rows: secRows});
        }
        return {flat, rows, secs};
    }
    readonly property var flat: layout.flat
    readonly property var rows: layout.rows
    clip: true
    contentHeight: col.implicitHeight + 14
    boundsBehavior: Flickable.StopAtBounds
    onQueryChanged: contentY = 0

    Connections {
        target: g.l
        function onEmojiSelChanged() {
            const t = g.tiles[g.l.emojiSel];
            if (!t)
                return;
            const p = t.mapToItem(col, 0, 0);
            if (p.y < g.contentY + 30)
                g.contentY = Math.max(0, p.y - 40);
            else if (p.y + t.height > g.contentY + g.height - 10)
                g.contentY = p.y + t.height - g.height + 20;
        }
    }
    property var tiles: ({})

    Txt {
        visible: !g.sections
        x: Theme.pad
        y: 12
        text: "Carregando os emoji…"
        size: "small"
    }

    ColumnLayout {
        id: col
        x: Theme.pad
        width: g.width - 2 * Theme.pad
        spacing: 8
        Repeater {
            model: g.layout.secs
            delegate: ColumnLayout {
                id: sec
                required property var modelData
                spacing: 8
                Txt {
                    text: sec.modelData.title
                    size: "tiny"
                    strong: true
                    topPadding: 6
                    leftPadding: 2
                }
                Repeater {
                    model: sec.modelData.rows
                    delegate: Row {
                        id: r
                        required property var modelData
                        spacing: g.gap
                        Repeater {
                            model: r.modelData
                            delegate: Rectangle {
                                id: t
                                required property int modelData
                                readonly property var x_: g.flat[modelData]
                                readonly property bool on: modelData === g.l.emojiSel
                                width: g.tile
                                height: Math.min(g.tile, 76)
                                radius: Theme.r3
                                color: on ? Theme.hover : Theme.surface
                                border.width: 2
                                border.color: on ? Qt.rgba(1, 1, 1, 0.75) : "transparent"
                                Component.onCompleted: g.tiles[modelData] = t
                                Text {
                                    anchors.centerIn: parent
                                    text: t.x_ ? t.x_.e : ""
                                    font.pointSize: 24
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    onEntered: g.l.emojiSel = t.modelData
                                    onClicked: g.l.emojiUse(t.x_, false)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
