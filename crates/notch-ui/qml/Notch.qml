// Ventana del notch (backend de respaldo): sin bordes, transparente, siempre
// encima y sin aceptar foco. La geometría global la fija la aplicación desde
// C++ (registry::set_geometry); aquí no se vinculan x/y/width/height para no
// competir con ella. La píldora se anima dentro de la envolvente fija.

import QtQuick
import QtQuick.Window
import com.notch.ui

Window {
    id: win

    // Tamaño inicial = envolvente expandida; la app la reposiciona al adjuntar.
    width: 420
    height: 200
    color: "transparent"
    flags: Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    visible: bridge.state !== "hidden"

    NotchBridge {
        id: bridge
    }

    Component.onCompleted: bridge.attachWindow()

    // Contador de fotogramas dibujados (solo builds de depuración lo incrementan).
    onFrameSwapped: bridge.bumpFrame()

    // Tick periódico solo mientras haya una transición en curso; al asentarse
    // la píldora no queda ningún timer ni animación activos.
    Timer {
        interval: 16
        repeat: true
        running: bridge.transitionPending
        onTriggered: bridge.tick()
    }

    // Vigía del monitor: cambios de dpr o de pantalla disparan el
    // reposicionamiento en la aplicación.
    Item {
        readonly property real dpr: Screen.devicePixelRatio
        readonly property string screenName: Screen.name
        onDprChanged: bridge.screenChanged()
        onScreenNameChanged: bridge.screenChanged()
    }

    Rectangle {
        id: pill
        anchors.top: parent.top
        anchors.horizontalCenter: parent.horizontalCenter
        width: bridge.state === "expanded" ? 420 : 320
        height: bridge.state === "expanded" ? 200 : 44
        radius: bridge.state === "expanded" ? 20 : 22
        color: "#E6111111"

        Behavior on width { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
        Behavior on height { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
        Behavior on radius { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }

        // Contenido provisional (Hito 1): título y artista, visibles expandido.
        Column {
            anchors.centerIn: parent
            spacing: 4
            opacity: bridge.state === "expanded" ? 1.0 : 0.0
            Behavior on opacity { NumberAnimation { duration: 180 } }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "white"
                font.pixelSize: 16
                text: bridge.titleText
            }
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                color: "#BBBBBB"
                font.pixelSize: 13
                text: bridge.artistText
            }
        }

        // La píldora captura el hover y los clics (no roba el foco: la ventana
        // tiene WindowDoesNotAcceptFocus y la máscara limita la zona activa).
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onEntered: bridge.onHover(true)
            onExited: bridge.onHover(false)
        }
    }
}
