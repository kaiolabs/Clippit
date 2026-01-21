fn main() {
    // Sem necessidade de cxx-qt-build por enquanto
    // Vamos usar uma abordagem mais simples
    println!("cargo:rerun-if-changed=src/config_model.rs");
    println!("cargo:rerun-if-changed=src/history_model.rs");
    println!("cargo:rerun-if-changed=src/theme_model.rs");
}
