//! Backend de respaldo (spec §7): ventana normal sin bordes, siempre encima y
//! sin aceptar foco; la región interactiva se limita con QWindow::setMask.
//!
//! La ventana la crea QML (`Notch.qml`); este backend la adopta desde el
//! registro de `notch_ui::registry` y aplica flags, geometría y máscara a
//! través de la fachada segura del registro (punteros contenidos allí).

use notch_core::backend::{BackendError, BackendId, EnvInfo, Probe, WindowBackend, WindowHandle};
use notch_core::Rect;
use notch_ui::registry;

/// Backend de respaldo: siempre disponible (es el último recurso).
#[derive(Debug, Default)]
pub struct FallbackBackend;

impl FallbackBackend {
    pub fn new() -> Self {
        Self
    }
}

fn invalid_handle(handle: &WindowHandle) -> BackendError {
    BackendError::Failed(format!("manejador de ventana no registrado: {}", handle.id()))
}

impl WindowBackend for FallbackBackend {
    fn id(&self) -> BackendId {
        BackendId::Fallback
    }

    fn probe(&self, _env: &EnvInfo) -> Probe {
        // Último recurso: no se descarta por entorno.
        Probe::Available
    }

    fn configure(&mut self, win: &mut WindowHandle, rect: Rect) -> Result<(), BackendError> {
        // Adopta la única ventana QML registrada (la crea Notch.qml al cargar).
        let handle = registry::first_window()
            .ok_or_else(|| BackendError::Failed("no hay ninguna ventana QML registrada".into()))?;
        registry::set_geometry(handle, rect).ok_or_else(|| invalid_handle(&handle))?;
        *win = handle;
        Ok(())
    }

    fn set_interactive_region(
        &mut self,
        win: &mut WindowHandle,
        region: Option<Rect>,
    ) -> Result<(), BackendError> {
        let result = match region {
            // Las coordenadas de la máscara son relativas a la ventana.
            Some(rect) => registry::apply_mask(*win, rect),
            None => registry::clear_mask(*win),
        };
        result.ok_or_else(|| invalid_handle(win))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notch_core::backend::{BackendError, BackendId, EnvInfo, Os, Probe, Session};
    use notch_core::Rect;

    fn env_info() -> EnvInfo {
        EnvInfo { os: Os::Linux, session: Session::Unknown, has_layer_shell: false }
    }

    #[test]
    fn id_is_fallback() {
        assert_eq!(FallbackBackend::new().id(), BackendId::Fallback);
    }

    #[test]
    fn probe_always_available() {
        // Es el último recurso: nunca se descarta por entorno.
        assert_eq!(FallbackBackend::new().probe(&env_info()), Probe::Available);
    }

    #[test]
    fn configure_with_unknown_handle_fails() {
        let mut backend = FallbackBackend::new();
        let mut win = WindowHandle::new(999);
        let rect = Rect { x: 0.0, y: 0.0, w: 420.0, h: 200.0 };
        let err = backend.configure(&mut win, rect).unwrap_err();
        assert!(matches!(err, BackendError::Failed(_)));
    }

    #[test]
    fn configure_without_registered_window_fails() {
        let mut backend = FallbackBackend::new();
        let mut win = WindowHandle::new(0);
        let rect = Rect { x: 0.0, y: 0.0, w: 420.0, h: 200.0 };
        // Sin QML cargado no hay ventanas registradas.
        let err = backend.configure(&mut win, rect).unwrap_err();
        assert!(matches!(err, BackendError::Failed(_)));
    }

    #[test]
    fn set_interactive_region_with_unknown_handle_fails() {
        let mut backend = FallbackBackend::new();
        let mut win = WindowHandle::new(999);
        let err = backend
            .set_interactive_region(&mut win, Some(Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 }))
            .unwrap_err();
        assert!(matches!(err, BackendError::Failed(_)));
    }
}
