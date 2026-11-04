use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    let builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("com.notch.ui").qml_file("qml/Notch.qml"),
    )
    .files(["src/bridge.rs"]);
    // SAFETY: únicamente añadimos nuestro .cpp auxiliar al compilador C++ de cc.
    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.file("src/qt_helpers.cpp");
        })
    };
    builder.build();
}
