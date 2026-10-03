//! Compiles `host.cpp`, a C++ host of the library as the Unreal plugin will be, with the system's
//! C++ compiler, and runs it on the built library (ADR-0005). It reads frames with
//! `commons/cpp`'s envelope, payloads with the generated C++ readers, builds a command with the
//! generated builders and watches the world it asked for arrive.

mod common;

use std::path::Path;
use std::process::Command;

#[test]
fn a_cpp_host_reads_frames_and_asks_for_a_world() {
    let dir = tempfile::tempdir().expect("temp dir");
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = common::repo();
    let exe = common::compile(
        &crate_dir.join("tests").join("host.cpp"),
        &[
            crate_dir.join("include"),
            repo.join("commons").join("cpp").join("include"),
            repo.join("commons/cpp/third_party/flatbuffers/include"),
            repo.join("kernel/crates/civ-schema/cpp"),
        ],
        true,
        dir.path(),
    );
    let content = civ_content::find_content_root(crate_dir).expect("content/ is above the crate");
    let output = Command::new(&exe)
        .arg(common::library())
        .arg(&content)
        .arg(dir.path().join("saves"))
        .output()
        .expect("the host runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the host failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("host: ok, Plugin Valley with"), "{stdout}");
}
