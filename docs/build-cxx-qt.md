# Cómo llegan los inicializadores de cxx-qt al enlace final

Tarea 3b (item 2). Este documento explica el mecanismo exacto —archivos,
directivas de build script y flags de enlazador— por el que los objetos
generados por cxx-qt-build (`public-initializer.o`, que DEFINE
`cxx_qt_init_crate_notch_ui` y `cxx_qt_init_qml_module_com_notch_ui`, más
`bridge.cxx.o`, mocs, qrc y recursos QML) terminan dentro de cada binario
final del workspace (`notch-app`, `idle_frames`, tests de
`notch-window-fallback`). Todo lo afirmado aquí se verificó sobre Debian 12
con rustc 1.99.1 y 1.91.1 (ver `reportes/reporte-tarea-3b.md` fuera del
repo) capturando la línea de enlace real con un wrapper de `cc` y listando
los miembros de los rlib con `ar t`.

## 1. El único build script cxx-qt del workspace

`crates/notch-ui/build.rs` (15 líneas) es el único build script del
workspace que usa `CxxQtBuilder`:

- `CxxQtBuilder::new_qml_module(QmlModule::new("com.notch.ui").qml_file("qml/Notch.qml"))`
- `.files(["src/bridge.rs"])` — el bridge `NotchBridge`.
- `unsafe { builder.cc_builder(|cc| cc.file("src/qt_helpers.cpp")) }` — los
  helpers C++ de u64-puntero usados por `registry`.
- `builder.build()`.

`crates/notch-app` y `crates/notch-window-fallback` **no tienen build
script** (deliberadamente: ver §5). `crates/notch-app/Cargo.toml` conserva
`cxx-qt-build` en `[build-dependencies]` sin `build.rs`; cargo lo ignora.

Qué genera `build()` en `OUT_DIR` de notch-ui (cxx-qt-build 0.10.0):

| Artefacto | Contenido relevante |
|---|---|
| `libnotch-ui-cxxqt-generated.a` | `public-initializer.o` ×2 (claves `crate_notch_ui` y `qml_module_com_notch_ui`; definen `cxx_qt_init_*` con `std::call_once` y encadenan los init de dependencias), `bridge.cxx.o`, `bridge.cxxqt.o`, `moc_bridge.cxxqt.h.o`, `qt_helpers.o`, `com_notch_ui_qmltyperegistration.o`, `com_notch_ui_plugin.o`, recursos qrc/qmlcache (`Notch.qml.o`, `qmlcache_loader.o`). |
| `libcxx-qt-call-init-crate_notch_ui.a`, `libcxx-qt-call-init-qml_module_com_notch_ui.a` | `call-initializers.o`: solo referencias INDEFINIDAS a los `cxx_qt_init_*` anteriores. |

Directivas que emite ese build script (salida literal observada con
`cargo build -vv`):

- `cargo:rustc-link-lib=static=notch-ui-cxxqt-generated` (la emite `cc`
  1.10.0 en `Build::compile`, `src/lib.rs:2161-2169`) y
  `cargo:rustc-link-search=native=<OUT_DIR>`.
- `cargo:rustc-link-lib=static:+whole-archive=cxx-qt-call-init-crate_notch_ui`
  y `...qml_module_com_notch_ui` (cxx-qt-build 0.10.0, `build_initializers`,
  `src/lib.rs:1157-1178`: el modificador `+whole-archive` garantiza que el
  objeto que llama a los init no se descarte).

## 2. El mecanismo que hace que funcionen: bundling en el rlib + whole-archive al principio de la línea

