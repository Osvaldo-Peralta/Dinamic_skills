//! Arranque mínimo: inicializa el log y registra el entorno detectado.
//! La ventana llega en la Tarea 3.

use std::fs::OpenOptions;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = match notch_app::logging::init_logging_in(&notch_app::logging::default_log_dir()) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("no se pudo iniciar el log: {err}");
            return ExitCode::FAILURE;
        }
    };
    let env = notch_app::detect_env();
    let line = format!(
        "entorno detectado: os={:?} session={:?} has_layer_shell={}",
        env.os, env.session, env.has_layer_shell
    );
    match OpenOptions::new()
        .append(true)
        .open(&path)
        .and_then(|mut f| writeln!(f, "{line}"))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("no se pudo escribir el log: {err}");
            ExitCode::FAILURE
        }
    }
}
