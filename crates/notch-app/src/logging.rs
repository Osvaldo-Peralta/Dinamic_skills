//! Archivo de log en el directorio de estado estándar del SO.

use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

/// Crea el directorio y el archivo de log si no existen; devuelve su ruta.
pub fn init_logging_in(dir: &Path) -> io::Result<PathBuf> {
    todo!()
}

/// Directorio de estado estándar: XDG_STATE_HOME o ~/.local/state en Linux,
/// %LOCALAPPDATA% en Windows.
pub fn default_log_dir() -> PathBuf {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
