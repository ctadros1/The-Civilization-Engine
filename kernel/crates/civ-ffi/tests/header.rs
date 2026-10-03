//! The C header is generated from the exported functions and types, and committed (ADR-0005 §1).
//! This test fails when it is stale; `TCE_BLESS=1 cargo test -p civ-ffi --test header` rewrites
//! it.

use std::path::Path;

#[test]
fn the_committed_header_matches_the_exports() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config =
        cbindgen::Config::from_file(crate_dir.join("cbindgen.toml")).expect("cbindgen.toml reads");
    let bindings = cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("the header generates");
    let mut generated = Vec::new();
    bindings.write(&mut generated);
    let path = crate_dir.join("include").join("tce_kernel.h");
    if std::env::var_os("TCE_BLESS").is_some() {
        std::fs::write(&path, &generated).expect("the header writes");
        return;
    }
    // A Windows checkout may have turned the line ends into CRLF.
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        committed.as_bytes() == generated.as_slice(),
        "include/tce_kernel.h is stale; run `TCE_BLESS=1 cargo test -p civ-ffi --test header`"
    );
}
