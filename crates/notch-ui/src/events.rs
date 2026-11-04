//! Canal puente -> aplicación. cxx-qt 0.10 no permite a Rust obtener un
//! manejador del QObject instanciado por QML, así que el bridge notifica a
//! través de un listener global que la aplicación registra antes de cargar
//! el QML.

use std::sync::{Mutex, MutexGuard};

use notch_core::backend::WindowHandle;
use notch_core::machine::NotchState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiEvent {
    /// QML instanció la ventana y quedó registrada (llega una sola vez).
    WindowAttached(WindowHandle),
    /// La máquina de estados cambió de estado.
    StateChanged(NotchState),
    /// Cambió el monitor (resolución, conexión o dpr).
    ScreenChanged,
    /// La ventana dibujó un fotograma (QML `onFrameSwapped`).
    FrameDrawn,
}

type Listener = Box<dyn FnMut(&UiEvent) + Send + 'static>;

static LISTENER: Mutex<Option<Listener>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<Listener>> {
    LISTENER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Registra el listener de eventos de la UI (sobrescribe el anterior).
pub fn set_listener(f: impl FnMut(&UiEvent) + Send + 'static) {
    *lock() = Some(Box::new(f));
}

/// Quita el listener registrado.
pub fn clear_listener() {
    *lock() = None;
}

/// Notifica un evento al listener. El listener no debe reentrar en `emit`
/// (el mutex no es reentrante); documentado como riesgo en el reporte.
pub(crate) fn emit(event: &UiEvent) {
    if let Some(listener) = lock().as_mut() {
        listener(event);
    }
}
