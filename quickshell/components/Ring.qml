import QtQuick
import ".."

// Loading ring (the same drawing as the badge).
Canvas {
    id: ring
    property bool running: visible
    width: 14
    height: 14
    onPaint: {
        const cr = getContext("2d");
        cr.reset();
        const r = Math.min(width, height) / 2 - 1.5;
        cr.lineWidth = 2;
        cr.strokeStyle = Qt.rgba(1, 1, 1, 0.12);
        cr.beginPath();
        cr.arc(width / 2, height / 2, r, 0, 2 * Math.PI);
        cr.stroke();
        cr.strokeStyle = Theme.accent;
        cr.beginPath();
        cr.arc(width / 2, height / 2, r, -Math.PI / 2, Math.PI / 4);
        cr.stroke();
    }
    RotationAnimation on rotation {
        from: 0
        to: 360
        duration: 800
        loops: Animation.Infinite
        running: ring.running
    }
}
