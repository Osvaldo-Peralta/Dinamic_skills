//! Detección de entorno y selección de backend con respaldo. Lógica pura.

use std::collections::HashMap;

use crate::backend::{BackendId, EnvInfo, Os, Probe, Session, WindowBackend, WindowHandle};
use crate::Rect;

/// Ningún backend de la lista de preferencia pudo configurarse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectError;

/// Construye `EnvInfo` a partir de variables de entorno simuladas.
pub fn detect_env_from(vars: &HashMap<String, String>, os: Os, has_layer_shell: bool) -> EnvInfo {
    let session = match os {
        Os::Windows => Session::Unknown,
        Os::Linux => {
            // Una variable con valor vacío cuenta como ausente.
            let present = |key: &str| vars.get(key).map(String::as_str).filter(|v| !v.is_empty());
            match present("XDG_SESSION_TYPE") {
                Some("x11") => Session::X11,
                Some("wayland") => Session::Wayland,
                // Ausente o con valor no reconocido: se mira el display activo.
                _ => {
                    if present("WAYLAND_DISPLAY").is_some() {
                        Session::Wayland
                    } else if present("DISPLAY").is_some() {
                        Session::X11
                    } else {
                        Session::Unknown
                    }
                }
            }
        }
    };
    EnvInfo {
        os,
        session,
        has_layer_shell,
    }
}

/// Lista de preferencia de backends según el entorno (spec §7).
fn preference(env: &EnvInfo) -> Vec<BackendId> {
    match env.os {
        Os::Windows => vec![BackendId::Win32, BackendId::Fallback],
        Os::Linux => match env.session {
            Session::X11 => vec![BackendId::X11, BackendId::Fallback],
            Session::Wayland if env.has_layer_shell => {
                vec![BackendId::WaylandLayerShell, BackendId::Fallback]
            }
            Session::Wayland | Session::Unknown => vec![BackendId::Fallback],
        },
    }
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
    let mut backends = backends;
    for id in preference(env) {
        let Some(idx) = backends.iter().position(|b| b.id() == id) else {
            log(&format!("backend {id:?} no registrado; se prueba el siguiente"));
            continue;
        };
        let backend = &mut backends[idx];
        match backend.probe(env) {
            Probe::Available => {}
            Probe::Unavailable(reason) => {
                log(&format!("backend {id:?} no disponible: {reason}; se prueba el siguiente"));
                continue;
            }
        }
        match backend.configure(win, rect) {
            Ok(()) => return Ok(id),
            Err(err) => log(&format!(
                "backend {id:?} falló al configurar la ventana: {err:?}; se prueba el siguiente"
            )),
        }
    }
    Err(SelectError)
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
    fn detect_empty_display_vars_are_absent() {
        let env = detect_env_from(
            &vars(&[("DISPLAY", ""), ("WAYLAND_DISPLAY", "")]),
            Os::Linux,
            false,
        );
        assert_eq!(env.session, Session::Unknown);
    }

    #[test]
    fn detect_unrecognized_session_type_falls_back_to_display_vars() {
        let env = detect_env_from(
            &vars(&[("XDG_SESSION_TYPE", "tty"), ("DISPLAY", ":0")]),
            Os::Linux,
            false,
        );
        assert_eq!(env.session, Session::X11);
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
    fn select_wayland_with_layer_shell_prefers_layer_shell() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::Wayland,
            has_layer_shell: true,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![
                FakeBackend::ok(BackendId::Fallback),
                FakeBackend::ok(BackendId::WaylandLayerShell),
            ],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::WaylandLayerShell));
    }

    #[test]
    fn select_skips_unregistered_backend_and_logs() {
        let env = EnvInfo {
            os: Os::Linux,
            session: Session::X11,
            has_layer_shell: false,
        };
        let mut win = WindowHandle::new(1);
        let mut logs = Vec::new();
        let got = select_backend(
            &env,
            vec![FakeBackend::ok(BackendId::Fallback)],
            &mut win,
            rect(),
            &mut |s| logs.push(s.to_string()),
        );
        assert_eq!(got, Ok(BackendId::Fallback));
        assert!(
            logs.iter().any(|l| l.contains("X11")),
            "debe registrar el backend no registrado: {logs:?}"
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
