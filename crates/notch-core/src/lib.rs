//! Núcleo del notch: tipos geométricos compartidos y módulos de lógica.
//! Sin dependencias de Qt ni del sistema operativo.

pub mod backend;
pub mod layout;
pub mod machine;
pub mod select;

/// Tamaño en píxeles lógicos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub w: f64,
    pub h: f64,
}

/// Rectángulo en píxeles lógicos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
