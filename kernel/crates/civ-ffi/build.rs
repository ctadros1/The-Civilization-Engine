//! Passes the target and host triples to the C harness test, which compiles C for the target the
//! library was built for.

fn main() {
    for var in ["TARGET", "HOST"] {
        if let Ok(value) = std::env::var(var) {
            println!("cargo:rustc-env=TCE_FFI_{var}={value}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
