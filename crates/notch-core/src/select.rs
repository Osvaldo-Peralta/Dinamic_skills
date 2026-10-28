//! Detección de entorno y selección de backend con respaldo. Lógica pura.

use std::collections::HashMap;

use crate::backend::{BackendId, EnvInfo, Os, Probe, Session, WindowBackend, WindowHandle};
use crate::Rect;

/// Ningún backend de la lista de preferencia pudo configurarse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectError;

/// Construye `EnvInfo` a partir de variables de entorno simuladas.
pub fn detect_env_from(vars: &HashMap<String, String>, os: Os, has_layer_shell: bool) -> EnvInfo {
    todo!()
}

/// Elige y configura el primer backend utilizable según el entorno.
/// Registra en `log` el motivo de cada descarte.
pub fn select_backend(
    env: &EnvInfo,
    backends: Vec<Box<dyn WindowBackend>>,
    win: &mut WindowHandle,
    rect: Rect,
    log: &mut dyn FnMut(&str),
) -> Result<BackendId, SelectError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendError;

    struct FakeBackend {
        id: BackendId,
        probe: Probe,
        configure_ok: bool,
    }

    impl FakeBackend {
        fn ok(id: BackendId) -> Box<Self> {
            Box::new(Self {
                id,
                probe: Probe::Available,
                configure_ok: true,
            })
        }

        fn bad_configure(id: BackendId) -> Box<Self> {
            Box::new(Self {
                id,
                probe: Probe::Available,
                configure_ok: false,
            })
        }

        fn unavailable(id: BackendId, reason: &str) -> Box<Self> {
            Box::new(Self {
                id,
                probe: Probe::Unavailable(reason.to_string()),
                configure_ok: true,
            })
        }
    }

    impl WindowBackend for FakeBackend {
        fn id(&self) -> BackendId {
            self.id
        }

        fn probe(&self, _env: &EnvInfo) -> Probe {
            self.probe.clone()
        }

        fn configure(&mut self, _win: &mut WindowHandle, _rect: Rect) -> Result<(), BackendError> {
            if self.configure_ok {
                Ok(())
            } else {
                Err(BackendError::Failed("fallo simulado de configure".to_string()))
            }
        }

        fn set_interactive_region(
            &mut self,
            _win: &mut WindowHandle,
            _region: Option<Rect>,
        ) -> Result<(), BackendError> {
            Ok(())
        }
    }

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn rect() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 44.0,
        }
    }

    #[test]
    fn detect_x11_from_session_type() {
        let env = detect_env_from(&vars(&[("XDG_SESSION_TYPE", "x11")]), Os::Linux, false);
        assert_eq!(env.os, Os::Linux);
        assert_eq!(env.session, Session::X11);
    }

    #[test]
    fn detect_wayland_from_session_type() {
        let env = detect_env_from(&vars(&[("XDG_SESSION_TYPE", "wayland")]), Os::Linux, true);
        assert_eq!(env.session, Session::Wayland);
        assert!(env.has_layer_shell);
    }

    #[test]
    fn detect_unknown_when_no_vars() {
        let env = detect_env_from(&vars(&[]), Os::Linux, false);
        assert_eq!(env.session, Session::Unknown);
        // En Windows la sesión es Unknown aunque haya variables de sesión.
        let win = detect_env_from(&vars(&[("XDG_SESSION_TYPE", "wayland")]), Os::Windows, false);
        assert_eq!(win.session, Session::Unknown);
    }

    #[test]
    fn select_x11_session_prefers_x11() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::X11,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![FakeBackend::ok(BackendId::Fallback), FakeBackend::ok(BackendId::X11)],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::X11));
    }

    #[test]
    fn select_wayland_without_layer_shell_uses_fallback() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::Wayland,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![
                FakeBackend::ok(BackendId::WaylandLayerShell),
                FakeBackend::ok(BackendId::Fallback),
            ],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::Fallback));
    }

    #[test]
    fn select_windows_prefers_win32() {
        let env = EnvInfo {
            os: Os::Windows,
            session: Session::Unknown,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![FakeBackend::ok(BackendId::Fallback), FakeBackend::ok(BackendId::Win32)],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::Win32));
    }

    #[test]
    fn select_unknown_session_uses_fallback() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::Unknown,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![FakeBackend::ok(BackendId::X11), FakeBackend::ok(BackendId::Fallback)],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::Fallback));
    }

    #[test]
    fn select_falls_through_when_configure_fails_and_logs() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::X11,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![
                FakeBackend::bad_configure(BackendId::X11),
                FakeBackend::ok(BackendId::Fallback),
            ],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::Fallback));
        assert!(
            logs.iter().any(|l| l.contains("X11")),
            "el log debe contener el motivo del descarte de X11: {logs:?}"
        );
    }

    #[test]
    fn select_errors_when_nothing_works() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::X11,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![
                FakeBackend::bad_configure(BackendId::X11),
                FakeBackend::unavailable(BackendId::Fallback, "sin soporte de panel"),
            ],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Err(SelectError));
        assert_eq!(logs.len(), 2, "cada descarte debe registrarse: {logs:?}");
    }
}
