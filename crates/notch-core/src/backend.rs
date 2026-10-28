//! Contratos de los backends de ventana. Solo tipos: sin implementación.
//! Ningún tipo de este módulo puede depender de Qt.

use crate::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendId {
    X11,
    WaylandLayerShell,
    Fallback,
    Win32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Available,
    Unavailable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    X11,
    Wayland,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvInfo {
    pub os: Os,
    pub session: Session,
    pub has_layer_shell: bool,
}

/// Identificador opaco de ventana creada por un backend. Sin dependencia de Qt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle(u64);

impl WindowHandle {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    Unsupported,
    Failed(String),
}

pub trait WindowBackend {
    fn id(&self) -> BackendId;
    fn probe(&self, env: &EnvInfo) -> Probe;
    fn configure(&mut self, win: &mut WindowHandle, rect: Rect) -> Result<(), BackendError>;
    fn set_interactive_region(
        &mut self,
        win: &mut WindowHandle,
        region: Option<Rect>,
    ) -> Result<(), BackendError>;
}
