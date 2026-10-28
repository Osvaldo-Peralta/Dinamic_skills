//! Ensamblador del notch: detección del entorno real del proceso.

pub mod logging;

use std::collections::HashMap;

use notch_core::backend::{EnvInfo, Os};

/// Detecta el entorno real del proceso (variables y SO en ejecución).
pub fn detect_env() -> EnvInfo {
    let vars: HashMap<String, String> = std::env::vars().collect();
    let os = if cfg!(windows) { Os::Windows } else { Os::Linux };
    // La detección real de layer-shell llega en la Tarea 5; por ahora siempre false.
    let has_layer_shell = false;
    notch_core::select::detect_env_from(&vars, os, has_layer_shell)
}
