//! Backend de respaldo: ventana normal sin bordes, siempre encima y sin
//! aceptar foco (spec §7). La región interactiva se limita con QWindow::setMask.

#[cfg(test)]
mod tests {
    use super::*;
    use notch_core::backend::{BackendId, EnvInfo, Os, Probe, Session};
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
    fn set_interactive_region_with_unknown_handle_fails() {
        let mut backend = FallbackBackend::new();
        let mut win = WindowHandle::new(999);
        let err = backend
            .set_interactive_region(&mut win, Some(Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 }))
            .unwrap_err();
        assert!(matches!(err, BackendError::Failed(_)));
    }
}