1. **`+bundle` por defecto.** Las directivas `static=` sin modificadores
   llevan implícito `+bundle` (rustc book, *linking modifiers*): cuando
   rustc compila `libnotch_ui.rlib`, **incrusta los miembros del archivo
   estático dentro del propio rlib**. Verificado:

   ```console
   $ ar t target/debug/deps/libnotch_ui-*.rlib | grep -v rcgu
   lib.rmeta
   lib.rmeta-link
   98b6e9e47a0de5e6-qt_helpers.o
   061b42e4c9ae982f-moc_bridge.cxxqt.h.o
   af8b4f1635559606-bridge.cxx.o
   af8b4f1635559606-bridge.cxxqt.o
   aecab4359334c7e3-com_notch_ui_qmltyperegistration.o
   0b2a87f5d45d1b0d-com_notch_ui_plugin.o
   0e0609ecf1c027c9-qml_module_resources_com_notch_ui.o
   3201ae67416bc6b1-Notch.qml.o
   3201ae67416bc6b1-qmlcache_loader.o
   3f71be2735af1846-public-initializer.o
   e72dc421d1c5e115-public-initializer.o
   libcxx-qt-call-init-qml_module_com_notch_ui.a
   libcxx-qt-call-init-crate_notch_ui.a
   ```

   Los `.o` generados Y los dos archivos `call-init` (anidados como `.a`)
   viajan dentro del rlib. Lo mismo aplica a `libcxx_qt_lib.rlib` y
   `libcxx_qt.rlib` con sus propios inicializadores.

2. **Colocación en el enlace final.** Al enlazar un binario/test, rustc
   extrae los `.a` anidados de los rlib a un directorio temporal
   `target/debug/deps/rustcXXXX/` y los pasa **por ruta absoluta y envueltos
   en `--whole-archive` al principio de la línea**, antes que los rlib.
   Línea de enlace real del test `idle_frames` (capturada con wrapper de
   `cc`; recortada a los flags relevantes):

   ```text
   "cc" "-m64" "<...>/deps/rustcXgw4kp/symbols.o" \
     "-Wl,--as-needed" "-Wl,-Bstatic" \
     "-Wl,--whole-archive" "-Wl,<...>/deps/rustcXgw4kp/libcxx-qt-call-init-qml_module_com_notch_ui.a" "-Wl,--no-whole-archive" \
     "-Wl,--whole-archive" "-Wl,<...>/deps/rustcXgw4kp/libcxx-qt-call-init-crate_notch_ui.a" "-Wl,--no-whole-archive" \
     "-Wl,--whole-archive" "-Wl,<...>/deps/rustcXgw4kp/libcxx-qt-call-init-crate_cxx_qt_lib.a" "-Wl,--no-whole-archive" \
     "-Wl,--whole-archive" "-Wl,<...>/deps/rustcXgw4kp/libcxx-qt-call-init-crate_cxx_qt.a" "-Wl,--no-whole-archive" \
     "-Wl,-Bdynamic" "-lQt6Qml" ... \
     "<31 rlibs, entre ellos libnotch_ui-dd21347680a4f047.rlib y libnotch_core-*.rlib>" \
     "-B<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld" "-fuse-ld=lld" ...
   ```

   Nótese lo que NO aparece: ningún `-lnotch-ui-cxxqt-generated`. No hace
   falta: los objetos ya están dentro de `libnotch_ui.rlib`.

3. **Resolución.** Los objetos `call-init` (whole-archive ⇒ siempre
   incluidos) crean referencias indefinidas a `cxx_qt_init_crate_notch_ui`,
   `cxx_qt_init_qml_module_com_notch_ui`, `cxx_qt_init_crate_cxx_qt_lib` y
   `cxx_qt_init_crate_cxx_qt` **desde el principio del enlace**. Cuando el
   enlazador escanea los rlib (archivos posicionales), extrae los miembros
   `public-initializer.o` empaquetados que los definen. Verificado en el
   binario resultante:

   ```console
   $ nm target/debug/deps/idle_frames-* | grep cxx_qt_init
   000000000028683f T cxx_qt_init_crate_cxx_qt
   000000000020c343 T cxx_qt_init_crate_cxx_qt_lib
   0000000000168d65 T cxx_qt_init_crate_notch_ui
   0000000000161742 T cxx_qt_init_qml_module_com_notch_ui
   ```

