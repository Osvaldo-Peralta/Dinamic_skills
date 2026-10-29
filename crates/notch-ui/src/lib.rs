//! UI del notch: bridge cxx-qt (`NotchBridge`), `Notch.qml`, registro de
//! ventanas y canal de eventos hacia la aplicación.

pub mod bridge;
pub mod events;
pub mod registry;

/// URL qrc del componente de ventana del notch (módulo QML `com.notch.ui`).
pub const NOTCH_QML_URL: &str = "qrc:/qt/qml/com/notch/ui/qml/Notch.qml";

/// Tamaño provisional de la píldora en reposo (spec Hito 1, ajustable).
pub const PILL_SIZE: notch_core::Size = notch_core::Size { w: 320.0, h: 44.0 };
/// Tamaño provisional del notch expandido; también es la envolvente de la
/// ventana (ver decisión en el reporte de Tarea 3).
pub const EXPANDED_SIZE: notch_core::Size = notch_core::Size { w: 420.0, h: 200.0 };
