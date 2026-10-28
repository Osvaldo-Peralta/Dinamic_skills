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

use notch_core::machine::NotchState;

use crate::events::{emit, UiEvent};

/// Parte Rust del QObject. `machine` se rellena en `Initialize`.
pub struct NotchBridgeRust {
    state: QString,
    title_text: QString,
    artist_text: QString,
    transition_pending: bool,
    frame_count: i32,
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

impl cxx_qt::Initialize for qobject::NotchBridge {
    fn initialize(self: Pin<&mut Self>) {
        // TODO(Tarea 3, paso siguiente): integrar NotchMachine y datos falsos.
        publish_state(self, NotchState::Hidden, false);
    }
}

impl qobject::NotchBridge {
    pub fn on_hover(self: Pin<&mut Self>, _enter: bool) {
        // TODO(Tarea 3, paso siguiente): alimentar NotchMachine.
    }

    pub fn tick(self: Pin<&mut Self>) {
        // TODO(Tarea 3, paso siguiente): NotchMachine::tick.
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
