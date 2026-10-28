//! Archivo de log en el directorio de estado estándar del SO.

use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

/// Crea el directorio y el archivo de log si no existen; devuelve su ruta.
pub fn init_logging_in(dir: &Path) -> io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("notch.log");
    OpenOptions::new().create(true).append(true).open(&path)?;
    Ok(path)
}

/// Directorio de estado estándar a partir de un lector de variables.
/// Los valores vacíos cuentan como ausentes.
pub fn default_log_dir_from(get: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    let present = |key: &str| get(key).filter(|v| !v.is_empty());
    if cfg!(windows) {
        present("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    } else if let Some(state) = present("XDG_STATE_HOME") {
        PathBuf::from(state)
    } else if let Some(home) = present("HOME") {
        PathBuf::from(home).join(".local").join("state")
    } else {
        PathBuf::from(".")
    }
}

/// Directorio de estado estándar: XDG_STATE_HOME o ~/.local/state en Linux,
/// %LOCALAPPDATA% en Windows.
pub fn default_log_dir() -> PathBuf {
    default_log_dir_from(&env_path)
}

fn env_path(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn default_log_dir_ignores_empty_xdg_state_home() {
        let reader = |key: &str| match key {
            "XDG_STATE_HOME" => Some(String::new()),
            "HOME" => Some("/home/ana".to_string()),
            _ => None,
        };
        let dir = default_log_dir_from(&reader);
        assert_eq!(dir, PathBuf::from("/home/ana/.local/state"));
    }

    #[cfg(unix)]
    #[test]
    fn default_log_dir_keeps_non_utf8_xdg_state_home() {
        use std::os::unix::ffi::OsStringExt;
        let raw = b"/tmp/xdg\xff".to_vec();
        let expected = PathBuf::from(OsString::from_vec(raw.clone()));
        let reader = |key: &str| match key {
            "XDG_STATE_HOME" => Some(OsString::from_vec(raw.clone())),
            _ => None,
        };
        let dir = default_log_dir_from(&reader);
        assert_eq!(dir, expected, "la ruta no UTF-8 debe conservarse byte a byte");
    }

    #[test]
    fn init_logging_writes_file() {
        let dir = std::env::temp_dir().join("notch-test-init_logging_writes_file");
        let _ = fs::remove_dir_all(&dir);
        let path = init_logging_in(&dir).expect("init_logging_in debe crear el archivo");
        assert!(path.exists(), "el archivo de log debe existir en {path:?}");
        {
            let mut f = OpenOptions::new()
                .append(true)
                .open(&path)
                .expect("debe poder abrirse en append");
            writeln!(f, "línea de prueba").expect("debe poder escribirse");
        }
        let content = fs::read_to_string(&path).expect("debe poder releerse");
        assert!(content.contains("línea de prueba"));
        let _ = fs::remove_dir_all(&dir);
    }
}
