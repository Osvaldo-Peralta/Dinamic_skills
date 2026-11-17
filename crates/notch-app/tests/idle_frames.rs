//! Prueba automática de la Tarea 3 (plataforma offscreen):
//! con la píldora en reposo y las animaciones terminadas, no se dibujan
//! fotogramas durante 3 segundos (spec §Transiciones: "con el notch quieto,
//! las animaciones y el redibujado se detienen").

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use notch_core::machine::NotchState;
use notch_ui::events::{clear_listener, set_listener, UiEvent};

fn pump(app: &QGuiApplication, duration: Duration) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        app.process_events();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn idle_renders_no_frames_after_settle() {
    // Entorno virtual: sin display real; se fuerza la plataforma offscreen.
    if std::env::var_os("QT_QPA_PLATFORM").is_none() {
        std::env::set_var("QT_QPA_PLATFORM", "offscreen");
    }

    let frames = Arc::new(AtomicU64::new(0));
    let rested = Arc::new(AtomicBool::new(false));
    {
        let frames = Arc::clone(&frames);
        let rested = Arc::clone(&rested);
        set_listener(move |event| match *event {
            UiEvent::FrameDrawn => {
                frames.fetch_add(1, Ordering::Relaxed);
            }
            UiEvent::StateChanged(NotchState::Resting) => rested.store(true, Ordering::Relaxed),
            _ => {}
        });
    }

    let app = QGuiApplication::new();
    assert!(!app.is_null(), "QGuiApplication debe crearse (offscreen)");
    let app = app.as_ref().expect("QGuiApplication válida");
    let mut engine = QQmlApplicationEngine::new();
    assert!(!engine.is_null(), "QQmlApplicationEngine debe crearse");
    engine.pin_mut().load(&QUrl::from(notch_ui::NOTCH_QML_URL));

    // Espera el asentamiento en Resting (fake player activo por defecto).
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && !rested.load(Ordering::Relaxed) {
        app.process_events();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        rested.load(Ordering::Relaxed),
        "la píldora no llegó al estado Resting en 10 s"
    );

    // Margen para que terminen las animaciones de la transición inicial.
    pump(app, Duration::from_secs(1));

    let before = frames.load(Ordering::Relaxed);
    pump(app, Duration::from_secs(3));
    let after = frames.load(Ordering::Relaxed);
    clear_listener();
    assert_eq!(
        before, after,
        "con el notch quieto no deben dibujarse fotogramas: {before} -> {after}"
    );
}
