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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[cfg(unix)]
    #[test]
    fn collect_vars_does_not_panic_on_non_utf8_value() {
        use std::os::unix::ffi::OsStringExt;
        let input = vec![(
            OsString::from("CLAVE"),
            OsString::from_vec(vec![0xFFu8, 0xFE]),
        )];
        let vars = collect_vars(input);
        assert_eq!(
            vars.get("CLAVE").map(String::as_str),
            Some("\u{FFFD}\u{FFFD}"),
            "el valor no UTF-8 debe convertirse con to_string_lossy, sin pánico"
        );
    }
}
