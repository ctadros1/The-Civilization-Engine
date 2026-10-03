//! Compiles `harness.c` with the system's C compiler against the committed header and runs it on
//! the built library (ADR-0005): the library loads at run time, C drives a kernel through the
//! table, and C and Rust agree on every structure's size and field offsets.

use std::mem::offset_of;
use std::path::{Path, PathBuf};
use std::process::Command;

use tce_kernel::{TceApiV1, TceBytes, TceConfig, TcePanelConfig, TceVersions};

/// The library cargo built for this test run, next to the test binary.
fn library() -> PathBuf {
    let name = format!(
        "{}tce_kernel{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let exe = std::env::current_exe().expect("the test's path");
    let deps = exe.parent().expect("the deps folder");
    [
        deps.to_path_buf(),
        deps.parent().expect("the profile folder").to_path_buf(),
    ]
    .into_iter()
    .map(|dir| dir.join(&name))
    .find(|path| path.exists())
    .unwrap_or_else(|| panic!("{name} was not built next to {}", exe.display()))
}

/// Compiles the harness into `dir` and returns the executable.
fn compile(dir: &Path) -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = crate_dir.join("tests").join("harness.c");
    let include = crate_dir.join("include");
    let exe = dir.join(format!("harness{}", std::env::consts::EXE_SUFFIX));
    let compiler = cc::Build::new()
        .target(env!("TCE_FFI_TARGET"))
        .host(env!("TCE_FFI_HOST"))
        .opt_level(0)
        .cargo_metadata(false)
        .try_get_compiler()
        .expect("a C compiler");
    let mut command = compiler.to_command();
    if compiler.is_like_msvc() {
        command
            .arg("/nologo")
            .arg("/W3")
            .arg("/WX")
            .arg(format!("/I{}", include.display()))
            .arg(&source)
            .arg(format!("/Fe{}", exe.display()))
            .arg(format!("/Fo{}\\", dir.display()));
    } else {
        command
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg("-I")
            .arg(&include)
            .arg(&source)
            .arg("-o")
            .arg(&exe);
        if cfg!(target_os = "linux") {
            command.arg("-ldl");
        }
    }
    let output = command.output().expect("the C compiler runs");
    assert!(
        output.status.success(),
        "the harness did not compile:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    exe
}

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
    let exe = compile(dir.path());
    let saves = dir.path().join("saves");
    let content = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate");
    let output = Command::new(&exe)
        .arg(library())
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
