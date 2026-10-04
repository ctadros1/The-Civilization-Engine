//! Compiles `harness.c` with the system's C compiler against the committed header and runs it on
//! the built library (ADR-0005): the library loads at run time, C drives a kernel through the
//! table, and C and Rust agree on every structure's size and field offsets.

mod common;

use std::mem::offset_of;
use std::path::Path;
use std::process::Command;

use tce_kernel::{TceApiV1, TceBytes, TceConfig, TcePanelConfig, TceVersions};

/// The sizes and field offsets Rust gives each structure, as the harness prints them for C.
fn rust_layouts() -> Vec<(&'static str, Vec<usize>)> {
    vec![
        (
            "TceBytes",
            vec![
                size_of::<TceBytes>(),
                offset_of!(TceBytes, ptr),
                offset_of!(TceBytes, len),
            ],
        ),
        (
            "TceConfig",
            vec![
                size_of::<TceConfig>(),
                offset_of!(TceConfig, struct_size),
                offset_of!(TceConfig, flags),
                offset_of!(TceConfig, content_dir),
                offset_of!(TceConfig, saves_dir),
            ],
        ),
        (
            "TcePanelConfig",
            vec![
                size_of::<TcePanelConfig>(),
                offset_of!(TcePanelConfig, struct_size),
                offset_of!(TcePanelConfig, port),
                offset_of!(TcePanelConfig, reserved),
                offset_of!(TcePanelConfig, web_dir),
            ],
        ),
        (
            "TceVersions",
            vec![
                size_of::<TceVersions>(),
                offset_of!(TceVersions, struct_size),
                offset_of!(TceVersions, abi_major),
                offset_of!(TceVersions, abi_minor),
                offset_of!(TceVersions, wire_major),
                offset_of!(TceVersions, wire_minor),
                offset_of!(TceVersions, save_schema),
                offset_of!(TceVersions, content_api),
            ],
        ),
        (
            "TceApiV1",
            vec![
                size_of::<TceApiV1>(),
                offset_of!(TceApiV1, struct_size),
                offset_of!(TceApiV1, abi_major),
                offset_of!(TceApiV1, abi_minor),
                offset_of!(TceApiV1, versions),
                offset_of!(TceApiV1, create),
                offset_of!(TceApiV1, destroy),
                offset_of!(TceApiV1, status),
                offset_of!(TceApiV1, last_error),
                offset_of!(TceApiV1, submit),
                offset_of!(TceApiV1, poll),
                offset_of!(TceApiV1, copy_snapshot),
                offset_of!(TceApiV1, serve_panels),
                offset_of!(TceApiV1, panel_url),
            ],
        ),
    ]
}

#[test]
fn c_loads_the_library_and_drives_a_kernel_with_the_same_layouts() {
    let dir = tempfile::tempdir().expect("temp dir");
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = common::compile(
        &crate_dir.join("tests").join("harness.c"),
        &[crate_dir.join("include")],
        false,
        dir.path(),
    );
    let saves = dir.path().join("saves");
    let content = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate");
    let output = Command::new(&exe)
        .arg(common::library())
        .arg(&content)
        .arg(&saves)
        .output()
        .expect("the harness runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the harness failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("harness: ok"), "{stdout}");
    assert!(stdout.contains("refused: malformed frame"), "{stdout}");
    for (name, expected) in rust_layouts() {
        let line = stdout
            .lines()
            .find(|l| l.starts_with(&format!("layout {name} ")))
            .unwrap_or_else(|| panic!("no layout for {name}:\n{stdout}"));
        let c: Vec<usize> = line
            .split_whitespace()
            .skip(2)
            .map(|n| n.parse().expect("a number"))
            .collect();
        assert_eq!(c, expected, "C and Rust disagree on {name}");
    }
}
