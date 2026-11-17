//! Ensamblador del notch: detección del entorno real del proceso.

pub mod logging;

use std::collections::HashMap;
use std::ffi::OsString;

use notch_core::backend::{EnvInfo, Os};
use notch_core::machine::NotchState;

/// Convierte variables crudas del entorno a texto sin paniquear ante valores
/// no UTF-8 (pérdida aceptable: solo se usan para detectar la sesión).
pub fn collect_vars<I>(iter: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    iter.into_iter()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.to_string_lossy().into_owned(),
            )
        })
        .collect()
}

/// Detecta el entorno real del proceso (variables y SO en ejecución).
pub fn detect_env() -> EnvInfo {
    let vars = collect_vars(std::env::vars_os());
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

    #[test]
    fn defers_shrink_on_expanded_to_resting() {
        assert!(defers_region_shrink(NotchState::Expanded, NotchState::Resting));
    }

    #[test]
    fn no_defer_on_resting_to_expanded() {
        assert!(!defers_region_shrink(NotchState::Resting, NotchState::Expanded));
    }

    #[test]
    fn no_defer_on_hidden_transitions() {
        for (prev, next) in [
            (NotchState::Hidden, NotchState::Resting),
            (NotchState::Resting, NotchState::Hidden),
            (NotchState::Hidden, NotchState::Expanded),
            (NotchState::Expanded, NotchState::Hidden),
        ] {
            assert!(
                !defers_region_shrink(prev, next),
                "{prev:?} -> {next:?} no debe diferir la región"
            );
        }
    }

    #[test]
    fn no_defer_on_same_state() {
        for state in [NotchState::Hidden, NotchState::Resting, NotchState::Expanded] {
            assert!(
                !defers_region_shrink(state, state),
                "{state:?} -> {state:?} no debe diferir la región"
            );
        }
    }
}