4. **Los init de cxx-qt y cxx-qt-lib llegan por propagación de dependencias.**
   `cxx-qt` y `cxx-qt-lib` declaran clave `links` en su `Cargo.toml` y su
   build script hace `Interface::export()` (escribe `manifest.json` en su
   `OUT_DIR` y emite `cargo::metadata=CXX_QT_MANIFEST_PATH=...`). Cargo
   expone eso al build script de notch-ui como
   `DEP_CXX_QT_CXX_QT_MANIFEST_PATH` / `DEP_CXX_QT_LIB_CXX_QT_MANIFEST_PATH`;
   `CxxQtBuilder::build()` los descubre (`Dependency::find_all()`) y
   reemite sus archivos `call-init` con `+whole-archive` (los `.o` de sus
   archivos generados no hacen falta reemitirlos: ya viajan bundled en
   `libcxx_qt.rlib` / `libcxx_qt_lib.rlib`). Por eso la línea de §2.2
   incluye 4 archivos call-init aunque el workspace solo declare un
   `CxxQtBuilder` propio.

## 3. Enlazador efectivo y flags

- rustc pasa `-B<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld`
  y `-fuse-ld=lld`: el enlazador efectivo es el **LLD empaquetado con la
  toolchain de Rust** (no el gold/bfd del sistema).
- qt-build-utils 0.10.0 emite además `cargo:rustc-link-arg=-fuse-ld=gold`
  cuando detecta GNU ld, pero `rustc-link-arg` solo aplica a
  bins/tests/ejemplos/cdylib **del paquete cuyo build script lo emite**: en
  la práctica solo afecta a los targets de test del propio notch-ui (de ahí
  el aviso `the gold linker is deprecated` al ejecutar `cargo test`, que
  enlazan igual de bien), y no se propaga a los paquetes consumidores, cuyo
  enlazador efectivo sigue siendo LLD. (Si un paquete con binarios añade su
  propio `CxxQtBuilder`, su enlace sí recibe `-fuse-ld=gold` al final de la
  línea, que tiene precedencia sobre `-fuse-ld=lld`; ver §5.)
- Variables de entorno que importan en este entorno:
  `QMAKE=/usr/bin/qmake6` (detección de Qt 6.4.2), `CARGO_BUILD_JOBS=1` y
  `CARGO_PROFILE_DEV_DEBUG=0` (entorno de ~1 GB de RAM; el perfil de debug
  ya no se fija en el repo — Tarea 3b item 7), `QT_QPA_PLATFORM=offscreen`
  para los tests. Para el enlace no se necesita ninguna variable extra.

## 4. Por qué los `.o` del `OUT_DIR` de notch-ui existen pero "no llegan al enlace"

Los `public-initializer.o` sueltos que se ven en
`target/debug/build/notch-ui-*/out/` son entradas intermedias: el enlace
final no los referencia por ruta ni por `-l`. Lo que llega al enlace son
sus **copias empaquetadas dentro de `libnotch_ui.rlib`** (§2). Un análisis
del fallo "undefined reference to `cxx_qt_init_*`" que solo inspeccione el
`OUT_DIR` y busque `-lnotch-ui-cxxqt-generated` en la línea de enlace no
encontrará ninguno de los dos y concluirá erróneamente que los objetos se
perdieron. La comprobación correcta es `ar t` sobre el rlib (§2.1) y la
presencia de los `-Wl,--whole-archive .../libcxx-qt-call-init-*.a` al
principio de la línea (§2.2).

## 5. Peligro verificado: añadir build scripts cxx-qt a los consumidores

Durante esta tarea se experimentó con añadir `CxxQtBuilder::new().build()`
a `notch-app`/`notch-window-fallback` (con `links` + `export()` en
notch-ui). **Rompe el enlace** de dos formas (reproducido localmente,
logs `/tmp/ws-v4b-*.log` del entorno de trabajo):

1. El paquete pasa a recibir `-fuse-ld=gold` (§3) y, si reemite
   `static=<dep>-cxxqt-generated` descubiertos por `DEP_*_CXX_QT_MANIFEST_PATH`,
   esos archivos entran como `-l` simples colocados **antes** de los rlib:
   en un enlace de una sola pasada, el enlazador extrae de ahí miembros
   (`bridge.cxx.o`) cuyas referencias a los shims `cxxbridge1$*` de Rust ya
   no pueden resolverse (el rlib correspondiente se escanea después, o ni
   siquiera es entrada de ese target: el lib-test de notch-app no enlaza
   `notch_ui` en absoluto).
