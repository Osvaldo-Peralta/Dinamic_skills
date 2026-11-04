//! Bridge cxx-qt `NotchBridge`: propiedades e invocables que usa Notch.qml.
//!
//! Arquitectura (validada con el probe `probe-cxxqt`, ver reporte de Tarea 3):
//! - QML instancia el bridge (`#[qml_element]`); Rust no puede poseer el
//!   QObject porque cxx-qt 0.10 no expone el contexto QML ni registros de
//!   singleton desde Rust.
//! - La comunicación Rust puente -> aplicación va por un listener global
//!   (`crate::events`), no por manejadores del QObject.
//! - Los helpers de Qt se declaran como funciones libres `extern "C++"` con
//!   punteros crudos `u64`: cxx-qt 0.10 genera llamadas rotas si las firmas
//!   llevan `Pin<&mut QWindow>`, y moc no admite `QWindow&` como parámetro de
//!   un Q_INVOKABLE.

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Alias al tipo QString de cxx-qt-lib.
        type QString = cxx_qt_lib::QString;

        include!("src/qt_helpers.h");
        pub fn notch_find_host_window(bridge: u64) -> u64;
        pub fn notch_configure_fallback(win: u64);
        pub fn notch_set_geometry(win: u64, x: f64, y: f64, w: f64, h: f64);
        pub fn notch_apply_mask(win: u64, x: f64, y: f64, w: f64, h: f64);
        pub fn notch_clear_mask(win: u64);
        pub fn notch_primary_screen(x: &mut f64, y: &mut f64, w: &mut f64, h: &mut f64, dpr: &mut f64) -> bool;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, state)]
        #[qproperty(QString, title_text, cxx_name = "titleText")]
        #[qproperty(QString, artist_text, cxx_name = "artistText")]
        #[qproperty(bool, transition_pending, cxx_name = "transitionPending")]
        #[qproperty(i32, frame_count, cxx_name = "frameCount")]
        type NotchBridge = super::NotchBridgeRust;

        #[cxx_name = "onHover"]
        #[qinvokable]
        fn on_hover(self: Pin<&mut NotchBridge>, enter: bool);

        #[cxx_name = "tick"]
        #[qinvokable]
        fn tick(self: Pin<&mut NotchBridge>);

        #[cxx_name = "bumpFrame"]
        #[qinvokable]
        fn bump_frame(self: Pin<&mut NotchBridge>);

        #[cxx_name = "attachWindow"]
        #[qinvokable]
        fn attach_window(self: Pin<&mut NotchBridge>);

        #[cxx_name = "screenChanged"]
        #[qinvokable]
        fn screen_changed(self: Pin<&mut NotchBridge>);
    }

    impl cxx_qt::Constructor<()> for NotchBridge {}
}

use std::time::{Duration, Instant};

use notch_core::machine::{HoverEvent, NotchMachine, NotchState};

use crate::events::{emit, UiEvent};

/// Retardos iniciales de transición (spec §Transiciones, ajustables).
const ENTER_DELAY: Duration = Duration::from_millis(150);
const LEAVE_DELAY: Duration = Duration::from_millis(400);

/// Parte Rust del QObject. `machine` se rellena en `Initialize`.
pub struct NotchBridgeRust {
    state: QString,
    title_text: QString,
    artist_text: QString,
    transition_pending: bool,
    frame_count: i32,
    machine: Option<NotchMachine>,
    last_state: NotchState,
}

impl Default for NotchBridgeRust {
    fn default() -> Self {
        Self {
            state: QString::from("hidden"),
            title_text: QString::default(),
            artist_text: QString::default(),
            transition_pending: false,
            frame_count: 0,
            machine: None,
            last_state: NotchState::Hidden,
        }
    }
}

fn state_str(state: NotchState) -> &'static str {
    match state {
        NotchState::Hidden => "hidden",
        NotchState::Resting => "resting",
        NotchState::Expanded => "expanded",
    }
}

/// Publica el estado en la propiedad QML y notifica a la aplicación si cambió.
fn publish_state(mut self_: Pin<&mut qobject::NotchBridge>, state: NotchState, pending: bool) {
    let changed = {
        let mut rust = self_.as_mut().rust_mut();
        let changed = rust.last_state != state;
        rust.last_state = state;
        changed
    };
    self_.as_mut().set_state(QString::from(state_str(state)));
    self_.as_mut().set_transition_pending(pending);
    if changed {
        emit(&UiEvent::StateChanged(state));
    }
}

/// Datos falsos del reproductor (Hito 1): presentes salvo
/// `NOTCH_FAKE_PLAYER=0` (entonces el notch permanece oculto).
fn fake_player_present() -> bool {
    std::env::var("NOTCH_FAKE_PLAYER")
        .map(|v| v.trim() != "0")
        .unwrap_or(true)
}

impl cxx_qt::Initialize for qobject::NotchBridge {
    fn initialize(mut self: Pin<&mut Self>) {
        let now = Instant::now();
        let mut machine = NotchMachine::new(ENTER_DELAY, LEAVE_DELAY);
        machine.set_player_present(fake_player_present(), now);
        let state = machine.tick(now);
        self.as_mut().rust_mut().machine = Some(machine);
        self.as_mut().set_title_text(QString::from("Título de ejemplo"));
        self.as_mut().set_artist_text(QString::from("Artista de ejemplo"));
        publish_state(self, state, false);
    }
}

impl qobject::NotchBridge {
    pub fn on_hover(mut self: Pin<&mut Self>, enter: bool) {
        let now = Instant::now();
        let Some((state, pending)) = self.as_mut().rust_mut().machine.as_mut().map(|machine| {
            machine.on_hover(if enter { HoverEvent::Enter } else { HoverEvent::Leave }, now);
            (machine.tick(now), machine.transition_pending())
        }) else {
            return;
        };
        publish_state(self, state, pending);
    }

    pub fn tick(mut self: Pin<&mut Self>) {
        let now = Instant::now();
        let Some((state, pending)) = self
            .as_mut()
            .rust_mut()
            .machine
            .as_mut()
            .map(|machine| (machine.tick(now), machine.transition_pending()))
        else {
            return;
        };
        publish_state(self, state, pending);
    }

    pub fn bump_frame(mut self: Pin<&mut Self>) {
        // frameCount solo tiene sentido en builds de depuración.
        if cfg!(debug_assertions) {
            let n = {
                let rust = self.as_mut().rust_mut();
                rust.frame_count + 1
            };
            self.as_mut().set_frame_count(n);
        }
        emit(&UiEvent::FrameDrawn);
    }

    pub fn attach_window(self: Pin<&mut Self>) {
        use cxx_qt::casting::Upcast;
        // &cxx_qt::QObject tiene la misma dirección que el QObject* de C++.
        let this: &Self = self.as_ref().get_ref();
        let base = <Self as Upcast<cxx_qt::QObject>>::upcast(this);
        let bridge_ptr = base as *const cxx_qt::QObject as u64;
        match crate::registry::register_window(bridge_ptr) {
            Some(handle) => emit(&UiEvent::WindowAttached(handle)),
            None => eprintln!("attach_window: no se encontró el QWindow anfitrión"),
        }
    }

    pub fn screen_changed(self: Pin<&mut Self>) {
        emit(&UiEvent::ScreenChanged);
    }
}
