//! Compiles each C++ test in `commons/cpp/tests` with the system's C++ compiler (warnings as
//! errors) and runs it. The envelope test gets the golden vectors the Rust and TypeScript decoders
//! use, written out as plain text.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenFile {
    vectors: Vec<Vector>,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    hex: String,
    kind: u8,
    flags: u16,
    schema_tag_hex: String,
    major: u16,
    minor: u16,
    epoch: u32,
    sequence: String,
    correlation: String,
    sim_time: String,
    payload_crc32: u32,
    payload_hex: String,
}

fn cpp_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cpp")
}

/// Compiles `name.cpp` from `commons/cpp/tests` into `dir` and returns the executable.
fn compile(name: &str, dir: &Path) -> PathBuf {
    let source = cpp_dir().join("tests").join(format!("{name}.cpp"));
    let include = cpp_dir().join("include");
    let exe = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    let compiler = cc::Build::new()
        .cpp(true)
        .target(env!("COMMONS_CPP_TARGET"))
        .host(env!("COMMONS_CPP_HOST"))
        .opt_level(0)
        .cargo_metadata(false)
        .try_get_compiler()
        .expect("a C++ compiler");
    let mut command = compiler.to_command();
    if compiler.is_like_msvc() {
        command
            .args(["/nologo", "/std:c++17", "/EHsc", "/W4", "/WX"])
            .arg(format!("/I{}", include.display()))
            .arg(&source)
            .arg(format!("/Fe{}", exe.display()))
            .arg(format!("/Fo{}\\", dir.display()));
    } else {
        command
            .args(["-std=c++17", "-Wall", "-Wextra", "-Wpedantic", "-Werror"])
            .arg("-I")
            .arg(&include)
            .arg(&source)
            .arg("-o")
            .arg(&exe);
    }
    let output = command.output().expect("the C++ compiler runs");
    assert!(
        output.status.success(),
        "{name}.cpp did not compile:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    exe
}

fn run(exe: &Path, args: &[&Path]) -> String {
    let output = Command::new(exe)
        .args(args)
        .output()
        .expect("the test runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "{} failed:\n{stdout}\n{}",
        exe.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

#[test]
fn the_cpp_envelope_reads_and_writes_the_golden_vectors() {
    let dir = tempfile::tempdir().expect("temp dir");
    let golden: GoldenFile =
        serde_json::from_str(include_str!("../../commons-wire/tests/golden.json"))
            .expect("golden.json parses");
    let lines: Vec<String> = golden
        .vectors
        .iter()
        .map(|v| {
            let payload = if v.payload_hex.is_empty() {
                "-"
            } else {
                &v.payload_hex
            };
            format!(
                "{} {} {} {} {} {} {} {} {} {} {} {} {payload}",
                v.name,
                v.hex,
                v.kind,
                v.flags,
                v.schema_tag_hex,
                v.major,
                v.minor,
                v.epoch,
                v.sequence,
                v.correlation,
                v.sim_time,
                v.payload_crc32
            )
        })
        .collect();
    let vectors = dir.path().join("vectors.txt");
    std::fs::write(&vectors, lines.join("\n") + "\n").expect("writes the vectors");
    let exe = compile("wire_test", dir.path());
    let out = run(&exe, &[&vectors]);
    assert!(out.contains("wire_test: ok"), "{out}");
}

#[test]
fn the_cpp_timed_paths_agree_with_the_web_observer() {
    let dir = tempfile::tempdir().expect("temp dir");
    let exe = compile("timed_path_test", dir.path());
    let out = run(&exe, &[]);
    assert!(out.contains("timed_path_test: ok"), "{out}");
}