2. Forzar símbolos con `-Wl,--undefined=cxx_qt_init_*` de forma
   incondicional arrastra los mismos objetos en targets que no incluyen los
   rlib con la parte Rust (21 referencias indefinidas observadas, entre
   ellas `cxx_qt_init_crate_cxx_qt` y `cxxbridge1$205$NotchBridge$*`).

Conclusión adoptada: **los paquetes consumidores no llevan build script**;
el mecanismo de §2 basta y se verificó verde en clon fresco con rustc
1.99.1 y 1.91.1.

## 6. Diagnóstico para entornos donde falle el enlace

Si en otro entorno (p. ej. Ubuntu 24.04 + Rust 1.91.1 + Qt 6.4.2) aparece
`undefined reference to cxx_qt_init_*`, comprobar en este orden:

1. `cargo build --workspace --tests -vv`: la salida del build script de
   notch-ui debe incluir `rustc-link-lib=static=notch-ui-cxxqt-generated`,
   `rustc-link-lib=static:+whole-archive=cxx-qt-call-init-crate_notch_ui` y
   `...qml_module_com_notch_ui`.
2. `ar t target/debug/deps/libnotch_ui-*.rlib | grep -c public-initializer`
   → esperado `2`. Si es `0`, el bundling no ocurrió (toolchain o flags
   `-C` no estándar); el enlace final dependería entonces de un
   `-lnotch-ui-cxxqt-generated` que rustc ya no emite y aparecerían
   indefinidos.
3. En la línea de enlace del target que falla deben aparecer los 4
   `-Wl,--whole-archive .../libcxx-qt-call-init-*.a` al principio. Si
   falta alguno, revisar que nadie sobrescriba las claves `links` de
   cxx-qt/cxx-qt-lib ni construya con features recortadas.
4. Verificar que ningún paquete consumidor añade su propio build script
   cxx-qt ni emite `rustc-link-lib` manualmente (§5).
5. `ldd $(which cc)` / `cc -v`: confirmar que `-B<sysroot>/.../bin/gcc-ld
   -fuse-ld=lld` tiene efecto (gcc ≥ 12 en Ubuntu 24.04 respeta `-B`).

## 7. Versiones exactas del entorno de verificación (item 3)

- Debian 12 (bookworm), kernel 4.19.91-c8dfc93.al7.x86_64, headless
  (Xvfb para pruebas X11; Wayland sin verificar en vivo).
- rustc 1.99.1 (58cf8f9c3 2026-10-13, LLVM 23.1.1) / cargo 1.99.1
  (5f94df478 2026-08-27); rustc 1.91.1 (ed61e7d7e) / cargo 1.91.1
  (ea2d97820). Enlazador efectivo: LLD empaquetado con rustc
  (`<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld`).
  Del sistema: GNU gold 1.16 y GNU ld (bfd) 2.40 presentes pero NO
  efectivos; lld del sistema ausente.
- gcc/g++ 12.2.0 (12.2.0-14+deb12u1); Qt 6.4.2 en
  `/usr/lib/x86_64-linux-gnu`; `QMAKE=/usr/bin/qmake6`; cmake 3.25.1.

## 8. Reconstrucción del entorno (el sandbox se recicla)

```bash
# Toolchains (rustup) y utilidades Qt:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
  --default-toolchain 1.99.1 --profile minimal
rustup toolchain install 1.91.1 --profile minimal
apt-get install -y qmake6 libqt6gui6 qt6-base-dev qt6-declarative-dev \
  qml6-module-qtquick qml6-module-qtquick-window qml6-module-qtqml \
  qml6-module-qtquick-controls qml6-module-qtqml-workerscript xvfb
. "$HOME/.cargo/env"
export QMAKE=/usr/bin/qmake6 CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0
cargo build --workspace --tests
QT_QPA_PLATFORM=offscreen cargo test --workspace
```
