//! What the tests that compile C and C++ hosts share: finding the library cargo built and
//! compiling a host with the system's compiler (GCC or Clang, MSVC on Windows).

// Each test uses part of this module.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// The library cargo built for this test run, next to the test binary.
pub fn library() -> PathBuf {
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

/// The repository's root, three levels above this crate.
pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Compiles `source` (C, or C++ when `cpp`) with warnings as errors into `dir`, with `include`
/// folders, and returns the executable.
pub fn compile(source: &Path, include: &[PathBuf], cpp: bool, dir: &Path) -> PathBuf {
    let stem = source.file_stem().expect("a file name").to_string_lossy();
    let exe = dir.join(format!("{stem}{}", std::env::consts::EXE_SUFFIX));
    let compiler = cc::Build::new()
        .cpp(cpp)
        .target(env!("TCE_FFI_TARGET"))
        .host(env!("TCE_FFI_HOST"))
        .opt_level(0)
        .cargo_metadata(false)
        .try_get_compiler()
        .expect("a compiler");
    let mut command = compiler.to_command();
    if compiler.is_like_msvc() {
        command.args(["/nologo", "/W3", "/WX"]);
        if cpp {
            command.args(["/std:c++17", "/EHsc"]);
        }
        for dir in include {
            command.arg(format!("/I{}", dir.display()));
        }
        command
            .arg(source)
            .arg(format!("/Fe{}", exe.display()))
            .arg(format!("/Fo{}\\", dir.display()));
    } else {
        command.args(["-Wall", "-Wextra", "-Werror"]);
        command.arg(if cpp { "-std=c++17" } else { "-std=c11" });
        for dir in include {
            command.arg("-I").arg(dir);
        }
        command.arg(source).arg("-o").arg(&exe);
        if cfg!(target_os = "linux") {
            command.args(["-ldl", "-pthread"]);
        }
    }
    let output = command.output().expect("the compiler runs");
    assert!(
        output.status.success(),
        "{} did not compile:\n{}\n{}",
        source.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    exe
}
