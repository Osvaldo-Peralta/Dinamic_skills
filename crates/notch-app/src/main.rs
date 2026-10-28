//! Ensamblador: inicializa el log (tolerando su fallo), detecta el entorno,
//! crea la aplicación Qt, carga Notch.qml y reacciona a los eventos del
//! bridge: al adjuntarse la ventana elige backend y la posiciona con
//! `compute_pill_rect` en el monitor principal; en cada cambio de estado
//! actualiza la región interactiva; si cambia el monitor, reposiciona.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use notch_core::backend::{BackendId, EnvInfo, WindowBackend, WindowHandle};
use notch_core::layout::{compute_pill_rect, interactive_region};
use notch_core::machine::NotchState;
use notch_core::select::select_backend;
use notch_core::Rect;
use notch_ui::events::{set_listener, UiEvent};
use notch_ui::{EXPANDED_SIZE, NOTCH_QML_URL, PILL_SIZE};
use notch_window_fallback::FallbackBackend;

/// Estado de la aplicación que usa el listener de eventos de la UI.
struct AppState {
    /// Instancia propia para actualizaciones posteriores a la selección
    /// (`select_backend` consume las suyas).
    backend: FallbackBackend,
    handle: WindowHandle,
    selected: Option<BackendId>,
    state: NotchState,
    log: Option<PathBuf>,
}

impl AppState {
    fn log_line(&mut self, line: &str) {
        match &self.log {
            Some(path) => {
                let written = OpenOptions::new()
                    .append(true)
                    .open(path)
                    .and_then(|mut f| writeln!(f, "{line}"));
                if let Err(err) = written {
                    eprintln!("no se pudo escribir el log: {err}");
                }
            }
            // Sin log disponible la aplicación sigue; el diagnóstico va a stderr.
            None => eprintln!("log: {line}"),
        }
    }

    /// Elige backend la primera vez (o reposiciona) y ajusta la región.
    fn reposition(&mut self, env: &EnvInfo) {
        let Some((screen, dpr)) = notch_ui::registry::primary_screen() else {
            self.log_line("no se pudo consultar el monitor principal");
            return;
        };
        // La ventana ocupa la envolvente expandida (decisión documentada en el
        // reporte); la píldora en reposo se dibuja centrada dentro.
        let win_rect = compute_pill_rect(screen, EXPANDED_SIZE, dpr);
        if self.selected.is_none() {
            let mut select_log: Vec<String> = Vec::new();
            let chosen = select_backend(
                env,
                vec![Box::new(FallbackBackend::new())],
                &mut self.handle,
                win_rect,
                &mut |msg| select_log.push(msg.to_string()),
            );
            for msg in &select_log {
                self.log_line(msg);
            }
            match chosen {
                Ok(id) => {
                    self.selected = Some(id);
                    self.log_line(&format!("backend activo: {id:?}"));
                }
                Err(_) => {
                    self.log_line("no se pudo seleccionar ningún backend");
                    return;
                }
            }
        } else if let Err(err) = self.backend.configure(&mut self.handle, win_rect) {
            self.log_line(&format!("error al reposicionar la ventana: {err:?}"));
        }
        self.apply_region(screen, win_rect, dpr);
    }

    /// Máscara interactiva según el estado, en coordenadas de ventana.
    fn apply_region(&mut self, screen: Rect, win_rect: Rect, dpr: f64) {
        let pill = compute_pill_rect(screen, PILL_SIZE, dpr);
        let resting = Rect {
            x: pill.x - win_rect.x,
            y: pill.y - win_rect.y,
            w: pill.w,
            h: pill.h,
        };
        let expanded = Rect {
            x: 0.0,
            y: 0.0,
            w: win_rect.w,
            h: win_rect.h,
        };
        let region = interactive_region(self.state, resting, expanded);
        if let Err(err) = self.backend.set_interactive_region(&mut self.handle, region) {
            self.log_line(&format!("error al actualizar la región interactiva: {err:?}"));
        }
    }

    fn on_state_changed(&mut self, state: NotchState) {
        self.state = state;
        if self.selected.is_none() {
            return;
        }
        if let Some((screen, dpr)) = notch_ui::registry::primary_screen() {
            let win_rect = compute_pill_rect(screen, EXPANDED_SIZE, dpr);
            self.apply_region(screen, win_rect, dpr);
        }
    }
}

fn main() -> ExitCode {
    // Un fallo del log no aborta el arranque (cambio de la Tarea 3 respecto
    // de la Tarea 2).
    let log = match notch_app::logging::init_logging_in(&notch_app::logging::default_log_dir()) {
        Ok(path) => Some(path),
        Err(err) => {
            eprintln!("no se pudo iniciar el log; la aplicación sigue sin registrar: {err}");
            None
        }
    };
    let env = notch_app::detect_env();
    let mut state = AppState {
        backend: FallbackBackend::new(),
        handle: WindowHandle::new(0),
        selected: None,
        state: NotchState::Hidden,
        log,
    };
    state.log_line(&format!(
        "entorno detectado: os={:?} session={:?} has_layer_shell={}",
        env.os, env.session, env.has_layer_shell
    ));

    let mut app = QGuiApplication::new();
    if app.is_null() {
        eprintln!("no se pudo crear QGuiApplication");
        return ExitCode::FAILURE;
    }
    let mut engine = QQmlApplicationEngine::new();
    if engine.is_null() {
        eprintln!("no se pudo crear QQmlApplicationEngine");
        return ExitCode::FAILURE;
    }

    // El listener debe registrarse antes de cargar el QML: la ventana se
    // adjunta durante la carga (Component.onCompleted).
    set_listener(move |event| match *event {
        UiEvent::WindowAttached(handle) => {
            state.handle = handle;
            state.reposition(&env);
        }
        UiEvent::StateChanged(next) => state.on_state_changed(next),
        UiEvent::ScreenChanged => {
            if state.selected.is_some() {
                state.reposition(&env);
            }
        }
        UiEvent::FrameDrawn => {}
    });

    engine.pin_mut().load(&QUrl::from(NOTCH_QML_URL));
    let code = app
        .as_mut()
        .expect("QGuiApplication válida")
        .exec();
    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
