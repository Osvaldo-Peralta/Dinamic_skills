//! Registro de ventanas QML: guarda el `QWindow*` como `u64` y lo expone
//! como `WindowHandle` opaco (id = posición + 1). Todo el `unsafe` de
//! punteros queda contenido en `src/qt_helpers.cpp`; aquí solo se administran
//! enteros. Propuesta para T4-T6 (compartir entre backends): ver reporte.

use std::sync::{Mutex, MutexGuard};

use notch_core::backend::WindowHandle;
use notch_core::Rect;

use crate::bridge::qobject as ffi;

static WINDOWS: Mutex<Vec<u64>> = Mutex::new(Vec::new());

fn lock() -> MutexGuard<'static, Vec<u64>> {
    WINDOWS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Registra el QWindow que aloja al QObject del bridge y aplica la
/// configuración del backend de respaldo. Devuelve su `WindowHandle`.
pub fn register_window(bridge_qobject_ptr: u64) -> Option<WindowHandle> {
    let win = ffi::notch_find_host_window(bridge_qobject_ptr);
    if win == 0 {
        return None;
    }
    ffi::notch_configure_fallback(win);
    let mut reg = lock();
    if !reg.contains(&win) {
        reg.push(win);
    }
    let id = reg.iter().position(|p| *p == win)? as u64 + 1;
    Some(WindowHandle::new(id))
}

/// Ejecuta `f` con el puntero crudo del QWindow asociado al manejador.
/// Devuelve `None` si el manejador no está registrado.
pub fn with_window<R>(handle: WindowHandle, f: impl FnOnce(u64) -> R) -> Option<R> {
    let reg = lock();
    let idx = handle.id().checked_sub(1)? as usize;
    let ptr = *reg.get(idx)?;
    Some(f(ptr))
}

/// Primer manejador registrado (la aplicación crea una única ventana).
pub fn first_window() -> Option<WindowHandle> {
    lock().first().map(|_| WindowHandle::new(1))
}

pub fn set_geometry(handle: WindowHandle, rect: Rect) -> Option<()> {
    with_window(handle, |win| ffi::notch_set_geometry(win, rect.x, rect.y, rect.w, rect.h))
}

pub fn apply_mask(handle: WindowHandle, rect: Rect) -> Option<()> {
    with_window(handle, |win| ffi::notch_apply_mask(win, rect.x, rect.y, rect.w, rect.h))
}

pub fn clear_mask(handle: WindowHandle) -> Option<()> {
    with_window(handle, |win| ffi::notch_clear_mask(win))
}

/// Geometría global y devicePixelRatio del monitor principal.
pub fn primary_screen() -> Option<(Rect, f64)> {
    let (mut x, mut y, mut w, mut h, mut dpr) = (0.0, 0.0, 0.0, 0.0, 0.0);
    if ffi::notch_primary_screen(&mut x, &mut y, &mut w, &mut h, &mut dpr) {
        Some((Rect { x, y, w, h }, dpr))
    } else {
        None
    }
}
