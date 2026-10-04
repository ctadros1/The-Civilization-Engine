//! Passes the target and host triples to the tests, which compile C++ for the target.

fn main() {
    for var in ["TARGET", "HOST"] {
        if let Ok(value) = std::env::var(var) {
            println!("cargo:rustc-env=COMMONS_CPP_{var}={value}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
